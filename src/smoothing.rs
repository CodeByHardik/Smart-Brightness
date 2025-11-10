// src/smoothing.rs
pub struct Ema {
    alpha: f32,
    value: f32,
    init: bool,
}

impl Ema {
    pub fn new(alpha: f32) -> Self {
        Self {
            alpha: alpha.clamp(0.0, 1.0),
            value: 0.0,
            init: false,
        }
    }

    pub fn update(&mut self, x: f32) -> f32 {
        if !self.init {
            self.value = x;
            self.init = true;
        } else {
            self.value = self.alpha * x + (1.0 - self.alpha) * self.value;
        }
        self.value
    }
}