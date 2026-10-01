# MagicX Toolbox Backend (src-tauri/src)

Rust backend for the MagicX Toolbox application, built with [Tauri](https://tauri.app/).

## Architecture

```
src/
├── lib.rs              # Application entry point and Tauri setup
├── main.rs             # Binary entry point
├── error.rs            # Custom error types (thiserror)
├── setup.rs            # Startup initialization and managed state
├── debug.rs            # Debug logging utilities
├── window_watchdog.rs  # Shows the window if the frontend never does
├── commands/           # Tauri command handlers
│   ├── general.rs      # Window display
│   ├── elevation.rs    # Restart as administrator
│   ├── update.rs       # App update checking
│   ├── system.rs       # System info retrieval
│   ├── debug.rs        # Debug mode commands
│   └── tweaks.rs       # Tweak query/apply/revert commands
├── services/           # Windows effect primitives
│   ├── elevation/              # Typed elevation broker (admin in-process, TrustedInstaller child)
│   ├── registry_service.rs     # Windows registry operations
│   ├── registry_value.rs       # Typed registry values
│   ├── service_control.rs      # Windows service management (SCM)
│   ├── scheduler_service.rs    # Task Scheduler operations (COM)
│   ├── hosts_service.rs        # Hosts file entries
│   ├── firewall_service.rs     # Firewall rules
│   ├── appx_index.rs           # Installed Appx package index
│   ├── ti_probe.rs             # Whether TrustedInstaller is reachable
│   ├── exclusive_temp.rs       # Tamper-proof temp files for elevated readers
│   ├── system32.rs             # Launch Windows tools by absolute path
│   └── system_info_service.rs  # Hardware/OS info
├── tweaks/             # Tweak system
│   ├── model.rs, parse.rs, schema.rs, validate.rs  # YAML schema, shared with build.rs
│   ├── engine/         # detect, apply, revert, execution-context routing, locks
│   ├── kinds/          # Effect kinds: registry (+ registry_key), service, task, hosts, firewall, action
│   ├── snapshot.rs     # Per-tweak snapshot store
│   ├── shared_claims.rs  # Claims on shared blocks (the `shared` effect kind)
│   └── winver.rs       # Running Windows version
└── models/             # Data structures
    ├── system.rs       # System info models
    └── win_types.rs    # Shared hive/value/startup types
```

## Key Features

- **Tweak System**: Option-based tweaks with apply and revert
- **Snapshots**: Capture state before changes for rollback
- **Privilege Elevation**: Administrator and TrustedInstaller support
- **Pre-compiled Tweaks**: YAML → Rust at build time for performance

## Testing

```bash
# Run all unit tests
cargo test --lib

# Run specific module tests
cargo test --lib tweaks::engine
cargo test --lib services::elevation
```

## Code Quality

- Linting: `cargo clippy` (see clippy.toml for config)
- Formatting: `cargo fmt` (see rustfmt.toml for config)
