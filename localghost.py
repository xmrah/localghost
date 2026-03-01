#!/usr/bin/env python3
"""
LocalGhost CLI — Local AI Terminal Assistant for Linux
https://github.com/xmrah/localghost

Privacy-first, offline, distro-aware command generator.
Powered by Ollama + any local LLM.
"""

import sys
import json
import urllib.request
import urllib.error
import re
import os
import shutil
import subprocess
import threading
import time
from datetime import datetime, timedelta
from pathlib import Path

__version__ = "0.8.1"

# --- DATA DIRECTORY ---
DATA_DIR = Path.home() / ".local" / "share" / "localghost"
ENV_CACHE_FILE = DATA_DIR / "env.json"
HISTORY_FILE = DATA_DIR / "history.json"
ENV_CACHE_MAX_AGE = 86400  # 24 hours in seconds
HISTORY_TTL_DAYS = int(os.environ.get("LOCALGHOST_HISTORY_TTL", "7"))
HISTORY_MAX_ENTRIES = 100
HISTORY_CONTEXT_COUNT = 5  # How many recent entries to inject into prompt

# --- SETTINGS ---
DEFAULT_MODEL = "gemma3:latest"
OLLAMA_BASE_URL = os.environ.get("LOCALGHOST_OLLAMA_URL", "http://127.0.0.1:11434")
MAX_QUERY_LENGTH = 500  # Characters — prevents abuse / accidental paste floods

# --- SAFETY FILTER ---
# Patterns that indicate potentially destructive commands.
# Case-insensitive matching is used at check time.
DANGEROUS_PATTERNS = [
    # Recursive/forced deletion targeting critical paths
    r"rm\s+.*(-[a-z]*r|-[a-z]*f|--recursive|--force).*(/|~|\$HOME)",
    r"rm\s+(-rf|-fr)\b",
    # Disk formatting / partitioning / raw writes
    r"mkfs\b",
    r"dd\s+.*(if|of)=.*/dev/",
    r"wipefs\b",
    r"fdisk\s+/dev/",
    r"parted\s+/dev/",
    # Fork bomb variants
    r":\(\)\{.*:\|:",
    r"\.\s*\(\)\s*\{.*\|",
    # Overwriting block devices
    r">\s*/dev/(sd|nvme|vd|hd|loop)",
    # Dangerous permissions on system dirs
    r"chmod\s+(-[a-z]*\s+)?777\s+/",
    r"chmod\s+(-[a-z]*\s+)?(777|666)\s+/etc",
    r"chown\s+.*\s+/\s",
    r"chown\s+.*\s+/$",
    # Moving / overwriting root
    r"mv\s+/\s",
    r"mv\s+/$",
    # Remote code execution (piping from internet to shell)
    r"(curl|wget)\s+.*\|\s*(sudo\s+)?(bash|sh|zsh|python|perl)",
    r"(curl|wget)\s+.*-o\s*-\s*\|",
    # eval / exec with variables
    r"eval\s+\$",
    r"eval\s+['\"].*\$",
    r"exec\s+\$",
    # Python/Perl one-liner attacks
    r"python[23]?\s+-c\s+.*os\.(system|remove|unlink|rmdir)",
    r"perl\s+-e\s+.*unlink",
    # History manipulation
    r"history\s+-c",
    r"shred.*\.(bash_history|zsh_history)",
    # Kernel / boot / critical file destruction
    r"rm\s+.*/boot/",
    r"rm\s+.*/etc/(passwd|shadow|fstab)",
]


def check_safety(command):
    """Returns (is_safe, matched_pattern) for a given command string."""
    for pattern in DANGEROUS_PATTERNS:
        if re.search(pattern, command, re.IGNORECASE):
            return False, pattern
    return True, None


