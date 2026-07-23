//! The redesigned tweak engine. `model` is the one typed representation (spec §5/§6); `parse` is
//! the authoring-surface parsers (spec §5.1/§5.2/§6.2/§6.6); `validate` is the build-time
//! structural guards (spec §10); `engine`/`kinds`/`snapshot`/`shared_claims` are the lifecycle,
//! per-address-kind effect implementations, and the two on-disk stores (spec §8/§11).
//!
//! `schema` (YAML → model) is `#[cfg(test)]`-only: `serde_yaml_bw` must never link into the
//! shipped binary, so the real loader lives here only for `cargo test` to exercise against the
//! fixtures in `tweaks_fixtures/`. `build.rs` `#[path]`-includes the same file — a separate
//! compilation unrelated to this crate's own `#[cfg(test)]` gate, exactly like the old
//! `models/tweak_schema.rs` this module's `model`/`parse`/`validate`/`schema` quartet replaced.

#[cfg(test)]
mod e2e_tests;
pub mod engine;
pub mod kinds;
pub mod model;
pub mod parse;
#[cfg(test)]
mod schema;
pub mod shared_claims;
pub mod snapshot;
pub mod validate;
pub mod winver;

pub use model::*;
pub use parse::*;
pub use validate::*;

/// The build-time-compiled corpus (spec §11): `tweaks/examples.yaml`, loaded and validated once
/// by `build.rs` (`schema::load_corpus` + `validate::{validate_structural, validate_semantic}`)
/// and embedded as JSON. Task 16 wires this to the tweak query commands; until then this accessor
/// (and its own round-trip test below, plus the E2E suite in `e2e_tests`) is the artifact's only
/// consumer -- without one, the generated `CORPUS` static would be unreachable dead code.
pub fn compiled_corpus() -> &'static Corpus {
    &crate::generated_corpus::CORPUS
}

#[cfg(test)]
mod compiled_corpus_tests {
    #[test]
    fn embedded_corpus_deserializes_and_is_nonempty() {
        let corpus = super::compiled_corpus();
        assert!(
            !corpus.tweaks.is_empty(),
            "no tweaks were compiled into the binary from tweaks/examples.yaml"
        );
    }
}
