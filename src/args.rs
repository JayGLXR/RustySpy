use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Find available windows for spying
    Find,

    /// Spy on a specific window or process
    Spy {
        /// Window title to spy on
        #[arg(short, long)]
        window: Option<String>,

        /// Process ID to spy on
        #[arg(short, long)]
        pid: Option<u32>,

        /// Log file to write events to
        #[arg(short, long)]
        logfile: Option<PathBuf>,

        /// Ignore app-specific handlers
        #[arg(short, long)]
        ignore_handlers: bool,

        /// Interval in seconds for event processing
        #[arg(short, long, default_value = "1")]
        timeout: u64,

        /// Disable UIA events
        #[arg(long)]
        no_uia_events: bool,

        /// Disable property change events
        #[arg(long)]
        no_property_events: bool,

        /// Block EDR processes
        #[arg(long)]
        block_edr: bool,
    },
} 