def get_distro_info():
    """Detects the Linux distribution from /etc/os-release."""
    distro_name = "Linux"
    distro_id = "linux"
    distro_pkg = "unknown"
    try:
        if os.path.exists("/etc/os-release"):
            with open("/etc/os-release", "r") as f:
                data = {}
                for line in f:
                    if "=" in line:
                        key, val = line.strip().split("=", 1)
                        data[key] = val.strip('"')
                distro_name = data.get("NAME", "Linux")
                distro_id = data.get("ID", "linux")

            # Map distro to its package manager
            pkg_map = {
                "nixos": "nixos-rebuild / nix",
                "arch": "pacman",
                "artix": "pacman",
                "manjaro": "pacman",
                "endeavouros": "pacman",
                "debian": "apt",
                "ubuntu": "apt",
                "linuxmint": "apt",
                "pop": "apt",
                "fedora": "dnf",
                "rhel": "dnf",
                "centos": "dnf",
                "opensuse-tumbleweed": "zypper",
                "opensuse-leap": "zypper",
                "void": "xbps",
                "alpine": "apk",
                "gentoo": "emerge",
            }
            distro_pkg = pkg_map.get(distro_id, "unknown")
    except Exception:
        pass
    return distro_name, distro_id, distro_pkg


def get_hardware_info():
    """Detects GPU and CPU vendor for context-aware command generation."""
    gpu_vendor = "unknown"
    cpu_vendor = "unknown"

    # Detect GPU vendor via lspci
    try:
        result = subprocess.run(
            ["lspci"], capture_output=True, text=True, timeout=3
        )
        lspci_out = result.stdout.lower()
        if "nvidia" in lspci_out:
            gpu_vendor = "nvidia"
        elif "amd" in lspci_out or "radeon" in lspci_out:
            gpu_vendor = "amd"
        elif "intel" in lspci_out:
            gpu_vendor = "intel"
    except Exception:
        pass

    # Detect CPU vendor via /proc/cpuinfo
    try:
        with open("/proc/cpuinfo", "r") as f:
            cpuinfo = f.read().lower()
            if "amd" in cpuinfo:
                cpu_vendor = "amd"
            elif "intel" in cpuinfo:
                cpu_vendor = "intel"
            elif "arm" in cpuinfo or "aarch64" in cpuinfo:
                cpu_vendor = "arm"
    except Exception:
        pass

    return gpu_vendor, cpu_vendor


def ensure_data_dir():
    """Creates the data directory if it doesn't exist."""
    DATA_DIR.mkdir(parents=True, exist_ok=True)


def detect_environment():
    """Detects shell aliases and tool replacements.
    Checks if common commands are actually modern replacements.
    Results are cached in env.json for 24 hours."""

    # Check cache freshness
    if ENV_CACHE_FILE.exists():
        try:
            age = time.time() - ENV_CACHE_FILE.stat().st_mtime
            if age < ENV_CACHE_MAX_AGE:
                with open(ENV_CACHE_FILE, "r") as f:
                    return json.load(f)
        except Exception:
            pass

    env = {
        "shell": os.environ.get("SHELL", "unknown"),
        "aliases": {},
        "detected_at": datetime.now().isoformat(),
    }

    # Detect common tool replacements via --version output
    checks = {
        "find": {"test": ["find", "--version"], "marker": "fd", "real": "fd"},
        "ls": {"test": ["ls", "--version"], "marker": "eza", "real": "eza"},
        "cat": {"test": ["cat", "--version"], "marker": "bat", "real": "bat"},
        "grep": {"test": ["grep", "--version"], "marker": "ripgrep", "real": "rg"},
    }

    for cmd, info in checks.items():
        try:
            result = subprocess.run(
                info["test"], capture_output=True, text=True, timeout=2
            )
            output = (result.stdout + result.stderr).lower()
            if info["marker"] in output:
                env["aliases"][cmd] = info["real"]
        except Exception:
            pass

    # Save cache
    ensure_data_dir()
    try:
        with open(ENV_CACHE_FILE, "w") as f:
            json.dump(env, f, indent=2)
    except Exception:
        pass

    return env


def load_history():
    """Loads command history, pruning expired entries."""
    if not HISTORY_FILE.exists():
        return []

    try:
        with open(HISTORY_FILE, "r") as f:
            entries = json.load(f)
    except Exception:
        return []

    # Prune expired entries
    cutoff = (datetime.now() - timedelta(days=HISTORY_TTL_DAYS)).isoformat()
    active = [e for e in entries if e.get("ts", "") >= cutoff]

    # Enforce max entries
    if len(active) > HISTORY_MAX_ENTRIES:
        active = active[-HISTORY_MAX_ENTRIES:]

    # Write back pruned list if changed
    if len(active) != len(entries):
        save_history(active)

    return active


