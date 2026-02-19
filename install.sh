#!/usr/bin/env bash
# Ghost CLI Installer (Universal Linux)
# https://github.com/xmrah/ghost
#
# What this script does:
#   Quick Setup:  Checks deps, offers Ollama install, pulls a model, configures PATH
#   Expert Setup: Just symlinks ghost.py → ~/.local/bin/ghost
#   --uninstall:  Removes the symlink and optionally cleans up
#   --dry-run:    Shows what would happen without changing anything

set -euo pipefail

INSTALL_DIR="$HOME/.local/bin"
SCRIPT_NAME="ghost"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_FILE="$SCRIPT_DIR/ghost.py"
DEFAULT_MODEL="gemma2:2b"

# --- Colors ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

info()  { echo -e "${GREEN}✓${NC} $1"; }
warn()  { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; }
step()  { echo -e "\n${CYAN}${BOLD}▸ $1${NC}"; }
ask()   {
    echo -en "${YELLOW}? $1 [y/N]:${NC} "
    read -r response
    [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]
}

header() {
    echo ""
    echo -e "${BOLD}👻 Ghost CLI — Local AI Terminal Assistant${NC}"
    echo "==========================================="
    echo ""
}

# ============================================================
# UNINSTALL
# ============================================================
do_uninstall() {
    header
    echo "Uninstalling Ghost..."
    echo ""

    if [ -L "$INSTALL_DIR/$SCRIPT_NAME" ] || [ -f "$INSTALL_DIR/$SCRIPT_NAME" ]; then
        rm "$INSTALL_DIR/$SCRIPT_NAME"
        info "Removed $INSTALL_DIR/$SCRIPT_NAME"
    else
        warn "Ghost is not installed at $INSTALL_DIR/$SCRIPT_NAME"
    fi

    echo ""
    if command -v ollama &>/dev/null; then
        echo "Ollama is still installed on your system."
        echo "Ghost does not remove Ollama or its models."
        echo "To remove models manually: ollama rm <model-name>"
        echo "To remove Ollama: use your package manager."
    fi

    echo ""
    info "Uninstall complete."
    exit 0
}

# ============================================================
# DRY RUN
# ============================================================
do_dry_run() {
    header
    echo -e "${CYAN}DRY RUN — No changes will be made${NC}"
    echo ""
    echo "Source:  $SOURCE_FILE"
    echo "Target:  $INSTALL_DIR/$SCRIPT_NAME"
    echo ""

    # Python
    if command -v python3 &>/dev/null; then
        info "[DRY] Python 3 found: $(python3 --version 2>&1)"
    else
        error "[DRY] Python 3 NOT found"
    fi

    # Ollama
    if command -v ollama &>/dev/null; then
        info "[DRY] Ollama found: $(command -v ollama)"
    elif curl -sf http://127.0.0.1:11434/api/tags &>/dev/null; then
        info "[DRY] Ollama API responding"
    else
        warn "[DRY] Ollama not detected"
    fi

    # Symlink
    info "[DRY] Would create symlink: $SOURCE_FILE → $INSTALL_DIR/$SCRIPT_NAME"

    # PATH
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) info "[DRY] $INSTALL_DIR is in PATH" ;;
        *) warn "[DRY] $INSTALL_DIR is NOT in PATH — would offer to fix" ;;
    esac

    echo ""
    info "Dry run complete. No changes were made."
    exit 0
}

# ============================================================
# DETECT DISTRO
# ============================================================
detect_distro() {
    DISTRO_ID="unknown"
    if [ -f /etc/os-release ]; then
        DISTRO_ID=$(grep "^ID=" /etc/os-release | cut -d= -f2 | tr -d '"')
    fi
    echo "$DISTRO_ID"
}

