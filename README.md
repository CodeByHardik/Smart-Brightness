# Smart Brightness

Smart Brightness introduces automatic screen brightness adjustment for Linux laptops and desktops using your webcam as a light sensor.

### Why Smart Brightness?

Most mobile devices and operating systems, like Android, feature automatic brightness control. However, mainstream Linux distributions lack this capability for desktops and laptops. Smart Brightness fills this gap with a lightweight, real-time utility designed for simplicity and flexibility.

---

## Features

- Real-time brightness adjustment based on ambient light
- Simple setup and usage
- Lightweight and resource-friendly
- Custom calibration for better accuracy
- Actively developed; expect frequent changes

---

## Installation

### 1. Clone the Repository

git clone https://github.com/CodeByHardik/Smart-Brightness.git
cd Smart-Brightness

### 2. Build the Project

cargo build --release

### 3. Enable Non-Root Brightness Control

sudo tee /etc/udev/rules.d/99-backlight.rules <<EOF
ACTION=="add", SUBSYSTEM=="backlight", RUN+="/bin/chmod g+w /sys/class/backlight/%k/brightness"
ACTION=="add", SUBSYSTEM=="backlight", RUN+="/bin/chgrp video /sys/class/backlight/%k/brightness"
EOF
sudo udevadm control --reload
sudo udevadm trigger

### 4. (Optional) Calibrate Your Camera

./target/release/smart_brightness --calibrate

### 5. Run Smart Brightness

./target/release/smart_brightness

Press Ctrl+C to stop.

---

## Testing

To monitor brightness changes:

watch -n 1 cat /sys/class/backlight/intel_backlight/actual_brightness

Vary ambient lighting in front of your camera and observe terminal updates. Use Ctrl+C to exit.

---

## Roadmap

- [x] Initial README and basic structure
- [x] Improved accuracy and bug fixes
- [ ] Options for launching at boot, login, or lock screen
- [ ] Advanced features for higher accuracy and performance
- [ ] Lightweight face detection integration
