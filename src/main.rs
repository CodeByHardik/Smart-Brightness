// src/main.rs
mod backlight;
mod camera;
mod config;
mod smoothing;
mod smooth_transition;
mod calibrate;

use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::thread;
use std::time::{Duration, Instant};

use backlight::Backlight;
use camera::Camera;
use config::read_config;
use smoothing::Ema;
use smooth_transition::SmoothTransition;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = read_config();

    if std::env::args().any(|a| a == "--calibrate") {
        return calibrate::run(cfg);
    }
    let bl = Backlight::resolve(&cfg)?;
    let hardware_max = bl.max_value;

    let real_min = cfg.real_min_brightness;
    let real_max = cfg.real_max_brightness;
    let range_u32 = real_max - real_min;
    let range_f32 = range_u32 as f32;

    println!("Lumen Active: Using brightness range {} → {}", real_min, real_max);

    // Ctrl-C handling
    let running = Arc::new(AtomicBool::new(true));
    {
        let r = running.clone();
        ctrlc::set_handler(move || r.store(false, Ordering::SeqCst))?;
    }

    let (w, h) = (cfg.resolution[0], cfg.resolution[1]);
    let mut cam = Camera::open(cfg.camera_device, w, h)?;
    cam.warmup(cfg.warmup_frames);

    let mut ema = Ema::new(cfg.smoothing_factor);
    let start = bl.current().unwrap_or(real_min).clamp(real_min, real_max);
    let mut transition = SmoothTransition::new(
        start,
        cfg.smooth_interval_ms,
        cfg.smooth_step_divisor,
        cfg.smooth_max_step,
    );

    let capture_interval = Duration::from_millis(cfg.capture_interval_ms);
    let mut last_capture = Instant::now();

    println!("Smart Brightness – running (Ctrl-C to stop)");

    while running.load(Ordering::SeqCst) {
        // 1. Capture new frame at configured rate
        if last_capture.elapsed() >= capture_interval {
            if let Ok(l) = cam.average_luma() {
                let l = if let (Some(min), Some(max)) = (cfg.camera_min_luma, cfg.camera_max_luma) {
                    if max > min { ((l - min) / (max - min)).clamp(0.0, 1.0) } else { l }
                } else { l };
                let smoothed = ema.update(l);
                let mapped = smoothed.mul_add(range_f32, real_min as f32).round() as u32;
                let final_target = mapped.clamp(real_min, real_max);
                transition.set_target(final_target, hardware_max);
            }
            last_capture = Instant::now();
        }

        // 2. Apply smooth step
        if let Some(val) = transition.update() {
            let _ = bl.set(val);
        }

        // 3. Tiny sleep
        thread::sleep(Duration::from_millis(10));
    }

    println!("Smart Brightness – stopped");
    Ok(())
}