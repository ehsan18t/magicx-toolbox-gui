//! Loads and validates the YAML corpus under `tweaks/` at compile time and embeds the result.
//!
//! `#[path]`-includes the runtime's own `model.rs`/`parse.rs`/`validate.rs`/`schema.rs`, so a
//! renamed field or a changed validation rule is a compile error on both sides, never drift.
//! `schema.rs` is `#[cfg(test)]` in the app, so `serde_yaml_bw` never links into it.

use std::path::Path;

// This build reaches only the corpus load and validation; the app crate checks the rest.
#[path = "src/tweaks/model.rs"]
#[allow(dead_code)]
mod model;
#[path = "src/tweaks/parse.rs"]
#[allow(dead_code)]
mod parse;
#[path = "src/tweaks/schema.rs"]
mod schema;
#[path = "src/tweaks/validate.rs"]
#[allow(dead_code)]
mod validate;

/// tauri-build's default manifest declares only Common Controls v6, which the dialog plugin needs.
/// The DPI awareness matches what tao requests at runtime, so the manifest settles it before any
/// window (or a startup failure's message box) exists.
const APP_MANIFEST: &str = r#"<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0"
        processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*" />
    </dependentAssembly>
  </dependency>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}" />
    </application>
  </compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <longPathAware xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">true</longPathAware>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2, PerMonitor</dpiAwareness>
    </windowsSettings>
  </application>
</assembly>
"#;

fn main() {
    let windows = tauri_build::WindowsAttributes::new().app_manifest(APP_MANIFEST);
    if let Err(e) =
        tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
    {
        panic!("{e:#}");
    }

    if let Err(e) = generate_corpus() {
        panic!("{e}");
    }
}

/// Loads `tweaks/`, runs every build-time guard (spec §10), and embeds the validated corpus as
/// JSON for `tweaks::compiled_corpus()` to deserialize at runtime.
fn generate_corpus() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let tweaks_dir = Path::new(&manifest_dir).join("tweaks");
    let out_dir = std::env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    // One `rerun-if-changed` line turns off Cargo's whole-package scan, so every input is listed.
    println!("cargo:rerun-if-changed=tweaks/");
    println!("cargo:rerun-if-changed=src/tweaks/model.rs");
    println!("cargo:rerun-if-changed=src/tweaks/parse.rs");
    println!("cargo:rerun-if-changed=src/tweaks/validate.rs");
    println!("cargo:rerun-if-changed=src/tweaks/schema.rs");

    let (corpus, apps) = match schema::load_corpus_with_apps(&tweaks_dir) {
        Ok(loaded) => loaded,
        Err(errors) => return Err(validation_report("YAML LOAD FAILED", &errors).into()),
    };

    let structural_errors = validate::validate_structural(&corpus);
    if !structural_errors.is_empty() {
        return Err(validation_report("STRUCTURAL VALIDATION FAILED", &structural_errors).into());
    }
    let semantic_errors = validate::validate_semantic(&corpus, validate::SUPPORT_MATRIX);
    if !semantic_errors.is_empty() {
        return Err(validation_report("SEMANTIC VALIDATION FAILED", &semantic_errors).into());
    }
    let app_errors = validate::validate_apps(&corpus, &apps);
    if !app_errors.is_empty() {
        return Err(validation_report("APP VALIDATION FAILED", &app_errors).into());
    }

    let corpus_json = serde_json::to_string(&corpus)?;
    std::fs::write(out_path.join("corpus.json"), corpus_json)?;
    std::fs::write(out_path.join("apps.json"), serde_json::to_string(&apps)?)?;

    let generated_code = r#"// AUTO-GENERATED FILE - DO NOT EDIT
// Generated from tweaks/*.yaml at build time by build.rs. To modify the corpus, edit the YAML and
// rebuild.

use crate::tweaks::model::{AppDef, Corpus};
use std::sync::LazyLock;

/// Raw JSON of the compiled, build-time-validated corpus (embedded at compile time).
pub const CORPUS_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/corpus.json"));

/// The compiled corpus, deserialized once. `tweaks::compiled_corpus()` is the crate's own
/// accessor -- callers should go through that, not this module, directly.
pub static CORPUS: LazyLock<Corpus> = LazyLock::new(|| {
    serde_json::from_str(CORPUS_JSON).expect("failed to parse embedded corpus JSON")
});

pub static APPS: LazyLock<Vec<AppDef>> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/apps.json")))
        .expect("failed to parse embedded apps JSON")
});
"#;
    std::fs::write(out_path.join("generated_corpus.rs"), generated_code)?;

    println!(
        "cargo:warning=✓ Validated and compiled {} categor{}, {} tweak{}, {} shared setting{}, {} app{} from tweaks/",
        corpus.categories.len(),
        if corpus.categories.len() == 1 { "y" } else { "ies" },
        corpus.tweaks.len(),
        if corpus.tweaks.len() == 1 { "" } else { "s" },
        corpus.shared.len(),
        if corpus.shared.len() == 1 { "" } else { "s" },
        apps.len(),
        if apps.len() == 1 { "" } else { "s" },
    );

    Ok(())
}

/// Every problem in one framed report, so an author sees them all in one run.
fn validation_report<E: std::fmt::Display>(title: &str, errors: &[E]) -> String {
    let mut report =
        String::from("\n╔══════════════════════════════════════════════════════════════╗\n");
    report.push_str(&format!("║ {title:<62}║\n"));
    report.push_str("╠══════════════════════════════════════════════════════════════╣\n");
    report.push_str(&format!(
        "║ {:.<62}║\n",
        format!("{} error(s) found:", errors.len())
    ));
    report.push_str("╠══════════════════════════════════════════════════════════════╣\n");
    for (i, error) in errors.iter().enumerate() {
        // Errors may span multiple lines; each line needs its own frame.
        let numbered = format!("{}. {}", i + 1, error);
        for line in numbered.lines() {
            report.push_str(&format!("║ {}\n", line));
        }
    }
    report.push_str("╚══════════════════════════════════════════════════════════════╝\n");
    report
}