def save_history(entries):
    """Persists history to disk."""
    ensure_data_dir()
    try:
        with open(HISTORY_FILE, "w") as f:
            json.dump(entries, f, indent=2)
    except Exception:
        pass


def append_history(query, command):
    """Appends a query-command pair to history."""
    entries = load_history()
    entries.append({
        "ts": datetime.now().isoformat(),
        "query": query,
        "cmd": command,
    })
    if len(entries) > HISTORY_MAX_ENTRIES:
        entries = entries[-HISTORY_MAX_ENTRIES:]
    save_history(entries)


def validate_command(command):
    """Checks if the first word of the command exists as a binary on the system."""
    # Extract the actual command (skip sudo, env, nix-shell prefixes)
    skip_prefixes = ("sudo", "env", "nix-shell", "doas", "pkexec")
    parts = command.split()
    cmd_name = None
    for part in parts:
        if part.startswith("-"):
            continue
        if part in skip_prefixes:
            continue
        if "=" in part:  # env VAR=val
            continue
        cmd_name = part
        break

    if not cmd_name:
        return True, None  # Can't determine, let it pass

    if shutil.which(cmd_name):
        return True, None
    else:
        return False, cmd_name


def get_available_models():
    """Fetches available models from Ollama API."""
    try:
        url = f"{OLLAMA_BASE_URL}/api/tags"
        with urllib.request.urlopen(url, timeout=5) as response:
            if response.status == 200:
                data = json.loads(response.read().decode("utf-8"))
                return [m["name"] for m in data.get("models", [])]
    except Exception:
        pass
    return []


def sanitize_input(text):
    """Strip control characters and enforce length limit."""
    # Remove ASCII control chars (0x00-0x1F) except newline/tab
    cleaned = re.sub(r"[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]", "", text)
    if len(cleaned) > MAX_QUERY_LENGTH:
        cleaned = cleaned[:MAX_QUERY_LENGTH]
        print(f"# Warning: Query truncated to {MAX_QUERY_LENGTH} chars.", file=sys.stderr)
    return cleaned


def spinner_task(stop_event):
    """Displays a waiting spinner on stderr."""
    frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
    i = 0
    while not stop_event.is_set():
        print(f"\r{frames[i % len(frames)]} Thinking...", end="", file=sys.stderr, flush=True)
        i += 1
        time.sleep(0.1)
    print("\r                    \r", end="", file=sys.stderr, flush=True)


def clean_response(raw_output):
    """Cleans LLM output: removes think blocks, markdown, and conversational filler."""
    # 1. Remove <think>...</think> blocks (Deepseek R1 reasoning traces)
    cleaned = re.sub(r"<think>.*?</think>", "", raw_output, flags=re.DOTALL)
    # 2. Handle unclosed <think> tags (model cut off mid-thought)
    cleaned = re.sub(r"<think>.*", "", cleaned, flags=re.DOTALL)
    cleaned = cleaned.strip()

    # 3. Remove markdown code blocks
    cleaned = re.sub(r"```[a-z]*\n?", "", cleaned)
    cleaned = re.sub(r"```", "", cleaned)

    # 4. Parse lines — skip empty, shell names, conversational filler
    lines = [line.strip() for line in cleaned.split("\n") if line.strip()]
    skip_prefixes = ("to ", "here ", "sure ", "okay ", "the ", "this ", "you ", "i ")

    for line in lines:
        if line.endswith(":"):
            continue
        if line.lower() in ("bash", "sh", "zsh", "fish"):
            continue
        if line.lower().startswith(skip_prefixes):
            continue
        return line

    # Fallback: return last non-shell line
    for line in reversed(lines):
        if line.lower() not in ("bash", "sh", "zsh", "fish"):
            return line

    return ""


