// src/camera.rs
use std::error::Error;

use v4l::buffer::Type;
use v4l::device::Device;
use v4l::format::FourCC;
use v4l::io::traits::CaptureStream;
use v4l::prelude::MmapStream;
use v4l::video::Capture;

pub struct Camera {
    _dev: Device,
    stream: MmapStream<'static>,
    width: u32,
    height: u32,
}

impl Camera {
    pub fn open(idx: usize, w: u32, h: u32) -> Result<Self, Box<dyn Error>> {
        let mut dev = Device::new(idx)?;
        let mut fmt = dev.format()?;
        fmt.width = w;
        fmt.height = h;
        fmt.fourcc = FourCC::new(b"YUYV");
        dev.set_format(&fmt)?;
        let stream = MmapStream::with_buffers(&mut dev, Type::VideoCapture, 4)?;
        Ok(Self {
            _dev: dev,
            stream,
            width: w,
            height: h,
        })
    }

    pub fn warmup(&mut self, frames: usize) {
        eprintln!("Warming up camera…");
        for _ in 0..frames {
            let _ = self.stream.next();
        }
        eprintln!("Camera ready.");
    }

    pub fn measure_luma(&mut self, half_precision: bool) -> Result<f32, Box<dyn Error>> {
        let (buf, _) = self.stream.next()?;
        let mut sum: f32 = 0.0;
        let mut weight_sum: f32 = 0.0;

        let w = self.width as usize;
        let h = self.height as usize;
        let cx = w / 2;
        let cy = h / 2;
        let max_dist_sq = ((cx * cx + cy * cy) as f32).max(1.0);

        // YUYV format: 4 bytes = 2 pixels.
        // Byte 0: Y0, Byte 1: U, Byte 2: Y1, Byte 3: V
        // We iterate 2 bytes at a time to get each Y.
        // Stride: If half_precision, step by 4 (skip every other Y).
        // Y values are at index 0, 2, 4, 6...
        
        let step = if half_precision { 4 } else { 2 };
        
        // We need to track pixel coordinates for center weighting.
        // Each step advances 1 pixel (if step=2) or 2 pixels (if step=4) but wait...
        // chunks_exact(2) gave us pairs.
        // Let's iterate raw buffer bytes.
        
        for (i, chunk) in buf.chunks(step).enumerate() {
            if chunk.is_empty() { break; }
            let y = chunk[0] as f32; // Y component is always at optional offset 0 of the block if we align right.
            // Wait, YUYV = Y0 U0 Y1 V0
            // idx 0 -> Y0
            // idx 2 -> Y1
            // idx 4 -> Y2
            // If we step by 2, we get Y0, Y1, Y2...
            // If we step by 4, we get Y0, Y2, Y4... (Skipping Y1, Y3) -> This is half precision.
            
            // To calculate weight, we need (x, y) coords.
            // Pixel index = i * (step / 2) -> because each Y is 2 bytes in memory (effectively)
            // No, Y is 1 byte, but shared UV makes it "2 bytes per pixel" on average, but positionally:
            // Byte 0 is Px0, Byte 2 is Px1.
            
            let pixel_idx = if half_precision { i * 2 } else { i };
            if pixel_idx >= w * h { break; }
            
            let px = pixel_idx % w;
            let py = pixel_idx / w;
            
            // Simple center weight: 1.0 at center, falling off to 0.2 at edges
            let dx = (px as isize - cx as isize) as f32;
            let dy = (py as isize - cy as isize) as f32;
            let dist_sq = dx*dx + dy*dy;
            let weight = 1.0 - 0.8 * (dist_sq / max_dist_sq).min(1.0);
            
            sum += y * weight;
            weight_sum += weight;
        }

        let avg = if weight_sum > 0.0 {
            (sum / weight_sum) / 255.0
        } else {
            0.0
        };
        Ok(avg.clamp(0.0, 1.0))
    }

    /// Legacy wrapper or for calibration (full precision, flat average)
    pub fn average_luma(&mut self) -> Result<f32, Box<dyn Error>> {
        // Calibration prefers raw flat average? Or consistent with measure?
        // User asked for "Smart... accurate".
        // For calibration keying "darkest vs bright", center weighting is probably fine too, 
        // but let's stick to measure_luma(false) for full precision.
        self.measure_luma(false)
    }

    pub fn average_luma_over(&mut self, frames: usize) -> Result<f32, Box<dyn Error>> {
        if frames == 0 {
            return self.average_luma();
        }
        let mut acc = 0.0f32;
        for _ in 0..frames {
            acc += self.average_luma()?;
        }
        Ok(acc / frames as f32)
    }
}
