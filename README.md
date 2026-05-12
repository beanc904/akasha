# Akasha

A **Rust TUI library and dashboard** for controlling [Mihomo](https://mihomo.me/).

Akasha provides a terminal-native experience for managing Mihomo proxy with a focus on terminal usability, lightweight interaction, and scriptable Rust APIs.

## Features

- **Unix Socket IPC Communication** - Efficient bidirectional communication with Mihomo over Unix sockets
- **Rust Client Library** - Fully async/await Rust client implementation with connection pooling
- **Terminal UI** - Cross-platform TUI built with [Ratatui](https://ratatui.rs/) (in development)
- **Scriptable APIs** - Usable as a standalone library in other Rust applications
- **Connection Management** - Connection pool with configurable timeouts and health checks
- **System Proxy Control** - System-wide proxy settings management (Windows, macOS, Linux)

## Project Status

### ✅ Implemented

- Mihomo Unix socket IPC communication layer
- Rust async client implementation (`akasha::client`)
- Connection pooling with health checks
- Version querying and system info
- Usable as a standalone library
- System proxy control utilities

### 🚧 In Progress

- TUI frontend and interactive dashboard
- Proxy and connection visualization
- Interactive Mihomo control interface

## Architecture

```
Mihomo Core
    │
    ├─ Unix Socket IPC
    │
Akasha Client (async Rust)
    │
    ├─ Connection Pool
    ├─ Command Handlers
    └─ Error Handling
    │
Akasha TUI (in development)
    │
    └─ Dashboard & Controls
```

## Installation

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
akasha = { git = "https://github.com/beanc904/akasha.git" }
```

### From Source

```bash
git clone https://github.com/beanc904/akasha.git
cd akasha
cargo build --release
```

## Quick Start

### Basic Example

```rust
use akasha::client as ac;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sock_path = env::var("AKASHA_SOCKET_PATH")
        .unwrap_or_else(|_| "/tmp/akasha/mihomo.sock".to_string());

    let mihomo = ac::Builder::new()
        .protocol(ac::Protocol::LocalSocket)
        .socket_path(sock_path)
        .pool_config(
            ac::IpcPoolConfigBuilder::new()
                .min_connections(0)
                .max_connections(20)
                .idle_timeout(std::time::Duration::from_millis(500))
                .health_check_interval(std::time::Duration::from_secs(10))
                .build(),
        )
        .build()?;

    let client = mihomo.read().await;
    let version = client.get_version().await?;
    println!("Mihomo version: {}", version);

    Ok(())
}
```

For more examples, see [examples/client.rs](examples/client.rs).

## Project Structure

```
akasha/
├── src/
│   ├── lib/
│   │   ├── client/          # Mihomo IPC client
│   │   │   ├── commands.rs  # Command implementations
│   │   │   ├── error.rs     # Error types
│   │   │   └── mod.rs       # Client builder & pool
│   │   └── mod.rs           # Library root
│   ├── bin/                 # Binary targets (TUI)
│   └── main.rs
├── crates/
│   ├── aka-logger/          # Logging utilities
│   └── sysproxy-rs/         # System proxy management
├── examples/                # Usage examples
├── Cargo.toml
└── README.md
```

## Configuration

### Socket Path

Configure the Mihomo socket path via environment variable:

```bash
export AKASHA_SOCKET_PATH=/path/to/mihomo.sock
```

Default: `/tmp/akasha/mihomo.sock`

### Connection Pool

Customize connection pool behavior in your code:

```rust
ac::IpcPoolConfigBuilder::new()
    .min_connections(5)              // Minimum pool size
    .max_connections(50)             // Maximum pool size
    .idle_timeout(Duration::from_secs(30))      // Connection idle timeout
    .health_check_interval(Duration::from_secs(30)) // Health check frequency
    .build()
```

## Platform Support

- **Linux** - Full support
- **macOS** - Full support
- **Windows** - Full support (with WSL for Unix sockets)

## Dependencies

Key dependencies:
- **tokio** - Async runtime
- **ratatui** - TUI framework
- **crossterm** - Terminal handling
- **serde** - Serialization
- **reqwest** - HTTP client
- **tokio-tungstenite** - WebSocket support

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).

## Background

Akasha originated from extracting the Mihomo control logic from a **Tauri plugin** and refactoring it into a standalone, framework-independent Rust library. This refactoring enables:

- Reusable Mihomo communication layer
- Independence from GUI frameworks
- Suitability for terminal and server applications
- Better testability and modularity

## Resources

- [Mihomo Documentation](https://mihomo.me/)
- [Ratatui Documentation](https://docs.rs/ratatui/)
- [Tokio Documentation](https://tokio.rs/)

## Support

For issues, questions, or suggestions, please [open an issue](https://github.com/beanc904/akasha/issues) on GitHub.
