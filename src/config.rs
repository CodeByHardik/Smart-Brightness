use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub camera_device: usize,
    pub resolution: (u32, u32),
    pub warmup_frames: u32,
    pub smoothing_factor: f32,
    pub min_brightness: u32,
    pub max_brightness_path: String,
    pub backlight_path: String,
    pub camera_min_lux: f32,
    pub camera_max_lux: f32,
    pub capture_interval_ms: u64,
}

impl Config {
    pub fn load_or_default(path: &str) -> Self {
        if std::path::Path::new(path).exists() {
            let content = std::fs::read_to_string(path).unwrap();
            toml::from_str(&content).unwrap()
        } else {
            Self {
                camera_device: 0,
                resolution: (640, 480),
                warmup_frames: 10,
                smoothing_factor: 0.15,
                min_brightness: 100,
                max_brightness_path: "/sys/class/backlight/intel_backlight/max_brightness".into(),
                backlight_path: "/sys/class/backlight/intel_backlight/brightness".into(),
                camera_min_lux: 20.0,
                camera_max_lux: 200.0,
                capture_interval_ms: 500,
            }
        }
    }
}
