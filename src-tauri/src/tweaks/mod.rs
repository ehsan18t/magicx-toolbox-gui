//! The redesigned tweak engine. `model` is the one typed representation (spec §5/§6); `parse` is
//! the authoring-surface parsers (spec §5.1/§5.2/§6.2/§6.6); `validate` is the build-time
//! structural guards (spec §10). Later tasks add kind implementations, the engine, and storage.
//!
//! `schema` (YAML → model) is `#[cfg(test)]`-only: `serde_yaml_bw` must never link into the
//! shipped binary, so the real loader lives here only for `cargo test` to exercise against the
//! fixtures in `tweaks_fixtures/`. It is written so a later task's `build.rs` can `#[path]`-include
//! the same file — that inclusion is a separate compilation unrelated to this crate's own
//! `#[cfg(test)]` gate, exactly like today's `models/tweak_schema.rs`.

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
