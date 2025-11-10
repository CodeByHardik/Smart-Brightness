// src/backlight.rs
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::config::autodetect_backlight_file;

fn read_u32_from<P: AsRef<Path>>(p: P) -> Option<u32> {
    std::fs::read_to_string(p).ok()?.trim().parse::<u32>().ok()
}

fn write_u32_to<P: AsRef<Path>>(p: P, v: u32) -> std::io::Result<()> {
    let mut f = File::create(p)?;
    write!(f, "{}", v)
}

pub struct Backlight {
    pub path: PathBuf,
    pub max_value: u32,
}

impl Backlight {
    pub fn resolve(_cfg: &crate::config::Config) -> Result<Self, Box<dyn std::error::Error>> {
        let max_path = autodetect_backlight_file("max_brightness")
            .ok_or("cannot find max_brightness")?;

        let path = autodetect_backlight_file("brightness")
            .ok_or("cannot find brightness")?;

        let max_value = read_u32_from(&max_path).ok_or("cannot read max_brightness")?;
        Ok(Self { path, max_value })
    }

    pub fn set(&self, value: u32) -> std::io::Result<()> {
        write_u32_to(&self.path, value.clamp(0, self.max_value))
    }

    pub fn current(&self) -> Option<u32> {
        read_u32_from(&self.path)
    }
}