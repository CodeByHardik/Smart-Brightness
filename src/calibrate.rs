use std::io::{self, Write};
use std::fs;
use crate::config::Config;
use toml;

pub fn run_calibration(config_path: &str) {
    println!("🛠️ Starting calibration...");

    let mut config = Config::load_or_default(config_path);

    // --- Camera index ---
    print!("Enter camera index [default {}]: ", config.camera_device);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if !input.trim().is_empty() {
        config.camera_device = input.trim().parse::<usize>().unwrap_or(config.camera_device);
    }

    // --- Backlight path ---
    print!(
        "Enter backlight brightness path [default {}]: ",
        config.backlight_path
    );
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    if !input.trim().is_empty() {
        config.backlight_path = input.trim().to_string();
    }

    // --- Max brightness path ---
    print!(
        "Enter max brightness path [default {}]: ",
        config.max_brightness_path
    );
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    if !input.trim().is_empty() {
        config.max_brightness_path = input.trim().to_string();
    }

    // --- Min brightness ---
    print!("Enter min display brightness [default {}]: ", config.min_brightness);
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    if !input.trim().is_empty() {
        config.min_brightness = input.trim().parse::<u32>().unwrap_or(config.min_brightness);
    }

    // --- Camera lux range ---
    print!(
        "Enter camera min lux [default {}]: ",
        config.camera_min_lux
    );
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    if !input.trim().is_empty() {
        config.camera_min_lux = input.trim().parse::<f32>().unwrap_or(config.camera_min_lux);
    }

    print!(
        "Enter camera max lux [default {}]: ",
        config.camera_max_lux
    );
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    if !input.trim().is_empty() {
        config.camera_max_lux = input.trim().parse::<f32>().unwrap_or(config.camera_max_lux);
    }

    // --- Smoothing factor ---
    print!(
        "Enter smoothing factor (0.0-1.0) [default {}]: ",
        config.smoothing_factor
    );
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    if !input.trim().is_empty() {
        config.smoothing_factor = input.trim().parse::<f32>().unwrap_or(config.smoothing_factor);
    }

    // --- Capture interval ---
    print!(
        "Enter capture interval in milliseconds [default {}]: ",
        config.capture_interval_ms
    );
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    if !input.trim().is_empty() {
        config.capture_interval_ms =
            input.trim().parse::<u64>().unwrap_or(config.capture_interval_ms);
    }

    // --- Resolution (optional) ---
    print!(
        "Enter camera resolution as WIDTHxHEIGHT [default {}x{}]: ",
        config.resolution.0, config.resolution.1
    );
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    if !input.trim().is_empty() {
        if let Some((w, h)) = input.trim().split_once('x') {
            if let (Ok(w), Ok(h)) = (w.parse::<u32>(), h.parse::<u32>()) {
                config.resolution = (w, h);
            }
        }
    }

    // --- Warmup frames ---
    print!(
        "Enter warmup frames (camera stabilization) [default {}]: ",
        config.warmup_frames
    );
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    if !input.trim().is_empty() {
        config.warmup_frames = input.trim().parse::<u32>().unwrap_or(config.warmup_frames);
    }

    // --- Save config ---
    let toml_str = toml::to_string_pretty(&config).unwrap();
    fs::write(config_path, toml_str).expect("Failed to write config.toml");

    println!("✅ Calibration complete. Config saved to {}", config_path);
}
