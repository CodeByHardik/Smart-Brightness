// src/main.rs
mod backlight;
mod calibrate;
mod camera;
mod config;
mod logging;
mod smooth_transition;
mod smoothing;
mod time_adjust;
mod tui;

use std::io;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

use backlight::Backlight;
use camera::Camera;
use config::{read_config, Config, DaemonMode, LogLevel};
use logging::Logger;
use smooth_transition::SmoothTransition;
use smoothing::Ema;
use time_adjust::TimeAdjuster;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for help flag
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    let mut cfg = read_config();

    // Check for configure flag
    if std::env::args().any(|a| a == "--configure") {
        tui::run(cfg)?;
        return Ok(());
    }

    let logger = Logger::new(cfg.logging, cfg.logging_path.as_deref());
    let calibrate_requested = std::env::args().any(|a| a == "--calibrate");

    if calibrate_requested {
        logger.info(|| "Calibration requested via --calibrate".into());
        calibrate::run(cfg)?;
        logger.info(|| "Calibration completed.".into());
        return Ok(());
    }

    if !cfg.calibrated {
        logger.info(|| "No calibration found. Running automatic first-time calibration…".into());
        cfg = calibrate::run(cfg)?;
        logger.info(|| "Initial calibration completed.".into());
    }

    if let Err(e) = cfg.validate() {
        let msg = format!("invalid config: {}", e);
        logger.error(msg.clone());
        return Err(io::Error::new(io::ErrorKind::InvalidData, msg).into());
    }

    // Handle interval_boot override
    // If enabled, we treat the current run as 'Interval' regardless of config.mode (unless overridden)
    // Actually, usually this means "on boot, if we are in boot mode, forces interval".
    // User request: "If interval_boot = true, force interval mode on boot."
    // This implies that if the process is started at boot (how do we know? Systemd doesn't tell us easily unless we check uptime or args),
    // we should use interval mode.
    // However, usually "on boot" just means "when the daemon starts".
    // So if `interval_boot` is true, we override `cfg.mode` to `DaemonMode::Interval`.
    if cfg.interval_boot {
        logger.info(|| "interval_boot is true: Forcing Interval mode.".into());
        cfg.mode = DaemonMode::Interval;
    }

    logger.info(|| format!("Starting Smart Brightness in {:?} mode", cfg.mode));

    // Ctrl-C handling
    let running = Arc::new(AtomicBool::new(true));
    {
        let r = running.clone();
        ctrlc::set_handler(move || r.store(false, Ordering::SeqCst))?;
    }

    match cfg.mode {
        DaemonMode::Realtime => {
            run_brightness_loop(&cfg, &logger, running, None)?;
        }
        DaemonMode::Boot => {
            let duration = Duration::from_secs_f64(cfg.run_duration);
            logger.info(|| format!("Running for {:.1} seconds...", cfg.run_duration));
            run_brightness_loop(&cfg, &logger, running, Some(duration))?;
        }
        DaemonMode::Interval => {
            let run_duration = Duration::from_secs_f64(cfg.run_duration);
            let pause_interval = Duration::from_secs_f64(cfg.pause_interval);

            while running.load(Ordering::SeqCst) {
                logger.info(|| "Interval: Active phase started".into());
                // We need a fresh 'running' signal for the inner loop if we want to support clean shutdown,
                // but the inner loop checks 'running' anyway.
                // However, the inner loop returns when duration expires.
                // We should pass the same 'running' flag so Ctrl-C breaks the inner loop immediately.
                
                run_brightness_loop(&cfg, &logger, running.clone(), Some(run_duration))?;

                if !running.load(Ordering::SeqCst) {
                   break;
                }

                logger.info(|| format!("Interval: Sleeping for {:.1} seconds...", cfg.pause_interval));
                
                // Sleep with check for interrupt
                let sleep_start = Instant::now();
                while sleep_start.elapsed() < pause_interval {
                     if !running.load(Ordering::SeqCst) {
                         break;
                     }
                     thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    logger.info(|| "Smart Brightness – stopped".into());
    Ok(())
}

fn run_brightness_loop(
    cfg: &Config,
    logger: &Logger,
    running: Arc<AtomicBool>,
    max_duration: Option<Duration>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    let bl = Backlight::resolve(cfg)?;
    let hardware_max = bl.max_value;
    let hardware_min = bl.min_value();

    let real_min = cfg.real_min_brightness;
    let real_max = cfg.real_max_brightness;
    let range_u32 = real_max - real_min;
    let range_f32 = range_u32 as f32;

    logger.info(|| {
        format!(
            "Hardware brightness range: {} → {} (max possible)",
            hardware_min, hardware_max
        )
    });
    logger.info(|| {
        format!(
            "Configured brightness range: {} → {} (from calibration)",
            real_min, real_max
        )
    });
    
    // Warn if configured range seems limited
    if real_min > hardware_min + 10 {
        logger.warn(|| {
            format!(
                "⚠ Configured minimum ({}) is significantly above hardware minimum ({}). \
                 Run calibration to use full range.",
                real_min, hardware_min
            )
        });
    }
    if real_max < hardware_max - 10 {
        logger.info(|| {
            format!(
                "ℹ Configured maximum ({}) is below hardware maximum ({}). \
                 This is normal if set during calibration.",
                real_max, hardware_max
            )
        });
    }

    logger.info(|| {
        format!(
            "Config: smoothing={:.3}, circadian_enabled={}, min_luma_delta={:.3}, status_interval={}s, fast_interval={:.2}s",
            cfg.smoothing_factor,
            cfg.enable_circadian,
            cfg.min_luma_delta,
            cfg.status_interval_secs,
            cfg.status_fast_interval_secs,
        )
    });

    let (w, h) = (cfg.resolution[0], cfg.resolution[1]);
    let mut cam = Camera::open(cfg.camera_device, w, h)?;
    cam.warmup(cfg.warmup_frames);

    let mut ema = Ema::new(cfg.smoothing_factor);
    let start_val = bl
        .actual()
        .or_else(|| bl.current())
        .unwrap_or(real_min)
        .clamp(real_min, real_max);
    let mut transition = SmoothTransition::new(
        start_val,
        cfg.smooth_interval_ms,
        cfg.smooth_step_divisor,
        cfg.smooth_max_step,
    );
    let mut status = StatusReporter::new(
        start_val,
        logger.clone(),
        cfg.status_interval_secs,
        cfg.status_threshold,
        cfg.status_fast_interval_secs,
        cfg.status_fast_threshold,
        cfg.log_target_brightness,
        cfg.status_log_only_on_change,
    );
    let circadian = TimeAdjuster::from_config(cfg);

    let capture_interval = Duration::from_millis(cfg.capture_interval_ms);
    let mut last_capture = Instant::now() - capture_interval;
    let mut capture_errors = ErrorThrottle::new(
        Duration::from_secs(cfg.error_throttle_secs),
        logger.clone(),
        LogLevel::Minimal,
    );

    let mut last_adjusted_luma = 0.0f32;
    let mut has_luma = false;

    while running.load(Ordering::SeqCst) {
        // Check duration
        if let Some(limit) = max_duration {
            if start_time.elapsed() >= limit {
                logger.info(|| "Run duration expired.".into());
                break;
            }
        }

        let mut work_done = false;

        // 1. Capture new frame at configured rate
        if last_capture.elapsed() >= capture_interval {
            match cam.measure_luma(cfg.half_precision) {
                Ok(raw_luma) => {
                    let normalized = normalize_luma(cfg, raw_luma);
                    let smoothed = ema.update(normalized);
                    let adjusted = apply_circadian(cfg, &circadian, smoothed);
                    if let Some(target) = update_brightness(
                        adjusted,
                        &mut has_luma,
                        &mut last_adjusted_luma,
                        cfg.min_luma_delta,
                        range_f32,
                        real_min,
                        real_max,
                        hardware_max,
                    ) {
                        transition.set_target(target, hardware_max);
                    }
                }
                Err(err) => {
                    capture_errors.log("Camera capture failed", err);
                }
            }
            last_capture = Instant::now();
            work_done = true;
        }

        // Always update status, regardless of capture interval
        status.record(transition.current_value(), last_adjusted_luma);

        // 2. Apply smooth step
        if let Some(val) = transition.update() {
            let _ = bl.set(val);
            work_done = true;
        }

        // 3. Sleep just enough to wait for the next due event
        if !work_done {
            let since_capture = last_capture.elapsed();
            let capture_wait = if since_capture >= capture_interval {
                Duration::from_millis(0)
            } else {
                capture_interval - since_capture
            };
            let smooth_wait = transition.time_until_next_step();
            let sleep_for = capture_wait.min(smooth_wait).min(Duration::from_millis(10));
            if sleep_for.is_zero() {
                std::thread::yield_now();
            } else {
                thread::sleep(sleep_for);
            }
        }
    }
    
    // Safety check: ensure we didn't crash
    Ok(())
}

struct StatusReporter {
    last_value: u32,
    last_luma: f32,
    last_print: Instant,
    base_interval: Duration,
    base_threshold: u32,
    fast_interval: Duration,
    fast_threshold: u32,
    logger: Logger,
    level: LogLevel,
    enabled: bool,
    only_on_change: bool,
}

impl StatusReporter {
    fn new(
        initial: u32,
        logger: Logger,
        interval_secs: u64,
        threshold: u32,
        fast_interval_secs: f64,
        fast_threshold: u32,
        enabled: bool,
        only_on_change: bool,
    ) -> Self {
        let base_interval = Duration::from_secs(interval_secs.max(1));
        Self {
            last_value: initial,
            last_luma: 0.0,
            // Initialize as if the last print was one full interval ago so that
            // the first significant brightness change can be logged promptly.
            last_print: Instant::now() - base_interval,
            base_interval,
            base_threshold: threshold.max(1),
            fast_interval: Duration::from_secs_f64(fast_interval_secs),
            fast_threshold: fast_threshold.max(1),
            logger,
            level: LogLevel::Low,
            enabled,
            only_on_change,
        }
    }

    fn record(&mut self, brightness: u32, normalized_luma: f32) {
        if !self.enabled {
            self.last_value = brightness;
            self.last_luma = normalized_luma;
            return;
        }
        let now = Instant::now();
        let delta = brightness.abs_diff(self.last_value);
        let interval = if delta >= self.fast_threshold {
            self.fast_interval
        } else {
            self.base_interval
        };
        let changed = delta >= self.base_threshold;
        let expired = now.duration_since(self.last_print) >= interval;
        let should_log = if self.only_on_change {
            changed && expired
        } else {
            changed || expired
        };
        if should_log {
            if self.logger.enabled(self.level) {
                let value = brightness;
                let luma = normalized_luma;
                self.logger
                    .status(|| format!("→ Target brightness {} (normalized {:.3})", value, luma));
            }
            self.last_value = brightness;
            self.last_luma = normalized_luma;
            self.last_print = now;
        } else {
            self.last_luma = normalized_luma;
        }
    }
}

struct ErrorThrottle {
    last_log: Option<Instant>,
    interval: Duration,
    logger: Logger,
    level: LogLevel,
}

impl ErrorThrottle {
    fn new(interval: Duration, logger: Logger, level: LogLevel) -> Self {
        Self {
            last_log: None,
            interval,
            logger,
            level,
        }
    }

    fn log<E: std::fmt::Display>(&mut self, context: &str, err: E) {
        let should_log = self
            .last_log
            .map(|t| t.elapsed() >= self.interval)
            .unwrap_or(true);
        if should_log && self.logger.enabled(self.level) {
            let msg = format!("{}: {}", context, err);
            self.logger.warn(|| msg);
            self.last_log = Some(Instant::now());
        }
    }
}

fn normalize_luma(cfg: &config::Config, raw: f32) -> f32 {
    if let (Some(min), Some(max)) = (cfg.camera_min_luma, cfg.camera_max_luma) {
        if max > min {
            return ((raw - min) / (max - min)).clamp(0.0, 1.0);
        }
    }
    raw
}

fn apply_circadian(cfg: &config::Config, circadian: &TimeAdjuster, smoothed: f32) -> f32 {
    if cfg.enable_circadian {
        circadian.adjust(smoothed)
    } else {
        smoothed
    }
}

fn update_brightness(
    adjusted: f32,
    has_luma: &mut bool,
    last_adjusted_luma: &mut f32,
    min_luma_delta: f32,
    range_f32: f32,
    real_min: u32,
    real_max: u32,
    hardware_max: u32,
) -> Option<u32> {
    let luma_delta = if *has_luma {
        (adjusted - *last_adjusted_luma).abs()
    } else {
        f32::MAX
    };
    if *has_luma && luma_delta < min_luma_delta {
        *last_adjusted_luma = adjusted;
        return None;
    }
    *has_luma = true;
    *last_adjusted_luma = adjusted;
    let mapped = adjusted.mul_add(range_f32, real_min as f32).round() as u32;
    let final_target = mapped.clamp(real_min, real_max).min(hardware_max);
    Some(final_target)
}

fn print_help() {
    println!("Smart Brightness - Automatic screen brightness adjustment");
    println!();
    println!("USAGE:");
    println!("    smart_brightness [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --configure     Launch TUI configuration interface");
    println!("    --calibrate     Run calibration wizard to detect camera sensitivity");
    println!("                    and monitor brightness range");
    println!("    -h, --help      Display this help message");
    println!();
    println!("CONFIGURATION:");
    println!("    Config files are loaded from (in order):");
    println!("      1. ~/.config/smart-brightness/config.toml");
    println!("      2. /etc/smart-brightness/config.toml");
    println!("      3. ./config.toml (current directory)");
    println!();
    println!("DAEMON MODES:");
    println!("    realtime    - Continuously adjust brightness (default)");
    println!("    boot        - Run for specified duration then exit");
    println!("    interval    - Run for duration, pause, then repeat");
    println!();
    println!("EXAMPLES:");
    println!("    # Run calibration");
    println!("    smart_brightness --calibrate");
    println!();
    println!("    # Run in realtime mode (reads from config)");
    println!("    smart_brightness");
    println!();
    println!("For more information, visit:");
    println!("    https://github.com/CodeByHardik/Smart-Brightness");
}
