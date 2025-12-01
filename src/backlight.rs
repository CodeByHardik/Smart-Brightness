// src/backlight.rs
use std::cell::Cell;
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
    actual_path: Option<PathBuf>,
    last_value: Cell<Option<u32>>,
}

impl Backlight {
    pub fn resolve(_cfg: &crate::config::Config) -> Result<Self, Box<dyn std::error::Error>> {
        let max_path =
            autodetect_backlight_file("max_brightness").ok_or("cannot find max_brightness")?;

        let path = autodetect_backlight_file("brightness").ok_or("cannot find brightness")?;

        let max_value = read_u32_from(&max_path).ok_or("cannot read max_brightness")?;
        let actual_path = path
            .parent()
            .map(|p| p.join("actual_brightness"))
            .filter(|p| p.exists());
        Ok(Self {
            path,
            max_value,
            actual_path,
            last_value: Cell::new(None),
        })
    }

    pub fn set(&self, value: u32) -> std::io::Result<()> {
        let v = value.clamp(0, self.max_value);
        if self.last_value.get() == Some(v) {
            return Ok(());
        }
        let r = write_u32_to(&self.path, v);
        if r.is_ok() {
            self.last_value.set(Some(v));
        }
        r
    }

    pub fn current(&self) -> Option<u32> {
        read_u32_from(&self.path)
    }

    pub fn actual(&self) -> Option<u32> {
        if let Some(p) = &self.actual_path {
            read_u32_from(p)
        } else {
            self.current()
        }
    }

    pub fn actual_path(&self) -> Option<&Path> {
        self.actual_path.as_deref()
    }
}