# ============================================================
# OLLAMA INSTALL GUIDANCE
# ============================================================
offer_ollama_install() {
    local distro
    distro=$(detect_distro)

    step "Ollama Installation"
    echo "Ghost needs Ollama to run AI models locally."
    echo ""

    case "$distro" in
        nixos)
            echo "  NixOS detected. Add to your configuration.nix:"
            echo -e "    ${CYAN}services.ollama.enable = true;${NC}"
            echo "  Then run: sudo nixos-rebuild switch"
            echo ""
            echo "  Or install temporarily:"
            echo -e "    ${CYAN}nix-env -iA nixpkgs.ollama${NC}"
            ;;
        arch|artix|manjaro|endeavouros)
            echo "  Arch-based distro detected."
            echo -e "    ${CYAN}sudo pacman -S ollama${NC}"
            if [ "$distro" = "artix" ]; then
                echo ""
                echo "  Artix (OpenRC): Start with:"
                echo -e "    ${CYAN}sudo rc-service ollama start${NC}"
            else
                echo ""
                echo "  Start the service:"
                echo -e "    ${CYAN}sudo systemctl enable --now ollama${NC}"
            fi
            ;;
        debian|ubuntu|linuxmint|pop)
            echo "  Debian-based distro detected."
            echo -e "    ${CYAN}curl -fsSL https://ollama.com/install.sh | sh${NC}"
            ;;
        fedora|rhel|centos)
            echo "  Fedora/RHEL detected."
            echo -e "    ${CYAN}curl -fsSL https://ollama.com/install.sh | sh${NC}"
            ;;
        *)
            echo "  Install from: https://ollama.com"
            echo -e "    ${CYAN}curl -fsSL https://ollama.com/install.sh | sh${NC}"
            ;;
    esac

    echo ""
    echo "After installing Ollama, re-run this installer."
    echo ""
}

# ============================================================
# MODEL PULL
# ============================================================
offer_model_pull() {
    step "AI Model Setup"

    # Check if any models exist
    local models_available=false
    if curl -sf http://127.0.0.1:11434/api/tags &>/dev/null; then
        local model_count
        model_count=$(curl -sf http://127.0.0.1:11434/api/tags | python3 -c "import sys,json; print(len(json.load(sys.stdin).get('models',[])))" 2>/dev/null || echo "0")
        if [ "$model_count" -gt 0 ]; then
            info "Found $model_count model(s) installed."
            models_available=true
        fi
    fi

    if [ "$models_available" = true ]; then
        echo "  You already have models. You're good to go!"
        return 0
    fi

    echo "  No AI models found. Ghost needs at least one model."
    echo ""
    echo "  Recommended models:"
    echo "    [1] gemma2:2b    — Fast, small (1.6 GB) — Good for quick commands"
    echo "    [2] gemma3:latest — Balanced (3.2 GB) — Better accuracy"
    echo "    [3] Skip          — I'll pull a model myself later"
    echo ""
    echo -en "${YELLOW}? Choose [1/2/3]:${NC} "
    read -r model_choice

    case "$model_choice" in
        1)
            echo ""
            info "Pulling gemma2:2b... (this may take a few minutes)"
            ollama pull gemma2:2b
            info "Model ready!"
            ;;
        2)
            echo ""
            info "Pulling gemma3:latest... (this may take a few minutes)"
            ollama pull gemma3:latest
            info "Model ready!"
            ;;
        *)
            echo ""
            warn "Skipped. Pull a model later with: ollama pull gemma2:2b"
            ;;
    esac
}

# ============================================================
# PATH CONFIGURATION
# ============================================================
configure_path() {
    case ":$PATH:" in
        *":$INSTALL_DIR:"*)
            info "$INSTALL_DIR is already in your PATH."
            return 0
            ;;
    esac

    step "PATH Configuration"
    warn "$INSTALL_DIR is not in your PATH."
    echo "  Without this, you'll need to type the full path to run ghost."
    echo ""

    if ! ask "Add $INSTALL_DIR to your PATH automatically?"; then
        warn "Skipped. Add it manually to your shell config."
        return 0
    fi

    # Detect shell and config file
    local shell_name config_file path_line
    shell_name=$(basename "${SHELL:-bash}")

    case "$shell_name" in
        fish)
            config_file="$HOME/.config/fish/config.fish"
            path_line="fish_add_path $INSTALL_DIR"
            ;;
        zsh)
            config_file="$HOME/.zshrc"
            path_line="export PATH=\"\$HOME/.local/bin:\$PATH\""
            ;;
        *)
            config_file="$HOME/.bashrc"
            path_line="export PATH=\"\$HOME/.local/bin:\$PATH\""
            ;;
    esac

    # Check if already present
    if [ -f "$config_file" ] && grep -qF "$INSTALL_DIR" "$config_file" 2>/dev/null; then
        info "PATH entry already exists in $config_file"
        return 0
    fi

    echo "$path_line" >> "$config_file"
    info "Added to $config_file"
    echo "  Restart your terminal or run: source $config_file"
}

