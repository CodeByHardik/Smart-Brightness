# 🌟 Smart Brightness

[![Rust](https://img.shields.io/badge/Made%20with-Rust-dea584.svg?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/CodeByHardik/Smart-Brightness?style=social)](https://github.com/CodeByHardik/Smart-Brightness/stargazers)

A lightweight, real-time automatic screen brightness adjustment tool for Linux that uses your webcam as an ambient light sensor. Inspired by mobile device auto-brightness, but built for your Linux desktop/laptop.

## ✨ Features

- 🌈 Real-time brightness adjustment based on ambient light
- ⚡ Lightweight and resource-efficient with half-precision option
- 🎯 Custom calibration for optimal accuracy
- ⚙️ Configurable via TOML configuration
- 🌙 Built-in circadian rhythm support (optimises brightness based on local time of day)
- 📊 Detailed logging and monitoring
- 🚀 Actively developed; expect frequent improvements

## 🚀 Quick Start

### Prerequisites

- Rust (latest stable)
- Linux system with a webcam
- Backlight control support (`/sys/class/backlight`)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/CodeByHardik/Smart-Brightness.git
   cd Smart-Brightness
   ```

2. **Run the Interactive Installer**
   ```bash
   chmod +x install.sh
   ./install.sh
   ```
   The installer will guide you through:
   - Installing dependencies and permissions
   - Selecting a daemon mode (Realtime, Boot, or Interval)
   - Running initial calibration
   - Setting up the systemd service

## 🛠️ Usage

### Configuration

You can configure Smart Brightness in two ways:

1. **Interactive TUI (Recommended)**
   ```bash
   smart-brightness --configure
   ```
   This opens a visual interface to edit settings, change modes, and adjust sensitivity.

2. **Manual Config Editing**
   Edit `~/.config/smart-brightness/config.toml` directly.

### Calibration
If you notice the brightness range is limited or ambient detection is off:
```bash
smart-brightness --calibrate
```

### Daemon Modes
- **Realtime**: Continuously adjusts brightness. Best for most users.
- **Boot**: Runs for a set duration (e.g. 5 mins) after login, then exits. Good for quick adjustment on startup without background resource usage.
- **Interval**: Runs for a duration, sleeps, then repeats. Good balance of power saving and responsiveness.

### Configuration files are present in the following locations:
```bash
/etc/smart-brightness/config.toml
~/.config/smart-brightness/config.toml
```

### Monitor Brightness

```bash
watch -n 1 cat /sys/class/backlight/*/actual_brightness
```

## 📊 Monitoring

View real-time status:

```bash
journalctl -f -u smart-brightness  # If running as a service
# OR
RUST_LOG=info ./target/release/smart_brightness
```

## 🛣️ Roadmap

### Core Features

- [x] Basic auto-brightness functionality
- [x] Configuration via TOML
- [x] Calibration tool
- [x] Systemd service daemon
- [ ] Systemd daemon fixes and enhancements in the installation procedure
- [ ] Widen the scope of config

### Advanced Features

- [ ] Face detection integration

---

<div align="center">
  Made with ❤️ by <a href="https://github.com/CodeByHardik">CodeByHardik</a>
</div>
