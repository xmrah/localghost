# LocalLocalGhost CLI 👻

> **Local AI Terminal Assistant for Linux**
> *Zero dependencies. 100% Privacy. Hybrid & Distribution Agnostic.*

![Architecture](https://github.com/xmrah/locallocalghost/blob/main/assets/demo.gif?raw=true)

**LocalLocalGhost** is a lightweight (~400 lines of Python) CLI tool that uses [Ollama](https://ollama.com) to translate natural language into Linux terminal commands.

It is designed to be **safe, transparent, and distro-agnostic**. It works on NixOS, Arch, Debian, Fedora, and more.

## Why LocalLocalGhost?

- **🔒 Privacy First:** Runs incorrectly offline. No data leaves your machine.
- **🛡️ Safe:** Features a strict regex-based safety filter to block destructive commands (`rm -rf`, fork bombs, etc.).
- **🐧 Hybrid:** Smartly detects your distro (NixOS vs Arch vs Debian) and hardware (AMD vs NVIDIA).
- **⚡ Fast:** Written in pure Python standard library. No `pip install` required.ed
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

**Requirements:** Linux, Python 3.8+, [Ollama](https://ollama.com)

```bash
# 1. Clone repo
git clone https://github.com/xmrah/locallocalghost.git
cd locallocalghost

# 2. Run interactive installer
./install.sh
```

The interactive installer offers two modes:

**Quick Setup** (recommended for new users):
- Checks for Python 3 and Ollama
- Offers to install Ollama if missing (distro-specific instructions)
- Offers to pull a starter AI model
- Symlinks `locallocalghost.py` to `~/.local/bin/locallocalghost`
- Configures your PATH automatically (fish/bash/zsh)

**Expert Setup** (for experienced users):
- Just creates the symlink, no checks

```bash
./install.sh              # Interactive wizard
./install.sh --dry-run    # See what would happen
./install.sh --uninstall  # Clean removal
./install.sh --help       # Show all options
```

### NixOS Users (Zero-Install)

Run LocalLocalGhost instantly without cloning or installing anything:

```bash
# Run directly from GitHub
nix run github:xmrah/locallocalghost -- "update my system"
```

Or start a development shell with all dependencies (Python 3, hardware tools):

```bash
# Clone and enter dev environment
git clone https://github.com/xmrah/locallocalghost.git
cd locallocalghost
nix develop
```

## Usage

```bash
# System updates
locallocalghost "update my system"
#   NixOS  → sudo nixos-rebuild switch --upgrade
#   Arch   → sudo pacman -Syu
#   Debian → sudo apt update && sudo apt upgrade -y

# Complex finds
locallocalghost "find all pdf files larger than 100MB modified in the last 7 days"
#   → find . -name "*.pdf" -size +100M -mtime -7

# Hardware info (Context Aware)
locallocalghost "show gpu info"
#   AMD GPU → radeontop / sensors
#   NVIDIA  → nvidia-smi
```

### Options

| Flag | Description |
|------|-------------|
| `--help` | Show usage help |
| `--version` | Show version |
| `--models` | List available Ollama models |

## Configuration

LocalLocalGhost works out of the box, but you can configure it via environment variables:

```bash
export LOCALGHOST_OLLAMA_URL="http://127.0.0.1:11434"  # Default
```

## Safety & Privacy

**LocalLocalGhost is designed to be safe.**

1. **Read-Only by default:** It only *prints* commands. It never executes them automatically.
2. **Safety Filter:** Blocks known destructive patterns like `rm -rf`, `/dev/sda` writes, etc.
3. **Local Only:** Your queries never leave `localhost`.

> ⚠️ **Disclaimer:** AI models can hallucinate. Always review the command before running it.

## How It Works

```
┌──────────────┐     ┌──────────┐     ┌───────────┐
│ Your query   │────▶│ localghost.py │────▶│  Ollama   │
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

- LocalGhost communicates **only** with `localhost` (127.0.0.1)
- No analytics, no telemetry, no crash reports
- No data is stored between runs
- Source code is fully auditable (single Python file)

## License

MIT

---

🇹🇷 [Türkçe Kullanım Kılavuzu](README-TR.md)
