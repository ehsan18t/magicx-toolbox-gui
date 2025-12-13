# MagicX Toolbox

A modern Windows system tweaking application built with Tauri 2 and SvelteKit. Apply, revert, and manage Windows registry tweaks and service configurations through an intuitive GUI.

## Features

- 🚀 **Tauri 2** - Build smaller, faster, and more secure desktop applications.
- ⚡ **SvelteKit + Svelte 5** - Modern reactive framework with runes.
- 🎨 **Tailwind CSS v4** - Utility-first CSS framework.
- 🌟 **Icons**: Easily use thousands of icons from [Iconify](https://iconify.design/).
- 🌙 **Dark/Light Theme** - Built-in theme switching with system preference detection.
- 🎭 **Custom Titlebar** - Beautiful, native-feeling window controls.
- 🔧 **TypeScript** - Full type safety.
- 📦 **Modern Build Tools** - Vite, ESLint, Prettier.

### System Tweaking Features

- **Registry Tweaks**: Toggle or multi-state registry modifications
- **Service Control**: Manage Windows services startup types
- **Scheduled Tasks**: Enable/disable Windows scheduled tasks
- **Windows Version Filtering**: Tweaks filtered by Windows 10/11 compatibility
- **Snapshot-Based Backup**: Automatic state capture before applying tweaks
- **Risk Levels**: Clear indication of tweak impact (low/medium/high/critical)

## Quick Start

1. **Clone and install**
   ```bash
   git clone https://github.com/ehsan18t/magicx-toolbox-gui.git
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
├── src/                      # Frontend source
│   ├── lib/
│   │   ├── api/              # Tauri command wrappers
│   │   ├── components/       # Svelte components
│   │   │   ├── ui/           # Reusable UI primitives
│   │   │   └── tweak-details/# Tweak display sub-components
│   │   ├── stores/           # Svelte 5 rune-based stores (.svelte.ts)
│   │   ├── config/           # App configuration
│   │   └── types/            # TypeScript types
│   ├── routes/               # SvelteKit routes
│   └── app.css               # Global styles & CSS variables
├── src-tauri/                # Tauri backend
│   ├── src/
│   │   ├── commands/         # Tauri command handlers
│   │   │   └── tweaks/       # Modular tweak commands (query, apply, batch)
│   │   ├── models/           # Data structures
│   │   └── services/         # Business logic (registry, services, scheduler)
│   └── tweaks/               # YAML tweak definitions
├── static/                   # Static assets
└── README.md
```

## Documentation

### Backend Development

For detailed information about working with the Rust backend:

**📖 [Read the Rust Backend Developer Guide](./RUST_BACKEND_GUIDE.md)**

### Authoring Tweaks (YAML)

Tweaks live in `src-tauri/tweaks/*.yaml`. Each file defines **one category** plus a list of tweaks.

**📖 [Read the Tweak Authoring Guide](./TWEAK_AUTHORING.md)**

### Architecture Overview

For understanding the overall architecture and data flow:

**📖 [Read the Architecture Guide](./ARCHITECTURE.md)**

## Customization

### Theme

Edit `src/app.css` to customize colors and design tokens.

### App Configuration

Update `src/lib/config/app.ts` for app metadata and settings.

### Window Settings
Modify `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml` for window behavior and permissions.

## License

MIT License - see LICENSE file for details.