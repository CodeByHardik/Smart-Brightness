use std::{fs, thread, time::Duration};
use v4l::prelude::*;
use v4l::video::Capture;
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use image::RgbImage;
use rand::seq::IteratorRandom;

mod calibrate;
mod config;
mod display;

fn main() {
    println!("🚀 Smart Brightness v0.1.0 — Real-Time Adaptive Engine");

    let config_path = "config.toml";

    // Run calibration if missing
    if !std::path::Path::new(config_path).exists() {
        println!("⚡ No config found — starting calibration...");
        calibrate::run_calibration(config_path);
    }

    let cfg = config::Config::load_or_default(config_path);

    // Max brightness
    let max_brightness: u32 = fs::read_to_string(&cfg.max_brightness_path)
        .unwrap()
        .trim()
        .parse()
        .unwrap();

    let min_display_brightness = cfg.min_brightness as f32;
    let max_display_brightness = max_brightness as f32;

    // Camera init
    let device = Device::new(cfg.camera_device).expect("Failed to open camera device");
    let format = device.format().expect("Failed to get format");
    println!(
        "🎥 Camera initialized: {}x{} @ {}",
        format.width,
        format.height,
        format.fourcc.str().unwrap_or("unknown")
    );

    let mut stream =
        MmapStream::with_buffers(&device, Type::VideoCapture, 4).expect("Failed to create stream");

    // Warmup
    println!("⏳ Warming up camera...");
    for _ in 0..cfg.warmup_frames {
        let _ = stream.next().expect("Failed warmup frame");
    }
    println!("✅ Camera warmup complete.");

    // Smooth brightness
    let mut smooth_brightness: f32 = -1.0;

    // Real-time loop
    loop {
        let (data, _) = stream.next().expect("Failed to capture frame");
        let img = RgbImage::from_raw(format.width, format.height, data.to_vec());

        if let Some(img) = img {
            // Sample 100 random pixels for real-time luminance
            let mut rng = rand::thread_rng();
            let pixels = img.pixels().choose_multiple(&mut rng, 100);

            let avg: f32 = pixels
                .iter()
                .map(|p| 0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32)
                .sum::<f32>()
                / pixels.len() as f32;

            // Gamma & clamp
            let gamma_corrected = avg.powf(0.7).clamp(cfg.camera_min_lux, cfg.camera_max_lux);

            let mapped = display::map_value(
                gamma_corrected,
                cfg.camera_min_lux,
                cfg.camera_max_lux,
                min_display_brightness,
                max_display_brightness,
            )
            .clamp(min_display_brightness, max_display_brightness);

            // Initialize or smooth
            if smooth_brightness < 0.0 {
                smooth_brightness = mapped;
            } else {
                smooth_brightness =
                    smooth_brightness * (1.0 - cfg.smoothing_factor) + mapped * cfg.smoothing_factor;
            }

            // Apply brightness
            fs::write(&cfg.backlight_path, smooth_brightness.round().to_string())
                .expect("Failed to set brightness");

            println!(
                "💡 Ambient: {:>6.1} → Target: {:>6.1} → Applied: {:>6.1} / {}",
                avg, mapped, smooth_brightness, max_brightness
            );
        }

        // Short delay for real-time responsiveness
        thread::sleep(Duration::from_millis(50));
    }
}
