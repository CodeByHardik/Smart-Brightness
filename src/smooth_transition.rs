// src/smooth_transition.rs
use std::time::{Duration, Instant};

pub struct SmoothTransition {
    target: u32,
    current: u32,
    step: u32,
    min_step: u32,
    max_step: u32,
    last: Instant,
    interval: Duration,
    divisor: u32,
}

impl SmoothTransition {
    pub fn new(initial: u32, interval_ms: u64, divisor: u32, max_step: u32) -> Self {
        Self {
            target: initial,
            current: initial,
            step: 1,
            min_step: 1,
            max_step,
            last: Instant::now(),
            interval: Duration::from_millis(interval_ms),
            divisor,
        }
    }

    pub fn set_target(&mut self, t: u32, max_brightness: u32) {
        self.target = t.clamp(0, max_brightness);
        let diff = self.target.abs_diff(self.current);
        self.step = (diff / self.divisor).max(self.min_step).min(self.max_step);
    }

    pub fn update(&mut self) -> Option<u32> {
        if self.current == self.target {
            return None;
        }
        let now = Instant::now();
        if now.duration_since(self.last) < self.interval {
            return None;
        }
        let step = self.step.min(self.target.abs_diff(self.current));
        self.current = if self.current < self.target {
            (self.current + step).min(self.target)
        } else {
            (self.current - step).max(self.target)
        };
        self.last = now;
        Some(self.current)
    }
}