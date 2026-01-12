# tronctl

> A modern CLI tool for deploying and managing Tron FullNode with ease.

[**中文文档**](./README.zh-CN.md) | [**English**](./README.md)

## Overview

**tronctl** is a production-ready command-line tool written in Rust that simplifies the entire lifecycle of running a Tron FullNode. It handles everything from initial setup to daily operations, with built-in security features and interactive configuration.

### Key Features

- 🚀 **One-Command Setup** - Initialize and deploy a Tron FullNode in minutes
- 📦 **Smart Downloads** - Automatically fetches the latest FullNode.jar and snapshot data
- 🌐 **Intelligent Server Selection** - Chooses the fastest snapshot server based on network latency
- ⚙️ **Interactive Configuration** - JVM memory settings and snapshot options via interactive prompts
- 🔒 **Security Hardened** - Path traversal protection, file locking, and optional MD5 verification
- 🔄 **Full Lifecycle Management** - Start, stop, restart, and monitor your node effortlessly
- 📊 **Real-time Monitoring** - Health checks, RPC status, and block synchronization tracking
- 🛡️ **Environment Validation** - Pre-flight checks for Java version, memory, and disk space

## System Requirements

| Component | Requirement |
|-----------|-------------|
| **OS** | Linux (tested on Arch Linux) |
| **Java** | Java 8 (1.8.x) |
| **Memory** | 32 GB recommended |
| **Storage** | 2.5 TB+ SSD recommended |
| **Privileges** | Root access required |

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/tron-launcher.git
cd tron-launcher

# Build release binary
cargo build --release

# Install system-wide (optional)
sudo cp target/release/tronctl /usr/local/bin/
```

## Quick Start

### 1. Initialize Your Node

Run the interactive initialization process:

```bash
sudo tronctl init
```

The wizard will guide you through:
- **Snapshot selection** - Choose Lite (53 GB) or Full (2937 GB) snapshot, or sync from genesis
- **MD5 verification** - Optional integrity checking (complete download) vs. streaming (space-efficient)
- **JVM memory** - Configure heap size based on your server (official recommendation: 32 GB RAM)

For non-interactive mode:

```bash
# Use Lite snapshot (recommended for most users)
sudo tronctl init --snapshot lite

# Use Full snapshot (for archive nodes)
sudo tronctl init --snapshot full

# Start from genesis (no snapshot)
sudo tronctl init --snapshot none
```

### 2. Start the Node

```bash
# Start in background (daemon mode)
sudo tronctl start --daemon

# Start in foreground (Ctrl+C to stop)
sudo tronctl start
```

### 3. Check Status

```bash
# Basic status
sudo tronctl status

# Detailed status with sync verification
sudo tronctl status --verbose
```

**Example Output:**
```
状态: 运行中
PID: 12345
进程存活: ✓
RPC 响应: ✓
当前区块: 67890123
```

### 4. View Logs

```bash
# Show last 100 lines (default)
sudo tronctl logs

# Follow logs in real-time
sudo tronctl logs -f

# Show last 500 lines
sudo tronctl logs --lines 500
```

### 5. Stop the Node

```bash
# Graceful shutdown (SIGTERM with 30s timeout)
sudo tronctl stop

# Force kill (SIGKILL)
sudo tronctl stop --force
```

### 6. Restart the Node

```bash
sudo tronctl restart --daemon
```

### 7. Clean Up

Remove all tronctl-generated files:

```bash
# Interactive cleanup (asks for confirmation)
sudo tronctl clean

# Non-interactive cleanup (auto-confirm)
sudo tronctl clean -y
```

During cleanup, you'll be asked:
- **Confirm overall cleanup** - Remove configuration, JAR, logs, and PID files
- **Blockchain data cleanup** - Choose whether to delete blockchain data (can be hundreds of GB to TB)

⚠️ **Note:** The node must be stopped before running cleanup.

## Configuration

After initialization, configuration files are located at:

- **`/etc/tronctl/tronctl.toml`** - tronctl configuration (JVM settings, snapshot type)
- **`/etc/tronctl/tron.conf`** - Tron node configuration (downloaded from official repository)

Edit these files to customize your node behavior. Changes to `tron.conf` require a restart to take effect.

## Advanced Usage

### Specify FullNode Version

```bash
sudo tronctl init --snapshot lite --version GreatVoyage-v4.7.4
```

### Skip Environment Checks

```bash
sudo tronctl init --skip-checks
```

⚠️ **Warning:** Only use this if you're certain your environment meets the requirements.

## Security Features

- **Path Traversal Protection** - Validates all tar archive entries during snapshot extraction
- **File Locking** - Prevents multiple instances from starting simultaneously (PID file locking)
- **Optional MD5 Verification** - Ensures snapshot integrity when enabled
- **No Unwrap Panics** - All error paths properly handled with expect/Result types

## Troubleshooting

### Java Version Error

```bash
# Check Java version
java -version

# Expected output: openjdk version "1.8.0_xxx" or "8.x.x"
```

Install Java 8 if needed:
```bash
# Arch Linux
sudo pacman -S jdk8-openjdk
sudo archlinux-java set java-8-openjdk

# Debian/Ubuntu
sudo apt install openjdk-8-jdk
```

### Permission Denied

All commands require root privileges:
```bash
sudo tronctl <command>
```

### Node Won't Start

1. Check logs: `sudo tronctl logs`
2. Verify Java process: `ps aux | grep java`
3. Check port availability: `sudo netstat -tlnp | grep 8090`
4. Ensure sufficient memory: `free -h`

### RPC Not Responding

The node needs time to initialize (typically 30-60 seconds after start). Monitor with:
```bash
sudo tronctl status
```

If RPC remains unresponsive after 2 minutes, check logs for errors.

## Development

### Building from Source

```bash
# Clone repository
git clone https://github.com/yourusername/tron-launcher.git
cd tron-launcher

# Build debug version
cargo build

# Run tests
cargo test

# Run linter
cargo clippy

# Format code
cargo fmt
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

## Technology Stack

- **Language:** Rust 2024 Edition
- **Async Runtime:** Tokio
- **CLI Framework:** Clap 4.5
- **HTTP Client:** Reqwest (with streaming support)
- **Serialization:** Serde, TOML
- **Logging:** Tracing
- **Interactive UI:** Dialoguer
- **File Locking:** fs2
- **Archive Handling:** tar, flate2, async-compression

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Tron Protocol](https://tron.network/) - The blockchain platform
- [java-tron](https://github.com/tronprotocol/java-tron) - Official Tron node implementation
- [java-tron Releases](https://github.com/tronprotocol/java-tron/releases) - FullNode.jar download repository

## Related Links

- [Tron Official Website](https://tron.network/)
- [java-tron GitHub](https://github.com/tronprotocol/java-tron)
- [Tron Developer Hub](https://developers.tron.network/)
- [Tron Documentation](https://tron.network/documentation)
