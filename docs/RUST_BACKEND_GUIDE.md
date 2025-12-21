# Tauri + SvelteKit Rust Backend Developer Guide

This guide will help you understand and work with the Rust backend of this Tauri + SvelteKit starter template.

## 📁 Project Structure Overview

```
src-tauri/
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Core application setup and configuration
│   ├── state.rs         # Shared application state management
│   ├── error.rs         # Custom error types and handling
│   ├── setup.rs         # Application initialization and setup
│   └── commands/        # Tauri commands (backend API)
│       ├── mod.rs       # Module exports
│       └── general.rs   # General purpose commands
├── Cargo.toml           # Rust dependencies and project configuration
├── build.rs             # Build script
└── tauri.conf.json      # Tauri application configuration
```

## 🔧 Core Components Explained

### 1. Application Entry Point (`main.rs`)

The simplest file in your project. It just calls the `run()` function from your library:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    app_lib::run();
}
```

**Key Points:**
- The `windows_subsystem` attribute prevents a console window from appearing on Windows in release builds
- All the real work happens in `lib.rs`

### 2. Core Application (`lib.rs`)

This is where your Tauri application is built and configured:

```rust
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState(Default::default()))  // Initialize shared state
        .setup(setup::setup)                   // Run setup logic
        .invoke_handler(tauri::generate_handler![...]) // Register commands
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Key Points:**
- `.manage()` makes your state available to all commands
- `.setup()` runs one-time initialization code
- `.invoke_handler()` registers functions that can be called from the frontend

### 3. State Management (`state.rs`)

Manages data that needs to be shared across your application:

```rust
use std::sync::Mutex;

pub struct AppState(pub Mutex<State>);

#[derive(Default)]
pub struct State {
    pub greetings: Vec<String>,
    // Add more fields as needed
}
```

**Key Points:**
- `Mutex` ensures thread-safe access to your data
- `AppState` is a wrapper that Tauri can manage
- Add any data you need to persist during the app's lifetime to `State`

### 4. Error Handling (`error.rs`)

Provides consistent error handling across your application:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    // Add custom error variants here
}

pub type Result<T> = std::result::Result<T, Error>;
```

**Key Points:**
- All command functions should return `Result<T>` instead of `T`
- Errors are automatically serialized and sent to the frontend
- Add custom error variants as your application grows

### 5. Setup Logic (`setup.rs`)

Handles one-time initialization when the app starts:

```rust
pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    {
        // Setup logging for development
        let log_plugin = LogBuilder::new()
            .targets([
                Target::new(TargetKind::Webview),
                Target::new(TargetKind::Stdout),
                Target::new(TargetKind::LogDir { file_name: None }),
            ])
            .level(log::LevelFilter::Info)
            .build();
        app.handle().plugin(log_plugin)?;
    }
    Ok(())
}
```

**Key Points:**
- Only runs once when the application starts
- Perfect place for database connections, configuration loading, etc.
- The example sets up logging for debug builds

## 🚀 Working with Commands

Commands are Rust functions that your SvelteKit frontend can call. They're defined in the `commands/` directory.

### Creating a New Command

**Step 1:** Add your function to a command file (e.g., `commands/general.rs`):

```rust
#[tauri::command]
pub fn my_new_command(input: String, state: tauri::State<AppState>) -> Result<String> {
    let mut state = state.0.lock().expect("Failed to lock state mutex");
    
    // Your logic here
    log::info!("Processing: {}", input);
    
    Ok(format!("Processed: {}", input))
}
```

**Step 2:** Register it in `lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    commands::general::greet,
    commands::general::get_greetings,
    commands::general::clear_greetings,
    commands::general::my_new_command  // Add your command here
])
```

**Step 3:** Call it from your SvelteKit frontend:

```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('my_new_command', { input: 'Hello from frontend!' });
console.log(result); // "Processed: Hello from frontend!"
```

### Command Best Practices

1. **Always use the custom `Result` type:**
   ```rust
   // ✅ Good
   pub fn my_command() -> Result<String> { Ok("success".to_string()) }
   
   // ❌ Avoid
   pub fn my_command() -> String { "success".to_string() }
   ```

2. **Use descriptive parameter names:**
   ```rust
   // ✅ Good
   pub fn save_user_data(username: String, email: String) -> Result<()>
   
   // ❌ Avoid
   pub fn save_data(a: String, b: String) -> Result<()>
   ```

3. **Add logging for debugging:**
   ```rust
   #[tauri::command]
   pub fn important_operation(data: String) -> Result<String> {
       log::info!("Starting important operation with: {}", data);
       // ... your logic
       log::info!("Operation completed successfully");
       Ok(result)
   }
   ```

## 📊 Working with State

The application state allows you to store data that persists throughout your app's lifetime.

### Adding New State Fields

**Step 1:** Update the `State` struct in `state.rs`:

```rust
#[derive(Default)]
pub struct State {
    pub greetings: Vec<String>,
    pub user_preferences: UserPreferences,  // New field
    pub connection_status: bool,            // New field
}

