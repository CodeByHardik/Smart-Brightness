use std::io::{self, Write};
use std::time::Duration;

use crate::camera::Camera;
use crate::config::{save_config, Config};

pub fn run(cfg: Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Calibration: This will measure your camera luma range.");
    println!("1) Prepare darkest typical condition (cover lens / dim room), then press Enter.");
    wait_enter()?;

    let (w, h) = (cfg.resolution[0], cfg.resolution[1]);
    let mut cam = Camera::open(cfg.camera_device, w, h)?;
    cam.warmup(cfg.warmup_frames);

    let dark = cam.average_luma_over(60)?;
    println!("Measured dark luma: {:.4}", dark);

    println!("2) Prepare brightest typical condition (point to bright light / daylight), then press Enter.");
    wait_enter()?;

    std::thread::sleep(Duration::from_millis(200));
    let bright = cam.average_luma_over(60)?;
    println!("Measured bright luma: {:.4}", bright);

    let (min_l, max_l) = if dark <= bright { (dark, bright) } else { (bright, dark) };
    if (max_l - min_l) < 0.02 {
        eprintln!("Warning: luma range is very small. Try stronger lighting contrast and re-run.");
    }

    let mut new_cfg = cfg.clone();
    new_cfg.camera_min_luma = Some(min_l);
    new_cfg.camera_max_luma = Some(max_l);

    save_config(&new_cfg)?;
    println!("Saved calibration to config.toml (camera_min_luma, camera_max_luma).");
    Ok(())
}

fn wait_enter() -> io::Result<()> {
    print!("Press Enter to continue...");
    io::stdout().flush()?;
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(())
}
