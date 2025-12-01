// src/config.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Off,
    Minimal,
    Low,
    #[default]
    Medium,
    High,
    Verbose,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(rename = "camera_index", alias = "camera_device")]
    pub camera_device: usize,
    #[serde(rename = "camera_resolution", alias = "resolution")]
    pub resolution: [u32; 2],
    #[serde(
        rename = "camera_warmup_frames",
        alias = "warmup_frames"
    )]
    pub warmup_frames: usize,
    #[serde(
        rename = "ambient_smoothing_strength",
        alias = "smoothing_factor"
    )]
    pub smoothing_factor: f32,
    #[serde(
        rename = "screen_brightness_min",
        alias = "real_min_brightness"
    )]
    pub real_min_brightness: u32,
    #[serde(
        rename = "screen_brightness_max",
        alias = "real_max_brightness"
    )]
    pub real_max_brightness: u32,
    #[serde(rename = "capture_interval_ms")]
    pub capture_interval_ms: u64,
    #[serde(
        rename = "brightness_step_interval_ms",
        alias = "smooth_interval_ms"
    )]
    pub smooth_interval_ms: u64,
    #[serde(
        rename = "brightness_step_divisor",
        alias = "smooth_step_divisor"
    )]
    pub smooth_step_divisor: u32,
    #[serde(
        rename = "brightness_step_max",
        alias = "smooth_max_step"
    )]
    pub smooth_max_step: u32,
    #[serde(
        rename = "ambient_luma_min",
        alias = "camera_min_luma"
    )]
    pub camera_min_luma: Option<f32>,
    #[serde(
        rename = "ambient_luma_max",
        alias = "camera_max_luma"
    )]
    pub camera_max_luma: Option<f32>,
    #[serde(default)]
    pub calibrated: bool,
    #[serde(default)]
    pub logging: LogLevel,
    #[serde(
        default,
        rename = "log_directory",
        alias = "logging_path"
    )]
    pub logging_path: Option<String>,
    #[serde(
        default = "default_enable_circadian",
        rename = "circadian_enabled",
        alias = "enable_circadian"
    )]
    pub enable_circadian: bool,
    #[serde(
        default = "default_day_multiplier",
        rename = "circadian_day_boost",
        alias = "circadian_day_multiplier"
    )]
    pub circadian_day_multiplier: f32,
    #[serde(
        default = "default_night_multiplier",
        rename = "circadian_night_dim",
        alias = "circadian_night_multiplier"
    )]
    pub circadian_night_multiplier: f32,
    #[serde(default = "default_day_start_hour")]
    pub circadian_day_start_hour: u8,
    #[serde(default = "default_night_start_hour")]
    pub circadian_night_start_hour: u8,
    #[serde(
        default = "default_status_interval_secs",
        rename = "status_interval_seconds",
        alias = "status_interval_secs"
    )]
    pub status_interval_secs: u64,
    #[serde(
        default = "default_status_threshold",
        rename = "status_min_brightness_change",
        alias = "status_threshold"
    )]
    pub status_threshold: u32,
    #[serde(
        default = "default_status_fast_interval_secs",
        rename = "status_fast_interval_seconds",
        alias = "status_fast_interval_secs"
    )]
    pub status_fast_interval_secs: u64,
    #[serde(
        default = "default_status_fast_threshold",
        rename = "status_fast_change_threshold",
        alias = "status_fast_threshold"
    )]
    pub status_fast_threshold: u32,
    #[serde(
        default = "default_error_throttle_secs",
        rename = "error_throttle_seconds",
        alias = "error_throttle_secs"
    )]
    pub error_throttle_secs: u64,
    #[serde(
        default = "default_min_luma_delta",
        rename = "ambient_luma_min_change",
        alias = "min_luma_delta"
    )]
    pub min_luma_delta: f32,
    #[serde(
        default = "default_log_target_brightness",
        rename = "status_show_target_brightness",
        alias = "log_target_brightness"
    )]
    pub log_target_brightness: bool,
    #[serde(
        default = "default_status_log_only_on_change",
        rename = "status_only_when_changed",
        alias = "status_log_only_on_change"
    )]
    pub status_log_only_on_change: bool,
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
            smooth_max_step: 60,
            camera_min_luma: Some(0.05),
            camera_max_luma: Some(0.8),
            calibrated: true,
            logging: LogLevel::Low,
            logging_path: None,
            enable_circadian: default_enable_circadian(),
            circadian_day_multiplier: default_day_multiplier(),
            circadian_night_multiplier: default_night_multiplier(),
            circadian_day_start_hour: default_day_start_hour(),
            circadian_night_start_hour: default_night_start_hour(),
            status_interval_secs: default_status_interval_secs(),
            status_threshold: default_status_threshold(),
            status_fast_interval_secs: default_status_fast_interval_secs(),
            status_fast_threshold: default_status_fast_threshold(),
            error_throttle_secs: default_error_throttle_secs(),
            min_luma_delta: default_min_luma_delta(),
            log_target_brightness: default_log_target_brightness(),
            status_log_only_on_change: default_status_log_only_on_change(),
        }
    }
}

