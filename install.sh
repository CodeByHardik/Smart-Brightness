#!/bin/bash
set -e

# Smart Brightness Installer
# Enhanced with calibration and mode selection

echo "========================================"
echo "   Smart Brightness Installer"
echo "========================================"
echo

# Check dependencies
check_dependencies() {
    echo "Checking dependencies..."
    
    # Check for cargo
    if ! command -v cargo &> /dev/null; then
        echo "❌ Error: cargo not found. Please install Rust toolchain."
        echo "   Visit: https://rustup.rs/"
        exit 1
    fi
    echo "✓ Cargo found"
    
    # Check for camera
    if [ ! -e /dev/video0 ]; then
        echo "⚠ Warning: No camera detected at /dev/video0"
        echo "  Smart Brightness requires a webcam to function."
        read -p "Continue anyway? (y/n): " continue_choice
        if [[ ! "$continue_choice" =~ ^[Yy]$ ]]; then
            exit 1
        fi
    else
        echo "✓ Camera detected"
    fi
    
    # Check video group membership
    if ! groups | grep -q video; then
        echo "⚠ Warning: Current user is not in 'video' group"
        echo "  This is required for backlight and camera access."
        echo "  Run: sudo usermod -aG video $USER"
        echo "  Then log out and log back in."
        read -p "Continue anyway? (y/n): " continue_choice
        if [[ ! "$continue_choice" =~ ^[Yy]$ ]]; then
            exit 1
        fi
    else
        echo "✓ User in video group"
    fi
    
    echo
}

# Detect OS
if [ -f /etc/arch-release ]; then
    OS="Arch"
else
    OS="Generic"
fi

echo "Detected OS: $OS"
echo

# Check dependencies
check_dependencies

# Installation method selection
if [ "$OS" == "Arch" ]; then
    echo "Choose installation method:"
    echo "1) Install pre-built binary (pacman)"
    echo "2) Compile and install (makepkg)"
    echo "3) Manual install (cargo build + copy)"
    read -p "Selection [1-3]: " method

    case $method in
        1)
            PKG=$(ls *x86_64.pkg.tar.zst 2>/dev/null | head -n 1)
            if [ -z "$PKG" ]; then
                echo "Error: No package found in project directory."
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
            sudo chmod +x /usr/local/bin/smart-brightness
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
    sudo chmod +x /usr/local/bin/smart-brightness
    method="3"
fi

echo
echo "✓ Binary installed successfully!"
echo

# Daemon Mode Selection
echo "=========================================="
echo "   Daemon Mode Configuration"
echo "=========================================="
echo
echo "Select daemon mode:"
echo "1) Realtime  - Continuously adjust brightness (recommended)"
echo "2) Boot      - Run for a duration then exit (good for startup)"
echo "3) Interval  - Run periodically with pauses (battery saving)"
echo
read -p "Selection [1-3] (default: 1): " mode_choice
mode_choice=${mode_choice:-1}

case $mode_choice in
    1)
        DAEMON_MODE="realtime"
        echo "✓ Selected: Realtime mode"
        ;;
    2)
        DAEMON_MODE="boot"
        echo "✓ Selected: Boot mode"
        read -p "Run duration in seconds (default: 300): " run_duration
        run_duration=${run_duration:-300}
        ;;
    3)
        DAEMON_MODE="interval"
        echo "✓ Selected: Interval mode"
        read -p "Active duration in seconds (default: 60): " run_duration
        run_duration=${run_duration:-60}
        read -p "Pause duration in seconds (default: 60): " pause_interval
        pause_interval=${pause_interval:-60}
        ;;
    *)
        echo "Invalid selection, using Realtime mode"
        DAEMON_MODE="realtime"
        ;;
esac

echo

# Config Setup
echo "=========================================="
echo "   Configuration Setup"
echo "=========================================="
echo
echo "Smart Brightness loads config from (in order):"
echo "  1. ~/.config/smart-brightness/config.toml (user config)"
echo "  2. /etc/smart-brightness/config.toml (system config)"
echo "  3. ./config.toml (project directory)"
echo

# Create system config directory
SYSTEM_CONFIG_DIR="/etc/smart-brightness"
if [ ! -d "$SYSTEM_CONFIG_DIR" ]; then
    echo "Creating $SYSTEM_CONFIG_DIR..."
    sudo mkdir -p "$SYSTEM_CONFIG_DIR"
fi

# Create user config directory
USER_CONFIG_DIR="$HOME/.config/smart-brightness"
if [ ! -d "$USER_CONFIG_DIR" ]; then
    echo "Creating $USER_CONFIG_DIR..."
    mkdir -p "$USER_CONFIG_DIR"
