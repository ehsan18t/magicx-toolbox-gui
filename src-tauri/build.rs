//! Build script for the redesigned tweak engine (spec §11/§12). Loads and validates the YAML
//! corpus under `tweaks/` at compile time and embeds the result — the hard cutover's replacement
//! for the old option-centric schema/validation codegen this file used to hold.
//!
//! `#[path]`-includes the runtime's own `model.rs`/`parse.rs`/`validate.rs`/`schema.rs` verbatim
//! (Task 3's carry-forward, continuing the technique the old `models/tweak_schema.rs` shim used):
//! build.rs and the `app_lib` crate compile the identical source twice, as two separate crate
//! compilations connected only by the JSON this script embeds below — a renamed field or a changed
//! validation rule is a compile error on *both* sides, never silent drift. `schema.rs` is the one
//! file with no other caller in a shipped binary (`tweaks/mod.rs` gates it `#[cfg(test)]` so
//! `serde_yaml_bw` never links into the app); this is its other, non-test caller.

use std::path::Path;

// `#[allow(dead_code)]`: build.rs's own compilation only ever reaches `load_corpus` +
// `validate_structural` + `validate_semantic` — a narrower call graph than the full runtime crate
// (which also links `kinds`/`engine`, exercising the rest of `parse`/`validate`'s public surface).
// The same source, included normally (`pub mod parse;` etc.) in `tweaks/mod.rs`, still gets full
// dead-code scrutiny there; this allow is scoped to this reduced, throwaway compilation only.
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

fn main() {
    tauri_build::build();

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

    // Emitting even one `rerun-if-changed` line switches Cargo off its whole-package-scan default
    // (see the Cargo book), so every input this script actually reads must be listed explicitly —
    // the corpus directory AND the four #[path]-included source files above, whose own edits must
    // re-trigger validation exactly as a YAML edit does.
    println!("cargo:rerun-if-changed=tweaks/");
    println!("cargo:rerun-if-changed=src/tweaks/model.rs");
    println!("cargo:rerun-if-changed=src/tweaks/parse.rs");
    println!("cargo:rerun-if-changed=src/tweaks/validate.rs");
    println!("cargo:rerun-if-changed=src/tweaks/schema.rs");

    let corpus = match schema::load_corpus(&tweaks_dir) {
        Ok(corpus) => corpus,
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

    let corpus_json = serde_json::to_string(&corpus)?;
    std::fs::write(out_path.join("corpus.json"), corpus_json)?;

    let generated_code = r#"// AUTO-GENERATED FILE - DO NOT EDIT
// Generated from tweaks/*.yaml at build time by build.rs. To modify the corpus, edit the YAML and
// rebuild.

use crate::tweaks::model::Corpus;
use std::sync::LazyLock;

/// Raw JSON of the compiled, build-time-validated corpus (embedded at compile time).
pub const CORPUS_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/corpus.json"));

/// The compiled corpus, deserialized once. `tweaks::compiled_corpus()` is the crate's own
/// accessor -- callers should go through that, not this module, directly.
pub static CORPUS: LazyLock<Corpus> = LazyLock::new(|| {
    serde_json::from_str(CORPUS_JSON).expect("failed to parse embedded corpus JSON")
});
"#;
    std::fs::write(out_path.join("generated_corpus.rs"), generated_code)?;

    println!(
        "cargo:warning=✓ Validated and compiled {} categor{}, {} tweak{}, {} shared setting{} from tweaks/",
        corpus.categories.len(),
        if corpus.categories.len() == 1 { "y" } else { "ies" },
        corpus.tweaks.len(),
        if corpus.tweaks.len() == 1 { "" } else { "s" },
        corpus.shared.len(),
        if corpus.shared.len() == 1 { "" } else { "s" },
    );

    Ok(())
}

/// One framed report naming every problem in a single build failure — an author sees everything
/// wrong in one run, matching the box-drawing shape the old build.rs's own reports used.
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