def print_help():
    """Prints usage information."""
    print(f"\033[1mLocalGhost ({__version__}) - Local AI Terminal Assistant\033[0m")
    print("Usage: localghost [OPTIONS] \"YOUR QUERY\"")
    print()
    print("Options:")
    print("  <query>              Generate a command from natural language")
    print("  --help               Show this help message")
    print("  --version            Show version")
    print("  --models             List available Ollama models")
    print("  --history            Show recent command history")
    print("  --env                Show detected environment profile")
    print("  --clear-history      Clear all command history")
    print("  --refresh-env        Force re-scan environment")
    print()
    print("Environment Variables:")
    print(f"  LOCALGHOST_OLLAMA_URL       Ollama API URL (default: {OLLAMA_BASE_URL})")
    print(f"  LOCALGHOST_HISTORY_TTL      History retention in days (default: 7)")
    print()
    print("Examples:")
    print('  localghost "update my system"')
    print('  localghost "find large files over 100MB"')
    print('  localghost "show disk usage"')
    print()
    print(f"Data: {DATA_DIR}")
    print("Privacy: All processing happens locally. No data is sent externally.")


def print_env():
    """Displays detected environment profile."""
    env = detect_environment()
    print(f"\033[1mEnvironment Profile\033[0m")
    print(f"  Shell: {env.get('shell', 'unknown')}")
    print(f"  Detected: {env.get('detected_at', 'unknown')}")
    aliases = env.get("aliases", {})
    if aliases:
        print(f"  Aliases:")
        for orig, real in aliases.items():
            print(f"    {orig} → {real}")
    else:
        print(f"  Aliases: none detected")
    print(f"  Cache: {ENV_CACHE_FILE}")


def print_history():
    """Displays recent command history."""
    entries = load_history()
    if not entries:
        print("No history yet.")
        return
    print(f"\033[1mCommand History ({len(entries)} entries, TTL: {HISTORY_TTL_DAYS} days)\033[0m")
    for e in entries[-20:]:  # Show last 20
        ts = e.get("ts", "")[:16].replace("T", " ")
        print(f"  [{ts}] {e.get('query', '')}")
        print(f"           → {e.get('cmd', '')}")


def print_models():
    """Lists available Ollama models."""
    models = get_available_models()
    if models:
        print(f"Available models ({len(models)}):")
        for m in models:
            marker = " ← default" if m == DEFAULT_MODEL else ""
            print(f"  • {m}{marker}")
    else:
        print("No models found. Is Ollama running?")
        print(f"Tried: {OLLAMA_BASE_URL}/api/tags")


