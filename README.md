Smart-Brightness

Manually changing your screen brightness is a hassle — why shouldn’t it adapt automatically?
Smart-Brightness provides an adaptive brightness controller for Linux laptops.

Features
- Automatic brightness adjustment based on ambient light using your webcam
- Smooth transitions to avoid sudden flickers
- Easy calibration on first run
- Configurable via config.toml

Dependencies
- Latest Rust (tested on Rust 1.90+)

Requirements
- A functional webcam

Tested On
- Dell Latitude 3450 (2013)
- Arch Linux Rolling
- Linux-zen 6.17.3-zen2-1-zen kernel
- Integrated webcam

Installation
1. Clone the repository:
   git clone https://github.com/CodeByHardik/Smart-Brightness.git
   cd Smart-Brightness

2. Build with Cargo:
   cargo build --release

3. Run the program:
   sudo ./target/release/smart_brightness
   (sudo is required to write to backlight paths)

TODO
- [x] Calibration logic
- [ ] Real-time brightness adjustment
- [ ] Smooth brightness transitions
- [ ] Systemd daemon integration for auto-start
- [ ] Advanced configuration options
- [ ] Integrate face detection models
- [ ] Adjust brightness based on the user’s viewing direction