# ============================================================
# INSTALL GHOST (symlink)
# ============================================================
install_ghost() {
    step "Installing Ghost"

    mkdir -p "$INSTALL_DIR"

    if [ -L "$INSTALL_DIR/$SCRIPT_NAME" ] || [ -f "$INSTALL_DIR/$SCRIPT_NAME" ]; then
        rm "$INSTALL_DIR/$SCRIPT_NAME"
        info "Removed previous installation."
    fi

    ln -s "$SOURCE_FILE" "$INSTALL_DIR/$SCRIPT_NAME"
    chmod +x "$SOURCE_FILE"
    info "Installed: $INSTALL_DIR/$SCRIPT_NAME → $SOURCE_FILE"
}

# ============================================================
# QUICK SETUP (Beginners)
# ============================================================
quick_setup() {
    step "Checking Python 3"
    if command -v python3 &>/dev/null; then
        info "Python 3 found: $(python3 --version 2>&1)"
    else
        error "Python 3 is required but not found."
        error "Install it with your package manager and re-run."
        exit 1
    fi

    step "Checking Ollama"
    local ollama_running=false

    if command -v ollama &>/dev/null; then
        info "Ollama binary found."
        if curl -sf http://127.0.0.1:11434/api/tags &>/dev/null; then
            info "Ollama API is responding."
            ollama_running=true
        else
            warn "Ollama is installed but not running."
            local distro
            distro=$(detect_distro)
            echo ""
            case "$distro" in
                artix)
                    echo "  Start with: sudo rc-service ollama start" ;;
                nixos)
                    echo "  Start with: sudo systemctl start ollama" ;;
                *)
                    echo "  Start with: ollama serve &" ;;
            esac
            echo ""
            if ask "Continue anyway? (You can start Ollama later)"; then
                ollama_running=false
            else
                exit 1
            fi
        fi
    else
        warn "Ollama is not installed."
        offer_ollama_install

        if ask "Continue without Ollama? (Install it later before using Ghost)"; then
            ollama_running=false
        else
            exit 1
        fi
    fi

    install_ghost

    if [ "$ollama_running" = true ]; then
        offer_model_pull
    fi

    configure_path
}

# ============================================================
# EXPERT SETUP
# ============================================================
expert_setup() {
    step "Expert Mode"
    echo "  Skipping all checks. Installing symlink only."
    install_ghost
    info "Done. You know what you're doing. 🤘"
}

# ============================================================
# MAIN
# ============================================================

# Handle flags
case "${1:-}" in
    --uninstall) do_uninstall ;;
    --dry-run)   do_dry_run ;;
    --help|-h)
        echo "Usage: ./install.sh [OPTIONS]"
        echo ""
        echo "Options:"
        echo "  (none)       Interactive installer"
        echo "  --dry-run    Show what would happen"
        echo "  --uninstall  Remove Ghost"
        echo "  --help       Show this help"
        exit 0
        ;;
esac

# Interactive mode
header

echo "How would you like to install Ghost?"
echo ""
echo -e "  ${GREEN}[1]${NC} Quick Setup  — Guided installation (recommended for new users)"
echo -e "  ${CYAN}[2]${NC} Expert Setup — Just the symlink, I'll handle the rest"
echo ""
echo -en "${YELLOW}? Choose [1/2]:${NC} "
read -r setup_mode

case "$setup_mode" in
    2)  expert_setup ;;
    *)  quick_setup ;;
esac

echo ""
echo "==========================================="
info "Ghost is ready! Try: ghost \"update my system\""
echo "==========================================="
echo ""
