# Ghost CLI

**Local AI terminal assistant for Linux.** Translates natural language into terminal commands using your own hardware. No cloud, no telemetry, no internet required.

> Ghost runs 100% locally via [Ollama](https://ollama.com). Your queries never leave your machine.

## Features
- **Distro-aware** — Detects your Linux distribution and generates appropriate commands (`pacman`, `apt`, `dnf`, `nix`, etc.)
- **Privacy-first** — All inference runs locally on your GPU/CPU via Ollama
- **Safety-first** — Flags dangerous commands (`rm -rf`, `mkfs`, `curl | bash`) before you execute them
- **Zero dependencies** — Pure Python 3 standard library, no `pip install` needed
- **Model-agnostic** — Works with any Ollama model (Gemma, Deepseek, Llama, Mistral, etc.)

## Supported Distributions
NixOS • Arch • Artix • Manjaro • EndeavourOS • Debian • Ubuntu • Mint • Pop!_OS • Fedora • RHEL • CentOS • openSUSE • Void • Alpine • Gentoo

## Requirements
| Requirement | Notes |
|---|---|
| Linux | Any modern distribution with `/etc/os-release` |
| Python 3.6+ | For f-string support |
| [Ollama](https://ollama.com) | Running locally on port 11434 |
| Any LLM model | `ollama pull gemma2:2b` (fast) or `ollama pull deepseek-r1:8b` (smart) |

## Installation

```bash
git clone https://github.com/xmrah/ghost.git
cd ghost
./install.sh
```

The interactive installer offers two modes:

**Quick Setup** (recommended for new users):
- Checks for Python 3 and Ollama
- Offers to install Ollama if missing (distro-specific instructions)
- Offers to pull a starter AI model
- Symlinks `ghost.py` to `~/.local/bin/ghost`
- Configures your PATH automatically (fish/bash/zsh)

**Expert Setup** (for experienced users):
- Just creates the symlink, no checks

```bash
./install.sh              # Interactive wizard
./install.sh --dry-run    # See what would happen
./install.sh --uninstall  # Clean removal
./install.sh --help       # Show all options
```

**NixOS users** can also use the provided `shell.nix`:
```bash
nix-shell
python3 ghost.py "update system"
```

## Usage

```bash
# System updates
ghost "update my system"
#   NixOS  → sudo nixos-rebuild switch --upgrade
#   Arch   → sudo pacman -Syu
#   Debian → sudo apt update && sudo apt upgrade

# Find files
ghost "find all mp4 files larger than 100MB"

# System info
ghost "show disk usage sorted by size"

# Network
ghost "list all open ports"
```

## CLI Flags

| Flag | Description |
|---|---|
| `--help`, `-h` | Show usage information |
| `--version`, `-v` | Print version |
| `--models` | List available Ollama models |

## Configuration

| Environment Variable | Default | Description |
|---|---|---|
| `GHOST_OLLAMA_URL` | `http://127.0.0.1:11434` | Ollama API endpoint |

## Safety

Ghost includes a built-in safety filter that detects potentially destructive commands:

```
$ ghost "delete everything on the system"
⚠  DANGEROUS COMMAND DETECTED
   Pattern: rm\s+(-[a-z]*r[a-z]*\s+|--recursive).*(/|~|\$HOME)
   Command: rm -rf /
   Review carefully before executing.
```

Ghost will **never** auto-execute commands. It only prints them. You decide what to run.

## How It Works

```
┌──────────────┐     ┌──────────┐     ┌───────────┐
│ Your query   │────▶│ ghost.py │────▶│  Ollama   │
│ "update sys" │     │ (local)  │     │  (local)  │
└──────────────┘     └────┬─────┘     └─────┬─────┘
                          │                 │
                          │◀────────────────┘
                          │   raw LLM output
                          ▼
                    ┌─────────────┐
                    │ Clean +     │
                    │ Safety Check│
                    └──────┬──────┘
                           │
                           ▼
                    sudo pacman -Syu
```

## Privacy Statement

- Ghost communicates **only** with `localhost` (127.0.0.1)
- No analytics, no telemetry, no crash reports
- No data is stored between runs
- Source code is fully auditable (single Python file)

## License

MIT

---

🇹🇷 [Türkçe Kullanım Kılavuzu](README-TR.md)
