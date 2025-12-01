use chrono::{Local, Timelike};
use crate::config::Config;

/// Applies a simple circadian boost to normalized ambient readings so the display
/// feels brighter during the day and softer at night.
#[derive(Debug, Clone)]
pub struct TimeAdjuster {
    day_multiplier: f32,
    night_multiplier: f32,
    day_start_hour: u8,
    night_start_hour: u8,
}

impl Default for TimeAdjuster {
    fn default() -> Self {
        Self {
            day_multiplier: 1.05,
            night_multiplier: 0.95,
            day_start_hour: 7,
            night_start_hour: 20,
        }
    }
}

impl TimeAdjuster {
    pub fn from_config(cfg: &Config) -> Self {
        Self {
            day_multiplier: cfg.circadian_day_multiplier.max(0.0),
            night_multiplier: cfg.circadian_night_multiplier.max(0.0),
            day_start_hour: cfg.circadian_day_start_hour,
            night_start_hour: cfg.circadian_night_start_hour,
        }
    }

    #[inline]
    fn is_day(&self, hour: u8) -> bool {
        if self.day_start_hour <= self.night_start_hour {
            hour >= self.day_start_hour && hour < self.night_start_hour
        } else {
            hour >= self.day_start_hour || hour < self.night_start_hour
        }
    }

    pub fn factor_now(&self) -> f32 {
        let hour = Local::now().hour() as u8;
        if self.is_day(hour) {
            self.day_multiplier
        } else {
            self.night_multiplier
        }
    }

    #[inline]
    pub fn adjust(&self, normalized_luma: f32) -> f32 {
        (normalized_luma * self.factor_now()).clamp(0.0, 1.0)
    }
}