fi

# Copy config if it exists
if [ -f "config.toml" ]; then
    # Update mode in config
    if [ -f "config.toml" ]; then
        cp config.toml config.toml.tmp
        sed -i "s/^mode = .*/mode = \"$DAEMON_MODE\"/" config.toml.tmp
        
        if [ "$DAEMON_MODE" == "boot" ] || [ "$DAEMON_MODE" == "interval" ]; then
            sed -i "s/^run_duration = .*/run_duration = $run_duration.0/" config.toml.tmp
        fi
        
        if [ "$DAEMON_MODE" == "interval" ]; then
            sed -i "s/^pause_interval = .*/pause_interval = $pause_interval.0/" config.toml.tmp
        fi
        
        echo "Copying config to $USER_CONFIG_DIR/config.toml..."
        cp config.toml.tmp "$USER_CONFIG_DIR/config.toml"
        
        echo "Copying config to $SYSTEM_CONFIG_DIR/config.toml (fallback)..."
        sudo cp config.toml.tmp "$SYSTEM_CONFIG_DIR/config.toml"
        
        rm config.toml.tmp
        echo "✓ Configuration files created"
    fi
else
    echo "⚠ No local config.toml found. Defaults will be used."
fi

echo

# Calibration Option
echo "=========================================="
echo "   Calibration"
echo "=========================================="
echo
echo "Calibration detects:"
echo "  • Camera sensitivity (ambient light range)"
echo "  • Monitor brightness range (min/max values)"
echo
echo "This ensures accurate brightness adjustment."
echo
read -p "Run calibration now? (y/n): " calibrate_choice

if [[ "$calibrate_choice" =~ ^[Yy]$ ]]; then
    echo
    echo "Starting calibration..."
    echo "Note: This will be saved to $USER_CONFIG_DIR/config.toml"
    echo
    
    # Determine binary path
    if [ "$method" == "3" ] || [ "$OS" != "Arch" ]; then
        BINARY_PATH="/usr/local/bin/smart-brightness"
    else
        BINARY_PATH="/usr/bin/smart-brightness"
    fi
    
    # Run calibration
    if $BINARY_PATH --calibrate; then
        echo
        echo "✓ Calibration completed successfully!"
    else
        echo
        echo "⚠ Calibration failed or was cancelled."
        echo "  You can run it later with: smart-brightness --calibrate"
    fi
else
    echo "Skipping calibration."
    echo "You can run it later with: smart-brightness --calibrate"
fi

echo

# Service Setup
echo "=========================================="
echo "   Systemd Service Setup"
echo "=========================================="
echo

SERVICE_FILE="smart-brightnessd.service"
DEST_SERVICE="/etc/systemd/system/$SERVICE_FILE"

# Adjust service file path if needed
if [ "$method" == "3" ] || [ "$OS" != "Arch" ]; then
    # Manual install to /usr/local/bin
    echo "Adjusting service file for /usr/local/bin..."
    sed 's|/usr/bin/smart-brightness|/usr/local/bin/smart-brightness|g' "$SERVICE_FILE" > "$SERVICE_FILE.tmp"
    sudo cp "$SERVICE_FILE.tmp" "$DEST_SERVICE"
    rm "$SERVICE_FILE.tmp"
else
    sudo cp "$SERVICE_FILE" "$DEST_SERVICE"
fi

sudo systemctl daemon-reload
echo "✓ Service file installed"
echo

read -p "Enable and start service now? (y/n): " start_choice
if [[ "$start_choice" =~ ^[Yy]$ ]]; then
    sudo systemctl enable --now smart-brightnessd
    echo "✓ Service enabled and started!"
    echo
    echo "Check status with: sudo systemctl status smart-brightnessd"
    echo "View logs with: journalctl -u smart-brightnessd -f"
else
    echo "Service installed but not started."
    echo
    echo "To start later:"
    echo "  sudo systemctl enable --now smart-brightnessd"
fi

echo
echo "=========================================="
echo "   Installation Complete!"
echo "=========================================="
echo
echo "Configuration:"
echo "  User config:   $USER_CONFIG_DIR/config.toml"
echo "  System config: $SYSTEM_CONFIG_DIR/config.toml"
echo
echo "Daemon mode: $DAEMON_MODE"
echo
echo "Useful commands:"
echo "  smart-brightness --help       - Show help"
echo "  smart-brightness --calibrate  - Run calibration"
echo "  sudo systemctl status smart-brightnessd - Check service status"
echo "  journalctl -u smart-brightnessd -f      - View logs"
echo
echo "Enjoy Smart Brightness! 🌟"
