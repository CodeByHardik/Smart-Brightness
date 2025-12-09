# Smart Brightness Configuration Guide

This document explains all available configuration options for Smart Brightness, their purposes, default values, and recommended settings.

## Table of Contents

- [Installation](#installation)
- [Daemon Modes](#daemon-modes)
- [Configuration File](#configuration-file)
- [Camera Settings](#camera-settings)
- [Brightness Control](#brightness-control)
- [Smoothing & Response](#smoothing--response)
- [Circadian Rhythm](#circadian-rhythm)
- [Logging & Monitoring](#logging--monitoring)
- [Troubleshooting](#troubleshooting)

## Installation

### Arch Linux

Use the provided `PKGBUILD`.

1.  Build and install:
    ```bash
    makepkg -si
    ```
2.  Enable and start the service:
    ```bash
    sudo systemctl enable --now smart-brightnessd
    ```

### Manual Installation

1.  Build:
    ```bash
    cargo build --release
    ```
2.  Install binary:
    ```bash
    sudo install -Dm755 target/release/smart-brightness /usr/bin/smart-brightness
    ```
3.  Install service:
    ```bash
    sudo install -Dm644 smart-brightnessd.service /usr/lib/systemd/system/smart-brightnessd.service
    ```
4.  Enable service:
    ```bash
    sudo systemctl enable --now smart-brightnessd
    ```

## Configuration File

The daemon automatically loads configuration from:

`~/.config/smart-brightness/config.toml`

Create this file if it does not exist.

## Daemon Modes

Smart Brightness supports three operation modes within the `[daemon]` configuration (or top-level).

### `mode`

-   **Type**: String (`"boot"`, `"interval"`, `"realtime"`)
-   **Default**: `"realtime"`

#### `realtime`
Runs continuously, adjusting brightness in real-time. Best for active use.

#### `interval`
Runs for `run_duration`, then sleeps for `pause_interval`. Useful for saving power.

-   **`run_duration`**: Seconds to run before pausing (default: `300.0` / 5 mins).
-   **`pause_interval`**: Seconds to sleep (default: `60.0` / 1 min).

#### `boot`
Runs once for `run_duration` seconds, then exits. Ideal for setting brightness on startup without keeping a process running.

### Example Daemon Config

```toml
# Select mode: "realtime", "interval", or "boot"
mode = "interval"

# For interval/boot modes
run_duration = 300.0   # Run for 5 minutes
pause_interval = 60.0  # Sleep for 1 minute (interval only)

# Force interval mode on boot even if mode="realtime" (mostly for internal use)
interval_boot = false
```

## Camera Settings

### `camera_index`

- **Type**: Integer
- **Default**: `0`
- **Description**: The index of the camera device to use. `0` is typically the built-in webcam.
- **Note**: If you have multiple cameras, you might need to try different indices.

### `camera_resolution`

- **Type**: Array `[width, height]`
- **Default**: `[640, 400]`
- **Description**: Resolution for camera capture in pixels. Higher values provide more accurate light sensing but use more CPU.
- **Recommended**: `[640, 400]` for most systems. Lower if you experience high CPU usage.

### `camera_warmup_frames`

- **Type**: Integer
- **Default**: `30`
- **Description**: Number of frames to discard while the camera adjusts its auto-exposure and white balance.
- **Recommended**: `30` for most cameras. Increase if the initial brightness is unstable.

## Brightness Control

### `screen_brightness_min`

- **Type**: Integer
- **Default**: `1`
- **Description**: Minimum brightness value (0-100% of your display's range).
- **Note**: Should be at least `1` (displays typically don't like 0).

### `screen_brightness_max`

- **Type**: Integer
- **Description**: Maximum brightness value (check your display's max in `/sys/class/backlight/*/max_brightness`).
- **Example**: `937` for many ThinkPads.

### `ambient_luma_min`

- **Type**: Float (0.0-1.0)
- **Default**: `0.05`
- **Description**: Minimum expected light level (dark room).
- **Note**: Auto-calibrated if not set.

### `ambient_luma_max`

- **Type**: Float (0.0-1.0)
- **Default**: `0.8`
- **Description**: Maximum expected light level (bright daylight).
- **Note**: Auto-calibrated if not set.

## Smoothing & Response

### `ambient_smoothing_strength`

- **Type**: Float (0.0-1.0)
- **Default**: `0.15`
- **Description**: How quickly brightness changes in response to light changes.
  - Lower (e.g., `0.05`): Slower, smoother transitions
  - Higher (e.g., `0.5`): Faster response but may flicker
- **Recommended**: `0.1`-`0.3` for most users.

### `capture_interval_ms`

- **Type**: Integer (milliseconds)
- **Default**: `500`
- **Description**: How often to check ambient light.
- **Recommended**: `200`-`1000` ms. Lower values react faster but use more CPU.

### `brightness_step_interval_ms`

- **Type**: Integer (milliseconds)
- **Default**: `50`
- **Description**: Time between brightness adjustment steps.
- **Recommended**: `20`-`100` ms. Lower = smoother but slower transitions.

### `brightness_step_divisor`

- **Type**: Integer
- **Default**: `20`
- **Description**: Controls how aggressively brightness changes.
  - Higher = smoother but slower transitions
  - Lower = faster but potentially jarring changes
- **Recommended**: `10`-`30` for most displays.

### `brightness_step_max`

- **Type**: Integer
- **Default**: `60`
- **Description**: Maximum brightness change per step.
- **Recommended**: `20`-`100` depending on your display's range.

## Circadian Rhythm

### `circadian_enabled`

- **Type**: Boolean
- **Default**: `true`
- **Description**: Whether to adjust brightness based on time of day.

### `circadian_day_boost`

- **Type**: Float
- **Default**: `1.05`
- **Description**: Brightness multiplier during daytime hours.
- **Recommended**: `1.0`-`1.2` (5-20% brighter during day)

### `circadian_night_dim`

- **Type**: Float
- **Default**: `0.95`
- **Description**: Brightness multiplier during nighttime hours.
- **Recommended**: `0.8`-`1.0` (0-20% dimmer at night)

### `circadian_day_start_hour`

- **Type**: Integer (0-23)
- **Default**: `6` (6 AM)
- **Description**: When daytime brightness begins.

### `circadian_night_start_hour`

- **Type**: Integer (0-23)
- **Default**: `18` (6 PM)
- **Description**: When nighttime brightness begins.

## Logging & Monitoring

### `logging`

- **Type**: String
- **Default**: `"low"`
- **Options**:
  - `"off"`: No logging
  - `"minimal"`: Only critical errors
  - `"low"`: Basic status updates
  - `"medium"`: More detailed info
  - `"high"`: Very verbose
  - `"verbose"`: Maximum detail
- **Recommended**: `"low"` for normal use, `"medium"` for troubleshooting.

### `log_directory`

- **Type**: String (path)
- **Default**: `~/.cache/SMART_BRIGHTNESS/logs`
- **Description**: Where to store log files.

### `status_interval_seconds`

- **Type**: Integer
- **Default**: `5`
- **Description**: How often to log status updates (in seconds).

### `status_min_brightness_change`

- **Type**: Integer
- **Default**: `8`
- **Description**: Minimum brightness change to log.

### `status_fast_interval_seconds`

- **Type**: Float
- **Default**: `0.25`
- **Description**: Faster logging interval during rapid brightness changes.

## Troubleshooting

### Common Issues

1. **Brightness not changing**

   - Check if your user has write permissions to `/sys/class/backlight/*/brightness`.
   - Verify `screen_brightness_max` matches your display's maximum.
   - Try running with `--calibrate` first.

2. **Error: "Camera capture failed"**
   
   - Ensure your user is in the `video` group: `sudo usermod -aG video $USER`.

### Example Configuration

```toml
# Daemon settings
mode = "realtime"

# Camera settings
camera_index = 0
camera_resolution = [640, 400]
camera_warmup_frames = 30

# Brightness range (adjust for your display)
screen_brightness_min = 1
screen_brightness_max = 937

# Smoothing & response
ambient_smoothing_strength = 0.15
capture_interval_ms = 500
brightness_step_interval_ms = 50
brightness_step_divisor = 20
brightness_step_max = 60

# Circadian rhythm
circadian_enabled = true
circadian_day_boost = 1.05
circadian_night_dim = 0.95
circadian_day_start_hour = 6
circadian_night_start_hour = 18

# Logging
logging = "low"
```

For more help, check the [GitHub repository](https://github.com/CodeByHardik/Smart-Brightness) or open an issue.
