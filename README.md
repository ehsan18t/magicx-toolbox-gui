# Tauri + SvelteKit Starter Template

A modern, full-featured starter template for building desktop applications with Tauri 2 and SvelteKit.

## Features

- 🚀 **Tauri 2** - Build smaller, faster, and more secure desktop applications.
- ⚡ **SvelteKit** - The fastest way to build svelte apps.
- 🎨 **Tailwind CSS v4** - Utility-first CSS framework.
- 🌟 **Icons**: Easily use thousands of icons from [Iconify](https://iconify.design/).
- 🌙 **Dark/Light Theme** - Built-in theme switching with system preference detection.
- 🎭 **Custom Titlebar** - Beautiful, native-feeling window controls.
- 🔧 **TypeScript** - Full type safety.
- 📦 **Modern Build Tools** - Vite, ESLint, Prettier.

## Quick Start

1. **Clone and install**
   ```bash
   git clone https://github.com/ehsan18t/magicx-toolbox-gui.git
   ```
   ```bash
   cd magicx-toolbox-gui
   bun install
   ```

2. **Development**
   ```bash
   bun run dev
   ```

3. **Build**
   ```bash
   bun run build
   ```

4. **Build Debug**
   ```bash
   bun run build:debug
   ```
## Development

### Commands

This template comes with a set of pre-configured scripts to help you with development and maintenance.

| Command                | Description                                                                           |
| :--------------------- | :------------------------------------------------------------------------------------ |
| `bun run dev`          | Starts the Tauri development server with hot-reloading for both frontend and backend. |
| `bun run build`        | Builds and bundles the application for production.                                    |
| `bun run build:debug`  | Creates a debug build of the application.                                             |
|                        |                                                                                       |
| `bun run format`       | Formats all source files with Prettier.                                               |
| `bun run format:check` | Checks for formatting errors without modifying files.                                 |
| `bun run lint`         | Lints the source files using ESLint.                                                  |
| `bun run lint:fix`     | Lints and automatically fixes problems.                                               |
| `bun run check`        | Runs the Svelte type-checker.                                                         |
| `bun run validate`     | Runs all quality checks: format, lint, and type-check.                                |
|                        |                                                                                       |
| `bun run clean`        | Removes all build artifacts and temporary directories.                                |
| `bun run prepare`      | SvelteKit's command to generate types                                                 |

### Project Structure
```
├── src/                   # Frontend source
│   ├── lib/               # Shared components and utilities
│   │   ├── components/    # Reusable components
│   │   ├── stores/        # Svelte stores
│   │   └── config/        # App configuration
│   ├── routes/            # SvelteKit routes
│   └── app.html           # HTML template
├── src-tauri/             # Tauri backend
├── static/                # Static assets
└── README.md
```

## Documentation

### Backend Development
For detailed information about working with the Rust backend, including:
- Understanding the project structure
- Creating and managing Tauri commands
- Working with application state
- Error handling best practices
- Frontend-backend communication

**📖 [Read the Rust Backend Developer Guide](./RUST_BACKEND_GUIDE.md)**

### Authoring Tweaks (YAML)
Tweaks live in `src-tauri/tweaks/*.yaml`. Each file defines **one category** plus a list of tweaks. The app auto-discovers all `.yaml` files; no code changes are needed when you add/edit files.

**File layout**
```yaml
category:
   id: performance             # slug, used in code/UI
   name: "Performance"          # display name
   description: "Optimize..."   # shown in UI
   icon: "⚡"                    # emoji or icon text
   order: 2                     # sort order in UI

tweaks:
   - id: unique_id             # must be unique across all files
      name: "Human title"
      description: "What it does"
      risk_level: low | medium | high | critical
      requires_reboot: true | false   # optional, defaults false
      requires_admin: true | false    # optional, defaults false
      info: "Optional extra notes"   # optional
      registry_changes:               # flat list of registry edits
         - hive: HKLM | HKCU
            key: "Path\\To\\Key"
            value_name: "ValueName"
            value_type: REG_DWORD | REG_SZ | REG_EXPAND_SZ | REG_BINARY | REG_MULTI_SZ | REG_QWORD
            enable_value: <json value>   # applied when enabling the tweak
            disable_value: <json value>  # optional; used when reverting
            windows_versions: [10, 11]   # optional; omit or empty = applies to all
```

**Version targeting**
- Omit `windows_versions` (or leave it empty) to apply on both Windows 10 and 11.
- Specify `[10]` or `[11]` to target a single OS.

**Examples**
Common change (applies to all):
```yaml
registry_changes:
   - hive: HKLM
      key: "System\\CurrentControlSet\\Services\\DiagTrack"
      value_name: "Start"
      value_type: REG_DWORD
      enable_value: 4
      disable_value: 2
```

Win10-only extra change:
```yaml
registry_changes:
   - hive: HKLM
      key: "System\\CurrentControlSet\\Services\\dmwappushservice"
      value_name: "Start"
      value_type: REG_DWORD
      enable_value: 4
      disable_value: 2
      windows_versions: [10]
```

Mixed list in one tweak:
```yaml
registry_changes:
   # applies to both 10/11
   - hive: HKCU
      key: "Software\\Microsoft\\GameBar"
      value_name: "UseNexusForGameMode"
      value_type: REG_DWORD
      enable_value: 1
      disable_value: 0
   # Win11-only adjustment
   - hive: HKCU
      key: "Software\\Classes\\CLSID\\{86ca1aa0-34aa-4e8b-a509-50c905bae9a9}\\InprocServer32"
      value_name: ""
      value_type: REG_SZ
      enable_value: ""
      disable_value: null
      windows_versions: [11]
```

**Notes**
- `enable_value` / `disable_value` accept any JSON literal (number, string, null, array for binary bytes, etc.).
- `value_name` can be empty (`""`) when setting the default value of a key.
- Keep IDs unique; categories are sorted by `order`, tweaks within categories keep file order.

## Customization

### Theme
Edit `src/app.css` to customize colors and design tokens.

### App Configuration
Update `src/lib/config/app.ts` for app metadata and settings.

### Window Settings
Modify `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml` for window behavior and permissions.

## License

MIT License - see LICENSE file for details.