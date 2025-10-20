use image::RgbImage;

/// Calculates average luminance (perceived brightness)
pub fn calc_luminance(img: &RgbImage) -> f32 {
    let mut total: f64 = 0.0;
    let mut count: f64 = 0.0;

    // Sample every 10th pixel for speed
    for p in img.pixels().step_by(10) {
        let y = 0.2126 * p[0] as f64 + 0.7152 * p[1] as f64 + 0.0722 * p[2] as f64;
        total += y;
        count += 1.0;
    }

    (total / count) as f32
}

/// Maps a value from one range to another
pub fn map_value(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}
