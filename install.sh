#!/bin/bash
set -e

# Smart Brightness Installer

echo "========================================"
echo "   Smart Brightness Installer"
echo "========================================"

# Detect OS
if [ -f /etc/arch-release ]; then
    OS="Arch"
else
    OS="Generic"
fi

echo "Detected OS: $OS"

if [ "$OS" == "Arch" ]; then
    echo
    echo "Choose installation method:"
    echo "1) Install pre-built binary (pacman)"
    echo "2) Compile and install (makepkg)"
    echo "3) Manual install (cargo build + copy)"
    read -p "Selection [1-3]: " method

    case $method in
        1)
            PKG=$(ls pkg/*x86_64.pkg.tar.zst | head -n 1)
            if [ -z "$PKG" ]; then
                echo "Error: No package found in pkg/ directory."
                exit 1
            fi
            echo "Installing $PKG..."
            sudo pacman -U "$PKG"
            ;;
        2)
            echo "Building with makepkg..."
            makepkg -si
            ;;
        3)
            echo "Building with cargo..."
            cargo build --release
            echo "Installing binary to /usr/local/bin..."
            sudo cp target/release/smart-brightness /usr/local/bin/
            ;;
        *)
            echo "Invalid selection."
            exit 1
            ;;
    esac
else
    echo "Generic installation (Cargo)..."
    cargo build --release
    echo "Installing binary to /usr/local/bin..."
    sudo cp target/release/smart-brightness /usr/local/bin/
fi

# Config Setup
echo
echo "Setting up configuration..."
CONFIG_DIR="/etc/smart-brightness"
if [ ! -d "$CONFIG_DIR" ]; then
    echo "Creating $CONFIG_DIR..."
    sudo mkdir -p "$CONFIG_DIR"
fi

if [ -f "config.toml" ]; then
    echo "Copying local config.toml to $CONFIG_DIR/config.toml..."
    sudo cp config.toml "$CONFIG_DIR/config.toml"
else
    echo "No local config.toml found. Skipping config copy (defaults will be used)."
fi

# Service Setup
echo
echo "Setting up Systemd service..."
SERVICE_FILE="smart-brightnessd.service"
DEST_SERVICE="/etc/systemd/system/$SERVICE_FILE"

# Edit service file to point to installed location if needed?
# The provided service file uses /usr/bin/smart-brightness (standard for pacman/makepkg)
# If we did manual install to /usr/local/bin, we might need to adjust.

if [ "$method" == "3" ] || [ "$OS" != "Arch" ]; then
    # Adjust path for /usr/local/bin
    sed 's|/usr/bin/smart-brightness|/usr/local/bin/smart-brightness|g' "$SERVICE_FILE" > "$SERVICE_FILE.tmp"
    sudo cp "$SERVICE_FILE.tmp" "$DEST_SERVICE"
    rm "$SERVICE_FILE.tmp"
else
    sudo cp "$SERVICE_FILE" "$DEST_SERVICE"
fi

sudo systemctl daemon-reload

echo "Enable and start service now? (y/n)"
read -p "Choice: " start_choice
if [[ "$start_choice" =~ ^[Yy]$ ]]; then
    sudo systemctl enable --now smart-brightnessd
    echo "Service started!"
else
    echo "Service installed but not started."
fi

echo
echo "Installation Complete!"
