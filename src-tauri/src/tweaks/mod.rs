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

/// The build-time-compiled corpus (spec §11): `tweaks/*.yaml`, loaded and validated once
/// by `build.rs` (`schema::load_corpus` + `validate::{validate_structural, validate_semantic}`)
/// and embedded as JSON. Task 16 wires this to the tweak query commands; until then this accessor
/// (and its own round-trip test below, plus the E2E suite in `e2e_tests`) is the artifact's only
/// consumer -- without one, the generated `CORPUS` static would be unreachable dead code.
pub fn compiled_corpus() -> &'static Corpus {
    &crate::generated_corpus::CORPUS
}

#[cfg(test)]
mod compiled_corpus_tests {
    use crate::tweaks::engine::context;
    use crate::tweaks::model::Level;

    #[test]
    fn embedded_corpus_deserializes_and_is_nonempty() {
        let corpus = super::compiled_corpus();
        assert!(
            !corpus.tweaks.is_empty(),
            "no tweaks were compiled into the binary from tweaks/*.yaml"
        );
    }

    /// The divergence pin. The original defect was not a wrong answer, it was TWO answers to "does
    /// this tweak touch HKCU": the availability guard read the `elevation:` floor while routing read
    /// the hive. Hand-built fixtures cannot catch that -- only asserting the two agree across the
    /// whole real corpus can. If a future `Effect` variant or a new hive-carrying `Setting` makes
    /// them drift again, this fails.
    #[test]
    fn the_guard_and_routing_agree_about_hkcu_across_the_whole_corpus() {
        let corpus = super::compiled_corpus();
        for tweak in &corpus.tweaks {
            let guard_says = context::tweak_touches_hkcu(tweak, corpus);

            // Route each effect against a synthetic `Ti`-floor copy. At that floor
            // `effective_level` can only ever answer `Ti`, so the ONE remaining way `route` can
            // return `Level::User` is the HKCU exception -- which isolates the hive decision from
            // the floor. Routing against the tweak's real floor would be vacuous: a `user`-floor
            // tweak routes EVERY effect to `User` whatever its hive, so the comparison would hold
            // for the wrong reason today and fail spuriously the day someone authors a `user`-floor
            // tweak with no HKCU effect (an `action:` script, say).
            let mut probe = tweak.clone();
            probe.elevation = Level::Ti;
            let routing_says = probe
                .surface
                .iter()
                .any(|e| context::route(e, &probe, corpus).level() == Level::User);

            assert_eq!(
                guard_says, routing_says,
                "tweak '{}' (floor {:?}): the availability guard says touches_hkcu={guard_says} \
                 but routing's HKCU exception says {routing_says}",
                tweak.id, tweak.elevation
            );
        }
    }

    /// The corpus fact the guard's re-keying exists for: HKCU effects are NOT confined to
    /// `elevation: user` tweaks. If this ever reads zero, the floor-keyed guard would have been
    /// adequate after all and this assertion is the place to find out.
    #[test]
    fn admin_floor_tweaks_do_touch_hkcu() {
        let corpus = super::compiled_corpus();
        let admin_floor_hkcu = corpus
            .tweaks
            .iter()
            .filter(|t| t.elevation != Level::User && context::tweak_touches_hkcu(t, corpus))
            .count();
        assert!(
            admin_floor_hkcu > 0,
            "expected some non-user-floor tweaks to drive HKCU effects; if this is genuinely 0 now, \
             the hive-keyed guard is still correct, just no longer load-bearing"
        );
    }
}