#[derive(Default)]
pub struct UserPreferences {
    pub theme: String,
    pub language: String,
}
```

**Step 2:** Access state in your commands:

```rust
#[tauri::command]
pub fn update_theme(new_theme: String, state: tauri::State<AppState>) -> Result<()> {
    let mut state = state.0.lock().expect("Failed to lock state mutex");
    state.user_preferences.theme = new_theme;
    Ok(())
}

#[tauri::command]
pub fn get_theme(state: tauri::State<AppState>) -> Result<String> {
    let state = state.0.lock().expect("Failed to lock state mutex");
    Ok(state.user_preferences.theme.clone())
}
```

### State Management Tips

1. **Keep state minimal:** Only store data that truly needs to be shared
2. **Use appropriate data structures:** Choose `Vec`, `HashMap`, etc. based on your access patterns
3. **Consider persistence:** State is lost when the app closes; use files/databases for permanent storage

## 🔄 Frontend-Backend Communication

### From SvelteKit to Rust

Use the `invoke` function to call Rust commands:

```typescript
import { invoke } from '@tauri-apps/api/core';

// Simple command
const greeting = await invoke('greet', { name: 'Alice' });

// Command with complex data
const result = await invoke('process_data', { 
    data: { 
        items: ['item1', 'item2'], 
        count: 42 
    } 
});

// Error handling
try {
    const result = await invoke('risky_operation');
} catch (error) {
    console.error('Command failed:', error);
}
```

### Data Types

Rust and JavaScript types are automatically converted:

| Rust Type            | JavaScript Type | Example            |
| -------------------- | --------------- | ------------------ |
| `String`             | `string`        | `"hello"`          |
| `i32`, `u32`, etc.   | `number`        | `42`               |
| `bool`               | `boolean`       | `true`             |
| `Vec<T>`             | `Array<T>`      | `[1, 2, 3]`        |
| `HashMap<String, T>` | `Object`        | `{key: value}`     |
| Custom structs       | `Object`        | `{field1: value1}` |

## 🐛 Debugging and Logging

### Using Logs

The template includes logging setup. Use these macros in your Rust code:

```rust
log::error!("Something went wrong: {}", error_message);
log::warn!("This might be a problem: {}", warning);
log::info!("Operation completed: {}", details);
log::debug!("Debug info: {:?}", complex_data);
```

### Viewing Logs

- **Development:** Logs appear in your terminal and browser console
- **Production:** Logs are written to files in the app's log directory

### Common Debugging Scenarios

1. **Command not found:**
   - Check if the command is registered in `lib.rs`
   - Verify the function name matches what you're calling from the frontend

2. **State access fails:**
   - Ensure `AppState` is managed in `lib.rs`
   - Check that you're passing `state: tauri::State<AppState>` to your command

3. **Serialization errors:**
   - Make sure your return types implement `Serialize`
   - Use `#[derive(Serialize)]` on custom structs

## 📦 Adding Dependencies

To add new Rust dependencies, edit `src-tauri/Cargo.toml`:

```toml
[dependencies]
serde_json = "1.0"
serde = { version = "1.0", features = ["derive"] }
# Add new dependencies here
tokio = { version = "1.0", features = ["full"] }  # For async operations
sqlx = "0.7"                                      # For databases
reqwest = "0.11"                                  # For HTTP requests
```

Then run `cargo build` to install them.

## 🏗️ Advanced Patterns

### Organizing Large Applications

As your app grows, organize commands into logical modules:

```
commands/
├── mod.rs
├── auth.rs       # Authentication commands
├── database.rs   # Database operations
├── file_system.rs # File operations
└── network.rs    # Network requests
```

Update `commands/mod.rs`:

```rust
pub mod auth;
pub mod database;
pub mod file_system;
pub mod general;
pub mod network;
```

### Async Commands

For operations that take time (network requests, file I/O):

```rust
#[tauri::command]
pub async fn fetch_data_from_api(url: String) -> Result<String> {
    let response = reqwest::get(&url).await
        .map_err(|_| Error::NetworkError)?;
    
    let text = response.text().await
        .map_err(|_| Error::NetworkError)?;
    
    Ok(text)
}
```

### Custom Error Types

Add specific error variants for better error handling:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    
    #[error("Network request failed: {0}")]
    NetworkError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Invalid input: {0}")]
    ValidationError(String),
}
```

## 📚 Additional Resources

- [Tauri API Documentation](https://docs.rs/tauri/latest/tauri/)
- [Tauri Documentation](https://tauri.app/start/)
- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust fundamentals
- [Rust Async Programming](https://rust-lang.github.io/async-book/) - Learn about async in Rust
- [Serde Documentation](https://serde.rs/) - For data serialization
- [thiserror Documentation](https://docs.rs/thiserror/) - For error handling
- [SvelteKit Documentation](https://kit.svelte.dev/docs/introduction)
- [SvelteKit Playground](https://svelte.dev/playground/hello-world)
