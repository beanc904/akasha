# Akasha

Akasha is a Rust project aimed at building a **terminal user interface (TUI)** for controlling **mihomo**.

The project provides a lightweight Rust client for communicating with mihomo through **Unix socket IPC**.
This client is primarily designed to support the TUI frontend, but it can also be used independently by other Rust applications.

# Project Goals

The main goal of Akasha is to create a **terminal-native experience for managing mihomo**.

Most existing mihomo clients are GUI-based. Akasha explores a different direction by focusing on:

- terminal usability
- lightweight interaction
- scriptable Rust APIs

The project currently focuses on building the **communication layer between Rust and mihomo**.

# Current Status

Implemented:

- mihomo Unix socket IPC communication
- Rust client implementation (`akasha::client`)
- usable as a standalone Rust library

Work in progress:

- TUI frontend
- proxy and connection visualization
- interactive mihomo control

# Architecture

```text
mihomo core
     │
     │ IPC (Unix socket)
     ▼
akasha::client
     │
     ▼
Akasha TUI (in development)
```

- **mihomo** handles proxy logic and networking
- **akasha::client** communicates with mihomo
- **Akasha TUI** will provide the terminal interface

# Installation

You can use Akasha directly from GitHub:

```toml
[dependencies]
akasha = { git = "https://github.com/beanc904/akasha.git" }
```

# Usage Example

Example of connecting to mihomo via Unix socket:

```rust
use std::env;
use std::error::Error;

use akasha::client as ac;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let sock_path = env::var("AKASHA_SOCKET_PATH").unwrap_or("/tmp/akasha/mihomo.sock".to_string());

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
        .build();

    if let Ok(mi) = &mihomo {
        let m = mi.read().await;
        let version = m.get_version().await;
        println!("Mihomo version: {:?}", version);
    }
    
    Ok(())
}
```

To learn more about the usage of client, visit [client.rs](examples/client.rs).

# Project Structure

```text
akasha
 ├─ tui           # terminal UI (work in progress)
 └─ lib
     └─ client        # mihomo IPC client implementation
         ├─ commands  # exposing the calling functions
         └─ error     # error definitions
```

Currently, the `client` module contains the core functionality for communicating with mihomo.

# Background

This project originated from extracting the mihomo control logic from a **Tauri plugin** and refactoring it into a standalone Rust library.

The goal of this refactoring was to make the mihomo communication layer:

- reusable
- independent from GUI frameworks
- suitable for terminal applications

# License

[GPLv3](LICENSE)
