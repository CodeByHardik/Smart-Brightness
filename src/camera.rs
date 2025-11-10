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
        Ok(Self { _dev: dev, stream })
    }

    pub fn warmup(&mut self, frames: usize) {
        eprintln!("Warming up camera…");
        for _ in 0..frames {
            let _ = self.stream.next();
        }
        eprintln!("Camera ready.");
    }

    pub fn average_luma(&mut self) -> Result<f32, Box<dyn Error>> {
        let (buf, _) = self.stream.next()?;
        let mut sum: u64 = 0;
        let mut cnt: u64 = 0;
        for pair in buf.chunks_exact(2) {
            sum += pair[0] as u64;
            cnt += 1;
        }
        let avg = if cnt > 0 { (sum as f32) / (cnt as f32) / 255.0 } else { 0.0 };
        Ok(avg.clamp(0.0, 1.0))
    }

    pub fn average_luma_over(&mut self, frames: usize) -> Result<f32, Box<dyn Error>> {
        if frames == 0 { return self.average_luma(); }
        let mut acc = 0.0f32;
        let mut n = 0usize;
        for _ in 0..frames {
            if let Ok(v) = self.average_luma() { acc += v; n += 1; }
        }
        if n == 0 { Ok(0.0) } else { Ok(acc / n as f32) }
    }
}