fn default_enable_circadian() -> bool {
    true
}

fn default_day_multiplier() -> f32 {
    1.05
}

fn default_night_multiplier() -> f32 {
    0.95
}

fn default_day_start_hour() -> u8 {
    6
}

fn default_night_start_hour() -> u8 {
    18
}

fn default_status_interval_secs() -> u64 {
    5
}

fn default_status_threshold() -> u32 {
    8
}

fn default_status_fast_interval_secs() -> u64 {
    1
}

fn default_status_fast_threshold() -> u32 {
    40
}

fn default_error_throttle_secs() -> u64 {
    2
}

fn default_min_luma_delta() -> f32 {
    0.01
}

fn default_log_target_brightness() -> bool {
    true
}

fn default_status_log_only_on_change() -> bool {
    true
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.real_max_brightness <= self.real_min_brightness {
            return Err("real_max_brightness must be greater than real_min_brightness".into());
        }
        if self.resolution.iter().any(|&d| d == 0) {
            return Err("resolution entries must be greater than 0".into());
        }
        if self.capture_interval_ms == 0 {
            return Err("capture_interval_ms must be greater than 0".into());
        }
        if self.smooth_interval_ms == 0 {
            return Err("smooth_interval_ms must be greater than 0".into());
        }
        if !(0.0..=1.0).contains(&self.smoothing_factor) {
            return Err("smoothing_factor must be in the range [0, 1]".into());
        }
        if self.smoothing_factor == 0.0 {
            return Err("smoothing_factor cannot be zero".into());
        }
        if self.smooth_step_divisor == 0 {
            return Err("smooth_step_divisor must be greater than 0".into());
        }
        if self.smooth_max_step == 0 {
            return Err("smooth_max_step must be greater than 0".into());
        }
        if self.warmup_frames == 0 {
            return Err("warmup_frames must be greater than 0".into());
        }
        if let (Some(min), Some(max)) = (self.camera_min_luma, self.camera_max_luma) {
            if max <= min {
                return Err("ambient_luma_max must be greater than ambient_luma_min".into());
            }
        }
        if self.status_interval_secs == 0 {
            return Err("status_interval_seconds must be greater than 0".into());
        }
        if self.status_fast_interval_secs == 0 {
            return Err("status_fast_interval_seconds must be greater than 0".into());
        }
        if self.status_threshold == 0 {
            return Err("status_min_brightness_change must be greater than 0".into());
        }
        if self.status_fast_threshold == 0 {
            return Err("status_fast_change_threshold must be greater than 0".into());
        }
        if self.error_throttle_secs == 0 {
            return Err("error_throttle_seconds must be greater than 0".into());
        }
        Ok(())
    }
}

pub fn read_config() -> Config {
    let path = Path::new("config.toml");
    if !path.exists() {
        println!("No config.toml found, using built-in defaults.");
        return Config::default();
    }
    let data = match fs::read_to_string(path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!(
                "Failed to read config.toml ({}). Falling back to defaults.",
                e
            );
            return Config::default();
        }
    };
    match toml::from_str(&data) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!(
                "Failed to parse config.toml ({}). Falling back to defaults.",
                e
            );
            Config::default()
        }
    }
}

pub fn autodetect_backlight_file(name: &str) -> Option<PathBuf> {
    let dir = Path::new("/sys/class/backlight");
    if !dir.exists() {
        return None;
    }
    fs::read_dir(dir).ok()?.flatten().find_map(|e| {
        let p = e.path().join(name);
        p.exists().then(|| p)
    })
}

pub fn save_config(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let s = toml::to_string_pretty(cfg)?;
    fs::write("config.toml", s)?;
    Ok(())
}
