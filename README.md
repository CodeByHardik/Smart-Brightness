# 🌟 Smart Brightness

[![Rust](https://img.shields.io/badge/Made%20with-Rust-dea584.svg?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/CodeByHardik/Smart-Brightness?style=social)](https://github.com/CodeByHardik/Smart-Brightness/stargazers)

A lightweight, real-time automatic screen brightness adjustment tool for Linux that uses your webcam as an ambient light sensor. Inspired by mobile device auto-brightness, but built for your Linux desktop/laptop.

## ✨ Features

- 🌈 Real-time brightness adjustment based on ambient light
- ⚡ Lightweight and resource-efficient
- 🎯 Custom calibration for optimal accuracy
- ⚙️ Configurable via TOML configuration
- 🌙 Built-in circadian rhythm support (optimizes brightness based on local time of day)
- 📊 Detailed logging and monitoring
- 🚀 Actively developed; expect frequent improvements

## 🚀 Quick Start

### Prerequisites

- Rust (latest stable)
- Linux system with a webcam
- Backlight control support

### Installation

1. **Clone the repository**

   ```bash
   git clone https://github.com/CodeByHardik/Smart-Brightness.git
   cd Smart-Brightness
   ```

2. **Build the project**

   ```bash
   cargo build --release
   ```

3. **Set up permissions** (one-time setup)
   ```bash
   sudo tee /etc/udev/rules.d/99-backlight.rules <<EOF
   ACTION=="add", SUBSYSTEM=="backlight", \
       RUN+="/bin/chmod g+w /sys/class/backlight/%k/brightness"
   ACTION=="add", SUBSYSTEM=="backlight", \
       RUN+="/bin/chgrp video /sys/class/backlight/%k/brightness"
   EOF
   sudo udevadm control --reload
   sudo udevadm trigger
   ```
   > Make sure your user is in the `video` group: `sudo usermod -aG video $USER`

## 🛠️ Usage

### Basic Usage

```bash
# Calibrate for your environment
./target/release/smart_brightness --calibrate

# Run with default settings
./target/release/smart_brightness

```

### Monitor Brightness

```bash
watch -n 1 cat /sys/class/backlight/*/actual_brightness
```

## ⚙️ Configuration

Copy `sample-config.toml` to `config.toml` and customize:

## 📊 Monitoring

View real-time status:

```bash
journalctl -f -u smart-brightness  # If running as service
# OR
RUST_LOG=info ./target/release/smart_brightness
```

## 🛣️ Roadmap

### Core Features

- [x] Basic auto-brightness functionality
- [x] Configuration via TOML
- [x] Calibration tool
- [ ] Systemd service templates

### Advanced Features

- [ ] Face detection integration

---

<div align="center">
  Made with ❤️ by <a href="https://github.com/CodeByHardik">Hardik</a>
</div>
