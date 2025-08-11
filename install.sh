#!/bin/sh

# Resource Cache Installation Script

set -e  # Exit on any error

echo "Installing Resource Cache..."

# Detect operating system and architecture
OS=$(uname -s)
ARCH=$(uname -m)
echo "Detected: $OS on $ARCH"

# Determine binary installation directory based on OS
if [ "$OS" = "Darwin" ]; then
    # macOS
    if [ "$ARCH" = "arm64" ]; then
        BIN_DIR="/opt/homebrew/bin"  # Apple Silicon Homebrew
    else
        BIN_DIR="/usr/local/bin"     # Intel macOS
    fi
    # Fallback if Homebrew path doesn't exist
    if [ ! -d "$BIN_DIR" ]; then
        BIN_DIR="/usr/local/bin"
    fi
elif [ "$OS" = "Linux" ]; then
    # Linux - check multiple common paths
    if [ -d "/opt" ] && [ -w "/opt" ]; then
        BIN_DIR="/opt/resource-cache/bin"
    elif [ -d "/usr/local/bin" ] && [ -w "/usr/local/bin" ]; then
        BIN_DIR="/usr/local/bin"
    elif [ -d "/usr/bin" ] && [ -w "/usr/bin" ]; then
        BIN_DIR="/usr/bin"
    else
        BIN_DIR="$HOME/.local/bin"
    fi
elif [ "$OS" = "FreeBSD" ] || [ "$OS" = "OpenBSD" ] || [ "$OS" = "NetBSD" ]; then
    # BSD systems
    BIN_DIR="/usr/local/bin"
elif [ "$OS" = "SunOS" ] || [ "$OS" = "Solaris" ]; then
    # Solaris/Illumos
    BIN_DIR="/opt/local/bin"
else
    # Unknown Unix-like system
    BIN_DIR="$HOME/.local/bin"
fi

echo "Binary installation directory: $BIN_DIR"

# Check if binary exists
if [ ! -f "./target/resource-cache" ]; then
    echo "Error: Binary not found. Please run ./build.sh first"
    exit 1
fi

# Check if completions exist
if [ ! -f "./target/_resource-cache" ]; then
    echo "Error: Zsh completions not found. Please run ./build.sh first"
    exit 1
fi

# Create binary directory if it doesn't exist
if [ ! -d "$BIN_DIR" ]; then
    echo "Creating binary directory: $BIN_DIR"
    mkdir -p "$BIN_DIR"
fi

# Install the binary
echo "Installing binary to $BIN_DIR..."
if [ -w "$BIN_DIR" ]; then
    install -Dm755 ./target/resource-cache "$BIN_DIR/resource-cache"
else
    echo "Warning: $BIN_DIR not writable, using sudo..."
    sudo install -Dm755 ./target/resource-cache "$BIN_DIR/resource-cache"
fi

# Detect shell and install appropriate completions
echo "Setting up shell completions..."

# Zsh completions - multiple possible locations
if command -v zsh >/dev/null 2>&1; then
    echo "Installing zsh completions..."
    
    # Try multiple zsh completion paths
    ZSH_COMPLETION_DIRS=(
        "$HOME/.zsh/completions"
        "$HOME/.oh-my-zsh/custom/plugins/zsh-completions/src"
        "/usr/local/share/zsh/site-functions"
        "/usr/share/zsh/site-functions"
        "$HOME/.local/share/zsh/site-functions"
    )
    
    ZSH_INSTALLED=false
    for dir in "${ZSH_COMPLETION_DIRS[@]}"; do
        if [ -d "$(dirname "$dir")" ] || [ -w "$(dirname "$dir")" ]; then
            mkdir -p "$dir"
            install -Dm644 ./target/_resource-cache "$dir/"
            echo "  Installed to: $dir"
            ZSH_INSTALLED=true
            break
        fi
    done
    
    # Configure zsh completions
    if [ -f "$HOME/.zshrc" ]; then
        if ! grep -q "$HOME/.zsh/completions" "$HOME/.zshrc" 2>/dev/null; then
            echo "" >> "$HOME/.zshrc"
            echo "# Resource Cache completions" >> "$HOME/.zshrc"
            echo "fpath=($HOME/.zsh/completions \$fpath)" >> "$HOME/.zshrc"
            echo "autoload -U compinit" >> "$HOME/.zshrc"
            echo "compinit" >> "$HOME/.zshrc"
            echo "  Added completions to $HOME/.zshrc"
        else
            echo "  Zsh completions already configured"
        fi
    fi
fi

# Bash completions - multiple possible locations
if [ -f "./target/resource-cache.bash" ]; then
    echo "Installing bash completions..."
    
    BASH_COMPLETION_DIRS=(
        "$HOME/.bash_completion.d"
        "$HOME/.local/share/bash-completion/completions"
        "/usr/local/share/bash-completion/completions"
        "/usr/share/bash-completion/completions"
    )
    
    BASH_INSTALLED=false
    for dir in "${BASH_COMPLETION_DIRS[@]}"; do
        if [ -d "$(dirname "$dir")" ] || [ -w "$(dirname "$dir")" ]; then
            mkdir -p "$dir"
            install -Dm644 ./target/resource-cache.bash "$dir/resource-cache"
            echo "  Installed to: $dir"
            BASH_INSTALLED=true
            break
        fi
    done
    
    # Fallback to ~/.bash_completion if no system directory found
    if [ "$BASH_INSTALLED" = false ]; then
        install -Dm644 ./target/resource-cache.bash "$HOME/.bash_completion"
        echo "  Installed to: $HOME/.bash_completion"
    fi
fi

# Fish completions
if [ -f "./target/resource-cache.fish" ]; then
    echo "Installing fish completions..."
    
    FISH_COMPLETION_DIRS=(
        "$HOME/.config/fish/completions"
        "$HOME/.local/share/fish/vendor_completions.fish"
        "/usr/local/share/fish/vendor_completions.fish"
        "/usr/share/fish/vendor_completions.fish"
    )
    
    for dir in "${FISH_COMPLETION_DIRS[@]}"; do
        if [ -d "$(dirname "$dir")" ] || [ -w "$(dirname "$dir")" ]; then
            mkdir -p "$dir"
            install -Dm644 ./target/resource-cache.fish "$dir/"
            echo "  Installed to: $dir"
            break
        fi
    done
fi

# Add to PATH if not already there
if ! echo "$PATH" | grep -q "$BIN_DIR"; then
    echo ""
    echo "Note: $BIN_DIR is not in your PATH. Add this to your shell config:"
    echo "  export PATH=\"$BIN_DIR:\$PATH\""
    
    # Try to add to common shell configs
    for config in "$HOME/.profile" "$HOME/.bash_profile" "$HOME/.zshrc" "$HOME/.bashrc"; do
        if [ -f "$config" ] && ! grep -q "$BIN_DIR" "$config" 2>/dev/null; then
            echo "" >> "$config"
            echo "# Resource Cache PATH" >> "$config"
            echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$config"
            echo "  Added to: $config"
            break
        fi
    done
fi

echo ""
echo "Installation complete!"
echo "Binary: $BIN_DIR/resource-cache"
echo ""
echo "Completions installed for detected shells:"
if command -v zsh >/dev/null 2>&1; then
    echo "  Zsh: Multiple locations checked"
fi
if [ -f "./target/resource-cache.bash" ]; then
    echo "  Bash: Multiple locations checked"
fi
if [ -f "./target/resource-cache.fish" ]; then
    echo "  Fish: Multiple locations checked"
fi
echo ""
echo "To activate completions, restart your terminal or source your shell config"
echo "Test the installation: resource-cache --help"
