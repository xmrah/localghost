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

__version__ = "0.7.0"

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
                "nixos": "nix/nixos-rebuild",
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
    print(f"# LocalGhost CLI (v{__version__})")
    print("# https://github.com/xmrah/localghost")
    print()
    print(f"\033[1mLocalGhost ({__version__}) - Local AI Terminal Assistant\033[0m")
    print("Usage: localghost [OPTIONS] \"YOUR QUERY\"")
    print()
    print("Options:")
    print("  localghost <query>         Generate a command from natural language")
    print("  localghost --help          Show this help message")
    print("  localghost --version       Show version")
    print("  localghost --models        List available Ollama models")
    print()
    print("Environment:")
    print(f"  LOCALGHOST_OLLAMA_URL           Ollama API URL (default: {OLLAMA_BASE_URL})")
    print()
    print("Examples:")
    print('  localghost "update my system"')
    print('  localghost "find large files over 100MB"')
    print('  localghost "show disk usage"')
    print()
    print("Safety: Dangerous commands (rm -rf, mkfs, dd) are flagged with warnings.")
    print("Privacy: All processing happens locally via Ollama. No data is sent externally.")


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

    # 2. Model Selection
    available_models = get_available_models()
    selected_model = DEFAULT_MODEL

    if available_models:
        if DEFAULT_MODEL not in available_models:
            selected_model = available_models[0]
            print(
                f"# Note: '{DEFAULT_MODEL}' not found. Using '{selected_model}'.",
                file=sys.stderr,
            )

    # 3. Dynamic System Prompt
    system_prompt = (
        "You are a Linux terminal command generator. "
        "Output ONLY a single shell command. No explanations, no markdown, no backticks. "
        "Rules: "
        "1. Use standard POSIX/GNU commands for file, directory, process, network, and disk operations. "
        f"2. For package management ONLY, use {distro_pkg} (this system runs {distro_name}). "
        f"3. Hardware: {hw_context} "
        "4. Prefer built-in flags over pipes. "
        "5. Never generate destructive commands unless explicitly asked. "
        "6. If the request is ambiguous, choose the safest interpretation."
    )

    payload = {
        "model": selected_model,
        "prompt": user_query,
        "system": system_prompt,
        "stream": False,
        "options": {"temperature": 0.2, "num_ctx": 2048},
    }

    # 4. Send request with spinner
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
