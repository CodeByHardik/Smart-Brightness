use std::fmt::Display;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use chrono::Local;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::Builder;

use crate::config::LogLevel;

const MAX_ARCHIVES: usize = 10;
const LATEST_LOG: &str = "latest-log.txt";

#[derive(Clone)]
pub struct Logger {
    level: LogLevel,
    sink: Option<Arc<LogSink>>,
}

impl Logger {
    pub fn new(level: LogLevel, path: Option<&str>) -> Self {
        let sink = match LogSink::create(path) {
            Ok(opt) => opt.map(Arc::new),
            Err(err) => {
                eprintln!("Failed to initialize log file: {}", err);
                None
            }
        };
        Self { level, sink }
    }

    #[inline]
    pub fn enabled(&self, level: LogLevel) -> bool {
        level <= self.level
    }

    #[inline]
    pub fn info<F>(&self, f: F)
    where
        F: FnOnce() -> String,
    {
        self.log(LogLevel::Medium, Target::Stdout, f);
    }

    #[inline]
    pub fn status<F>(&self, f: F)
    where
        F: FnOnce() -> String,
    {
        self.log(LogLevel::Low, Target::Stdout, f);
    }

    #[inline]
    pub fn warn<F>(&self, f: F)
    where
        F: FnOnce() -> String,
    {
        self.log(LogLevel::Minimal, Target::Stderr, f);
    }

    #[inline]
    pub fn error<E: Display>(&self, err: E) {
        self.log(LogLevel::Minimal, Target::Stderr, || err.to_string());
    }

    fn log<F>(&self, level: LogLevel, target: Target, f: F)
    where
        F: FnOnce() -> String,
    {
        if !self.enabled(level) || level == LogLevel::Off {
            return;
        }
        let msg = f();
        match target {
            Target::Stdout => println!("{}", msg),
            Target::Stderr => eprintln!("{}", msg),
        }
        if let Some(sink) = &self.sink {
            sink.write_line(level, &msg);
        }
    }
}

#[derive(Clone, Copy)]
enum Target {
    Stdout,
    Stderr,
}

struct LogSink {
    writer: Mutex<BufWriter<File>>,
}

impl LogSink {
    fn create(path: Option<&str>) -> io::Result<Option<Self>> {
        let (active_dir, archive_dir) = resolve_dirs(path)?;
        fs::create_dir_all(&active_dir)?;
        fs::create_dir_all(&archive_dir)?;

        let latest_path = active_dir.join(LATEST_LOG);
        rotate_logs(&latest_path, &archive_dir)?;
        let file = File::create(&latest_path)?;
        Ok(Some(Self {
            writer: Mutex::new(BufWriter::new(file)),
        }))
    }

    fn write_line(&self, level: LogLevel, msg: &str) {
        if let Ok(mut guard) = self.writer.lock() {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(guard, "[{}][{:?}] {}", timestamp, level, msg);
            let _ = guard.flush();
        }
    }
}

fn resolve_dirs(custom: Option<&str>) -> io::Result<(PathBuf, PathBuf)> {
    if let Some(path) = custom {
        let base = expand_path(path);
        return Ok((base.clone(), base));
    }
    let cache_base = dirs::cache_dir().unwrap_or_else(default_root);
    let base = cache_base.join("SMART_BRIGHTNESS").join("logs");
    Ok((base.clone(), base))
}

fn expand_path(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

fn default_root() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn rotate_logs(latest: &Path, archive_dir: &Path) -> io::Result<()> {
    if latest.exists() {
        let ts = Local::now().format("%Y%m%d-%H%M%S");
        let archive_path = archive_dir.join(format!("log-{}.tar.gz", ts));
        let file = File::create(&archive_path)?;
        let encoder = GzEncoder::new(file, Compression::default());
        let mut builder = Builder::new(encoder);
        builder.append_path_with_name(latest, LATEST_LOG)?;
        builder.finish()?;
        fs::remove_file(latest)?;
    }
    prune_archives(archive_dir)?;
    Ok(())
}

fn prune_archives(dir: &Path) -> io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    let mut archives = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        if name.starts_with("log-") && name.ends_with(".tar.gz") {
            let modified = entry
                .metadata()?
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            archives.push((modified, path));
        }
    }
    if archives.len() <= MAX_ARCHIVES {
        return Ok(());
    }
    archives.sort_by_key(|(modified, _)| *modified);
    let excess = archives.len() - MAX_ARCHIVES;
    for (_, path) in archives.into_iter().take(excess) {
        let _ = fs::remove_file(path);
    }
    Ok(())
}