def main():
    # --- CLI flags ---
    if len(sys.argv) < 2:
        print_help()
        sys.exit(0)

    arg = sys.argv[1]
    if arg in ("--help", "-h"):
        print_help()
        sys.exit(0)
    elif arg in ("--version", "-v"):
        print(f"localghost {__version__}")
        sys.exit(0)
    elif arg in ("--models", "--list-models"):
        print_models()
        sys.exit(0)
    elif arg == "--history":
        print_history()
        sys.exit(0)
    elif arg == "--env":
        print_env()
        sys.exit(0)
    elif arg == "--clear-history":
        if HISTORY_FILE.exists():
            HISTORY_FILE.unlink()
            print("✓ History cleared.")
        else:
            print("No history to clear.")
        sys.exit(0)
    elif arg == "--refresh-env":
        if ENV_CACHE_FILE.exists():
            ENV_CACHE_FILE.unlink()
        env = detect_environment()
        print("✓ Environment re-scanned.")
        print_env()
        sys.exit(0)

    # --- Main flow ---
    raw_query = " ".join(sys.argv[1:])
    user_query = sanitize_input(raw_query)

    if not user_query.strip():
        print("# Error: Empty query.", file=sys.stderr)
        sys.exit(1)

    # 1. System Detection
    distro_name, distro_id, distro_pkg = get_distro_info()
    gpu_vendor, cpu_vendor = get_hardware_info()

    # Build hardware context string
    hw_hints = []
    if gpu_vendor == "amd":
        hw_hints.append("GPU is AMD (use radeontop, rocm-smi, or sensors for GPU info, NOT nvidia-smi)")
    elif gpu_vendor == "nvidia":
        hw_hints.append("GPU is NVIDIA (use nvidia-smi for GPU info)")
    elif gpu_vendor == "intel":
        hw_hints.append("GPU is Intel (use intel_gpu_top for GPU info)")
    if cpu_vendor:
        hw_hints.append(f"CPU is {cpu_vendor.upper()}")
    hw_context = ". ".join(hw_hints) + "." if hw_hints else ""

    # 2. Environment Profiling
    env = detect_environment()
    alias_hints = []
    for orig, real in env.get("aliases", {}).items():
        alias_hints.append(f"Modern replacement for {orig} is {real}")
    env_context = ". ".join(alias_hints) + "." if alias_hints else ""

    shell_name = os.path.basename(env.get("shell", "bash"))

    # 3. History Context
    history = load_history()
    history_context = ""
    if history:
        recent = history[-HISTORY_CONTEXT_COUNT:]
        history_lines = [f"Q: {e['query']} → A: {e['cmd']}" for e in recent]
        history_context = f"Recent commands for context: {'; '.join(history_lines)}"

    # 4. Model Selection
    available_models = get_available_models()
    selected_model = DEFAULT_MODEL

    if available_models:
        if DEFAULT_MODEL not in available_models:
            selected_model = available_models[0]
            print(
                f"# Note: '{DEFAULT_MODEL}' not found. Using '{selected_model}'.",
                file=sys.stderr,
            )

    # 5. Dynamic System Prompt
    system_prompt = (
        "You are an expert Linux terminal command generator. Output ONLY the raw command. "
        "EXAMPLES: "
        "- User: 'git status' -> Command: git status "
        "- User: 'find large files' -> Command: fd --size +500M "
        "- User: 'update apps' -> Command: nix-env -u "
        "- User: 'github push' -> Command: git push "
        "- User: 'git komutları' -> Command: git --help "
        "- User: 'nixos manuel güncelleme' -> Command: nix-env --upgrade '*' "
        "\nRULES: "
        "1. Prefer the specific tool's CLI (git, nix, docker) over search tools. "
        "2. Use search tools (fd, find, rg) ONLY if the user wants to LOCATE files in the filesystem. "
        f"3. Package Management ({distro_name}): Use {distro_pkg}. For user apps on NixOS, prefer 'nix-env' or 'nix profile'. "
        "\nCONTEXT: "
        f"Hardware: {hw_context} "
        f"Aliases: {env_context} "
        f"History: {history_context if history_context else 'None'}"
    )

    payload = {
        "model": selected_model,
        "prompt": user_query,
        "system": system_prompt,
        "stream": False,
        "options": {"temperature": 0.4, "num_ctx": 2048},
    }

    # 6. Send request with spinner
    stop_spinner = threading.Event()
    spinner = threading.Thread(target=spinner_task, args=(stop_spinner,), daemon=True)

    try:
        req = urllib.request.Request(
            f"{OLLAMA_BASE_URL}/api/generate",
            data=json.dumps(payload).encode("utf-8"),
            headers={"Content-Type": "application/json"},
        )

        spinner.start()
        with urllib.request.urlopen(req, timeout=90) as response:
            stop_spinner.set()
            spinner.join()
            if response.status == 200:
                result = json.loads(response.read().decode("utf-8"))
                raw_output = result.get("response", "").strip()
                final_cmd = clean_response(raw_output)

                if final_cmd:
                    is_safe, pattern = check_safety(final_cmd)
                    if not is_safe:
                        print(f"\033[91m⚠  DANGEROUS COMMAND DETECTED\033[0m")
                        print(f"\033[91m   Pattern: {pattern}\033[0m")
                        print(f"\033[93m   Command: {final_cmd}\033[0m")
                        print(f"   Review carefully before executing.")
                    else:
                        # Validate that the command binary exists
                        cmd_exists, missing = validate_command(final_cmd)
                        if not cmd_exists:
                            print(f"\033[93m# Warning: '{missing}' not found on this system.\033[0m", file=sys.stderr)
                        print(final_cmd)

                    # Save to history (even dangerous ones, for context)
                    append_history(user_query, final_cmd)
                else:
                    print("# Error: Model returned empty response.", file=sys.stderr)
            else:
                print(f"# Error: HTTP {response.status}", file=sys.stderr)

    except urllib.error.URLError:
        stop_spinner.set()
        spinner.join()
        print("# Error: Cannot connect to Ollama.", file=sys.stderr)
        print(f"# Is Ollama running at {OLLAMA_BASE_URL}?", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        stop_spinner.set()
        spinner.join()
        print(f"# Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
