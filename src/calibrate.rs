use std::io::{self, Write};
use std::time::Duration;

use crate::backlight::Backlight;
use crate::camera::Camera;
use crate::config::{save_config, Config};

pub fn run(mut cfg: Config) -> Result<Config, Box<dyn std::error::Error>> {
    println!("Calibration: This will tune camera sensitivity and monitor limits.");
    println!("1) Prepare darkest typical condition (cover lens / dim room), then press Enter.");
    wait_enter()?;

    let (w, h) = (cfg.resolution[0], cfg.resolution[1]);
    let mut cam = Camera::open(cfg.camera_device, w, h)?;
    cam.warmup(cfg.warmup_frames.max(30));

    let dark = cam.average_luma_over(120)?;
    println!("Measured dark luma: {:.4}", dark);

    println!("2) Prepare brightest typical condition (point to bright light / daylight), then press Enter.");
    wait_enter()?;

    std::thread::sleep(Duration::from_millis(200));
    let bright = cam.average_luma_over(120)?;
    println!("Measured bright luma: {:.4}", bright);

    let (min_l, max_l) = if dark <= bright {
        (dark, bright)
    } else {
        (bright, dark)
    };
    if (max_l - min_l) < 0.02 {
        eprintln!("Warning: luma range is very small. Try stronger lighting contrast and re-run.");
    }

    let (detected_min_brightness, detected_max_brightness) = calibrate_monitor_range(&cfg)?;
    println!(
        "Detected monitor brightness range: {} → {}",
        detected_min_brightness, detected_max_brightness
    );

    cfg.camera_min_luma = Some(min_l);
    cfg.camera_max_luma = Some(max_l);
    cfg.real_min_brightness = detected_min_brightness;
    cfg.real_max_brightness = detected_max_brightness;
    cfg.calibrated = true;

    save_config(&cfg)?;
    println!("Saved calibration to config.toml.");
    Ok(cfg)
}

fn wait_enter() -> io::Result<()> {
    print!("Press Enter to continue...");
    io::stdout().flush()?;
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(())
}

fn calibrate_monitor_range(cfg: &Config) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    println!("3) Monitor calibration using hardware brightness keys.");
    let bl = Backlight::resolve(cfg)?;
    let actual_path = bl
        .actual_path()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| bl.path.clone());
    println!(
        "   (Reading levels from {} – e.g. intel panels expose /sys/class/backlight/intel_backlight/actual_brightness)",
        actual_path.display()
    );

    println!(
        "   • Set the monitor to its MAXIMUM brightness using the hardware keys, then press Enter."
    );
    wait_enter()?;
    let max_level = read_manual_level(&bl)?;
    println!("   → Recorded maximum actual brightness: {}", max_level);

    println!("   • Now set the monitor to the LOWEST brightness that still keeps the screen visible, then press Enter.");
    wait_enter()?;
    let min_level = read_manual_level(&bl)?;
    println!("   → Recorded minimum actual brightness: {}", min_level);

    if max_level <= min_level {
        return Err(
            "Recorded maximum brightness must be greater than minimum; please rerun calibration."
                .into(),
        );
    }

    Ok((min_level, max_level))
}

fn read_manual_level(bl: &Backlight) -> Result<u32, Box<dyn std::error::Error>> {
    std::thread::sleep(Duration::from_millis(150));
    bl.actual()
        .or_else(|| bl.current())
        .ok_or_else(|| "Unable to read actual_brightness from backlight device".into())
}
