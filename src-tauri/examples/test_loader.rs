fn main() {
    // Inline test - just try to read a YAML file directly
    use serde::Deserialize;
    use std::fs;

    #[derive(Debug, Deserialize)]
    struct CategoryDef {
        id: String,
        name: String,
    }

    #[derive(Debug, Deserialize)]
    struct TweakFile {
        category: CategoryDef,
        tweaks: Vec<serde_json::Value>,
    }

    println!("Testing YAML file loading...\n");

    let yaml_path = "tweaks/privacy.yaml";
    match fs::read_to_string(yaml_path) {
        Ok(content) => {
            println!("✓ Read file: {} bytes", content.len());
            match serde_yaml::from_str::<TweakFile>(&content) {
                Ok(file) => {
                    println!("✓ Parsed successfully!");
                    println!("  Category: {} ({})", file.category.name, file.category.id);
                    println!("  Tweaks: {}", file.tweaks.len());
                }
                Err(e) => {
                    println!("✗ YAML parse error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to read file: {}", e);
            println!("  Trying to find tweaks directory...");
            match std::env::current_dir() {
                Ok(cwd) => println!("  Current directory: {}", cwd.display()),
                Err(_) => println!("  Could not get current directory"),
            }
        }
    }
}
