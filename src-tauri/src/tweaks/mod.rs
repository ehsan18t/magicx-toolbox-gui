//! The tweak engine: `model` (spec §5/§6), `parse`, `validate` (spec §10), and
//! `engine`/`kinds`/`snapshot`/`shared_claims` (spec §8/§11).
//!
//! `schema` (YAML to model) is `#[cfg(test)]`-only: `serde_yaml_bw` must never link into the
//! shipped binary. `build.rs` `#[path]`-includes it as a separate compilation.

#[cfg(test)]
mod e2e_tests;
pub mod engine;
pub mod kinds;
pub mod model;
pub mod parse;
#[cfg(test)]
mod scan_bench;
#[cfg(test)]
mod schema;
pub mod shared_claims;
pub mod snapshot;
pub mod validate;
pub mod winver;

pub use model::*;
pub use parse::*;
pub use validate::*;

/// The build-time-compiled corpus (spec §11): `tweaks/*.yaml`, loaded and validated once by
/// `build.rs` and embedded as JSON.
pub fn compiled_corpus() -> &'static Corpus {
    &crate::generated_corpus::CORPUS
}

/// The build-time-compiled app items (ADR-0009), from the same YAML files as the corpus.
pub fn compiled_apps() -> &'static [AppDef] {
    &crate::generated_corpus::APPS
}

#[cfg(test)]
mod compiled_corpus_tests {
    use crate::tweaks::engine::context;
    use crate::tweaks::model::{AppSource, InstallSource, Level};

    #[test]
    fn the_shipped_app_items_compile_with_their_sources_and_install_routes() {
        use InstallSource::{Store, StorePage, Winget};
        let expected: [(&str, &str, &[&str], InstallSource); 14] = [
            (
                "teams_consumer",
                "debloat",
                &["MSTeams"],
                Store("XP8BT8DW290MPQ".into()),
            ),
            (
                "clipchamp",
                "debloat",
                &["Clipchamp.Clipchamp"],
                Store("9P1J8S7CCWWT".into()),
            ),
            (
                "quick_assist",
                "debloat",
                &["MicrosoftCorporationII.QuickAssist"],
                Store("9P7BP5VNWKX5".into()),
            ),
            (
                "bing_news",
                "debloat",
                &["Microsoft.BingNews"],
                Store("9WZDNCRFHVFW".into()),
            ),
            (
                "bing_weather",
                "debloat",
                &["Microsoft.BingWeather"],
                Store("9WZDNCRFJ3Q2".into()),
            ),
            (
                "solitaire",
                "debloat",
                &["Microsoft.MicrosoftSolitaireCollection"],
                StorePage("9WZDNCRFHWD2".into()),
            ),
            (
                "get_help",
                "debloat",
                &["Microsoft.GetHelp"],
                Store("9PKDZBMV1H3T".into()),
            ),
            (
                "getstarted_tips",
                "debloat",
                &["Microsoft.Getstarted"],
                StorePage("9WZDNCRDTBJJ".into()),
            ),
            (
                "feedback_hub",
                "debloat",
                &["Microsoft.WindowsFeedbackHub"],
                Store("9NBLGGH4R32N".into()),
            ),
            (
                "phone_link",
                "debloat",
                &["Microsoft.YourPhone"],
                Store("9NMPJ99VJBWV".into()),
            ),
            (
                "outlook_new",
                "debloat",
                &["Microsoft.OutlookforWindows"],
                Store("9NRX63209R7B".into()),
            ),
            (
                "xbox_game_bar",
                "debloat",
                &["Microsoft.XboxGamingOverlay"],
                Store("9NZKPSTSNW4P".into()),
            ),
            (
                "onedrive",
                "debloat",
                &[],
                Winget("Microsoft.OneDrive".into()),
            ),
            (
                "copilot",
                "ai",
                &["Microsoft.Copilot"],
                Store("9NHT9RB2F4HD".into()),
            ),
        ];
        let apps = super::compiled_apps();
        assert_eq!(apps.len(), expected.len(), "unexpected app item count");
        for (id, category, packages, install) in expected {
            let app = apps
                .iter()
                .find(|a| a.id == id)
                .unwrap_or_else(|| panic!("app '{id}' was not compiled"));
            assert_eq!(app.category, category, "{id}");
            assert_eq!(app.install.as_ref(), Some(&install), "{id}");
            match &app.source {
                AppSource::Appx(names) => assert_eq!(names, packages, "{id}"),
                AppSource::Script { .. } => assert!(packages.is_empty(), "{id} is a script item"),
            }
        }
    }

    /// Probe exit 2 reads as absent, so a probe that fails must not land there.
    #[test]
    fn the_onedrive_probe_never_reports_absent_from_its_catch_block() {
        let app = super::compiled_apps()
            .iter()
            .find(|a| a.id == "onedrive")
            .expect("onedrive app item");
        let AppSource::Script { probe, timeout, .. } = &app.source else {
            panic!("onedrive must be a script item");
        };
        assert_eq!(*timeout, Some(600));
        let catch = probe.find("catch").expect("the probe has a catch block");
        let open = catch + probe[catch..].find('{').expect("catch opens a block");
        let mut depth = 0;
        let close = probe[open..]
            .char_indices()
            .find_map(|(i, c)| {
                match c {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
                (depth == 0).then_some(open + i)
            })
            .expect("catch block closes");
        let body = &probe[open..=close];
        assert!(!body.contains("exit 2"), "catch block exits 2: {body}");
        assert!(body.contains("exit 1"), "catch block must exit 1: {body}");
    }

    #[test]
    fn embedded_apps_deserialize_in_the_shared_category_space() {
        let corpus = super::compiled_corpus();
        for app in super::compiled_apps() {
            assert!(
                corpus.categories.iter().any(|c| c.id == app.category),
                "app '{}' names no compiled category",
                app.id
            );
        }
    }

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
            let routing_says = probe.surface.iter().any(|e| {
                context::route(e, &probe, corpus).level() == Level::User
                    || context::probe_reads_hkcu(e)
            });

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
