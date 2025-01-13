use thiserror::Error;
use windows::core::Error as WindowsError;

#[derive(Error, Debug)]
pub enum SpyError {
    #[error("Windows API error: {0}")]
    Windows(#[from] WindowsError),
    
    #[error("Process not found: {0}")]
    ProcessNotFound(u32),
    
    #[error("Window not found: {0}")]
    WindowNotFound(String),
    
    #[error("UI Automation error: {0}")]
    UiaError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("DNS exfiltration error: {0}")]
    DnsExfiltration(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

pub type Result<T> = std::result::Result<T, SpyError>; 