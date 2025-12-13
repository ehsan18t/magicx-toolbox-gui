# MagicX Toolbox Backend (src-tauri/src)

Rust backend for the MagicX Toolbox application, built with [Tauri](https://tauri.app/).

## Architecture

```
src/
├── lib.rs              # Application entry point and Tauri setup
├── main.rs             # Binary entry point
├── error.rs            # Custom error types (thiserror)
├── state.rs            # Application state management
├── setup.rs            # Startup initialization
├── debug.rs            # Debug logging utilities
├── commands/           # Tauri command handlers
│   ├── general.rs      # Theme and preferences
│   ├── elevation.rs    # SYSTEM/TI elevation commands
│   ├── update.rs       # App update checking
│   ├── system.rs       # System info retrieval
│   ├── backup.rs       # Backup commands
│   ├── debug.rs        # Debug mode commands
│   └── tweaks/         # Tweak apply/revert commands
├── services/           # Business logic
│   ├── backup_service.rs       # Snapshot-based backup system
│   ├── registry_service.rs     # Windows registry operations
│   ├── trusted_installer.rs    # SYSTEM/TI elevation
│   ├── service_control.rs      # Windows service management
│   ├── scheduler_service.rs    # Task scheduler operations
│   ├── system_info_service.rs  # Hardware/OS info via WMI
│   └── tweak_loader.rs         # Pre-compiled tweak definitions
└── models/             # Data structures
    ├── tweak.rs        # Tweak definitions
    ├── system.rs       # System info models
    ├── backup.rs       # Backup helper types
    ├── registry.rs     # Registry types
    └── tweak_snapshot.rs  # Snapshot models
```

## Key Features

- **Tweak System**: Option-based tweaks with atomic apply/revert
- **Snapshot Backup**: Capture state before changes for rollback
- **Privilege Elevation**: SYSTEM and TrustedInstaller support
- **Pre-compiled Tweaks**: YAML → Rust at build time for performance

## Testing

```bash
# Run all unit tests
cargo test --lib

# Run specific module tests
cargo test --lib backup_service
cargo test --lib trusted_installer
```

## Code Quality

- Linting: `cargo clippy` (see clippy.toml for config)
- Formatting: `cargo fmt` (see rustfmt.toml for config)
