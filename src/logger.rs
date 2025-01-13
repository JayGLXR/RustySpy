use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use chrono::Local;

use crate::error::Result;

pub struct Logger {
    file: Option<File>,
}

impl Logger {
    pub fn new() -> Self {
        Logger { file: None }
    }

    pub fn with_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(path)?;
        
        Ok(Logger { file: Some(file) })
    }

    pub fn log(&mut self, message: &str) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let log_entry = format!("[{}] {}\n", timestamp, message);
        
        if let Some(file) = &mut self.file {
            file.write_all(log_entry.as_bytes())?;
            file.flush()?;
        }
        
        log::info!("{}", message);
        Ok(())
    }

    pub fn log_error(&mut self, error: &str) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let log_entry = format!("[{}] ERROR: {}\n", timestamp, error);
        
        if let Some(file) = &mut self.file {
            file.write_all(log_entry.as_bytes())?;
            file.flush()?;
        }
        
        log::error!("{}", error);
        Ok(())
    }
} 