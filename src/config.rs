// src/config.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub camera_device: usize,
    pub resolution: [u32; 2],
    pub warmup_frames: usize,
    pub smoothing_factor: f32,
    pub real_min_brightness: u32,
    pub real_max_brightness: u32,
    pub capture_interval_ms: u64,
    pub smooth_interval_ms: u64,
    pub smooth_step_divisor: u32,
    pub smooth_max_step: u32,
    pub camera_min_luma: Option<f32>,
    pub camera_max_luma: Option<f32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            camera_device: 0,
            resolution: [640, 400],
            warmup_frames: 30,
            smoothing_factor: 0.15,
            real_min_brightness: 47,
            real_max_brightness: 937,
            capture_interval_ms: 500,
            smooth_interval_ms: 50,
            smooth_step_divisor: 20,
            smooth_max_step: 100,
            camera_min_luma: None,
            camera_max_luma: None,
        }
    }
}

pub fn read_config() -> Config {
    let path = Path::new("config.toml");
    if !path.exists() {
        return Config::default();
    }
    let data = fs::read_to_string(path).unwrap_or_default();
    toml::from_str(&data).unwrap_or_default()
}

pub fn autodetect_backlight_file(name: &str) -> Option<PathBuf> {
    let dir = Path::new("/sys/class/backlight");
    if !dir.exists() {
        return None;
    }
    fs::read_dir(dir)
        .ok()?
        .flatten()
        .find_map(|e| {
            let p = e.path().join(name);
            p.exists().then(|| p)
        })
}

pub fn save_config(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let s = toml::to_string_pretty(cfg)?;
    fs::write("config.toml", s)?;
    Ok(())
}