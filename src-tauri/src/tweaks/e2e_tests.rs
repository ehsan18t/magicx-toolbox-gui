//! Real-machine end-to-end proof of the compiled engine (spec §11; Task 15 brief): the real
//! `AllKinds`/`RealProbe`/`RealActions` dispatcher, a real `SnapshotStore`/`ClaimsStore` rooted in
//! a temp dir, and genuine HKCU registry addresses under a unique
//! `HKCU\Software\MagicXToolboxE2E\...` scratch subtree — never mocks (unlike the engine's own
//! per-module unit tests in `engine::apply`/`engine::revert`, which substitute
//! `MockKind`/`MockProbes`/`MockActions`). These three tests prove invariants 1-26 hold together
//! end-to-end on a real machine, not just per module in isolation.
//!
//! Default-run, HKCU-only, no admin needed: every test cleans its own scratch registry key(s) via
//! a Drop guard, even on panic/assertion failure (mirrors `kinds::registry`'s own `Scratch`), so a
//! failed run never leaves residue behind or collides with a later run. Residue is additionally
//! confirmed from *outside* this process (PowerShell) after the suite runs — see the task report.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::models::RegistryHive;
use crate::services::registry_service;
use crate::tweaks::engine::detect::TweakState;
use crate::tweaks::engine::{
    apply, detect, revert, AllKinds, Deps, ProbeCache, RealActions, RealProbe,
};
use crate::tweaks::model::{
    Corpus, Effect, EffectDef, EffectId, FieldAddr, Hive, Level, Opt, OptLabel, OptValue,
    PackedFormat, RegAddr, RegType, RiskLevel, ScopedValue, Setting, SharedDef, SharedId, Tweak,
    TypedRegValue, Value,
};
use crate::tweaks::shared_claims::ClaimsStore;
use crate::tweaks::snapshot::SnapshotStore;
use crate::tweaks::winver::WinVer;

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A unique HKCU scratch subtree that deletes itself (recursively) on drop, even on panic —
/// mirrors `kinds::registry`'s own `Scratch` so a failed assertion never leaks a real key.
struct Scratch {
    path: String,
}
impl Scratch {
    fn new(label: &str) -> Self {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        Scratch {
            path: format!(
                "Software\\MagicXToolboxE2E\\{label}_{}_{n}",
                std::process::id()
            ),
        }
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        // Cleanup only -- the one accepted `let _` exception (a Drop-guard restoring state).
        let _ = registry_service::delete_key(&RegistryHive::Hkcu, &self.path);
    }
}

/// Owns the temp-dir-backed real stores + probe cache for one test, so `Deps` (all borrows)
/// outlives the engine calls it drives. Mirrors `engine::apply`'s own test `Harness`, with the
/// real dispatcher/probe/action sources (`AllKinds`/`RealProbe`/`RealActions`) in place of mocks.
struct Harness {
    kinds: AllKinds,
    probes: RealProbe,
    actions: RealActions,
    claims: ClaimsStore,
    snapshots: SnapshotStore,
    cache: ProbeCache,
    _tmp: tempfile::TempDir,
}
impl Harness {
    fn new() -> Self {
        let tmp = tempfile::tempdir().expect("create temp snapshots/claims root");
        Self {
            kinds: AllKinds,
            probes: RealProbe,
            actions: RealActions,
            claims: ClaimsStore::open(tmp.path().to_path_buf(), Some("e2e-test-guid".into())),
            snapshots: SnapshotStore::open(tmp.path().to_path_buf()),
            cache: ProbeCache::new(),
            _tmp: tmp,
        }
    }

    fn deps(&self) -> Deps<'_> {
        Deps {
            kinds: &self.kinds,
            probes: &self.probes,
            actions: &self.actions,
            claims: &self.claims,
            snapshots: &self.snapshots,
            probe_cache: &self.cache,
            machine_guid: Some("e2e-test-guid"),
            level: Level::User,
            running: WinVer {
                build: 19045,
                revision: 0,
            },
        }
    }
}

// --- fixture builders (mirrors engine::apply/engine::revert's own test-module helpers) --------

fn dword_effect(id: &str, path: &str, name: &str) -> EffectDef {
    EffectDef {
        id: EffectId(id.to_string()),
        kind: Effect::Setting(Setting::Registry(RegAddr {
            hive: Hive::Hkcu,
            path: path.to_string(),
            name: name.to_string(),
            ty: RegType::Dword,
            field: None,
        })),
        elevation: None,
        optional: false,
        if_missing: None,
        windows: None,
    }
}

fn shared_effect(id: &str, shared_id: &str) -> EffectDef {
    EffectDef {
        id: EffectId(id.to_string()),
        kind: Effect::Shared(SharedId(shared_id.to_string())),
        elevation: None,
        optional: false,
        if_missing: None,
        windows: None,
    }
}

fn set_value(value: Value) -> OptValue {
    OptValue::Set(ScopedValue {
        value,
        windows: None,
    })
}

fn opt(label: &str, values: Vec<(&str, OptValue)>) -> Opt {
    let mut map = BTreeMap::new();
    for (id, v) in values {
        map.insert(EffectId(id.to_string()), v);
    }
    Opt {
        label: OptLabel(label.to_string()),
        values: map,
    }
}

fn tweak(id: &str, surface: Vec<EffectDef>, options: Vec<Opt>) -> Tweak {
    Tweak {
        id: id.to_string(),
        name: id.to_string(),
        description: String::new(),
        category: "e2e".to_string(),
        info: None,
        warning: None,
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        elevation: Level::User,
        reversible: true,
        surface,
        options,
        windows: None,
    }
}

fn corpus(tweaks: Vec<Tweak>, shared: Vec<SharedDef>) -> Corpus {
    Corpus {
        categories: Vec::new(),
        tweaks,
        shared,
    }
}

// --- the three brief-mandated scenarios ---------------------------------------------------------

/// `detect SD -> apply A -> Active(A) -> apply B -> Active(B) -> restore -> Active(A) -> restore
/// -> SD`, asserting snapshots are consumed exactly per the walk (brief's `registry_tweak_full_lifecycle`).
#[tokio::test]
async fn registry_tweak_full_lifecycle() {
    let scratch = Scratch::new("registry_lifecycle");
    let h = Harness::new();
    let t = tweak(
        "e2e_registry_lifecycle",
        vec![dword_effect("demo", &scratch.path, "Demo")],
        vec![
            opt(
                "A",
                vec![("demo", set_value(Value::Reg(TypedRegValue::Dword(1))))],
            ),
            opt(
                "B",
                vec![("demo", set_value(Value::Reg(TypedRegValue::Dword(2))))],
            ),
        ],
    );
    let c = corpus(vec![t.clone()], vec![]);
    let deps = h.deps();
    let list_len = |h: &Harness| {
        h.snapshots
            .list(&t.id, &c, Some("e2e-test-guid"), 19045)
            .unwrap()
            .len()
    };

    // detect SD: a never-touched scratch key matches neither authored option.
    let status = detect::detect(&t, &c, &deps);
    assert_eq!(status.state, TweakState::SystemDefault);
    assert_eq!(list_len(&h), 0);

    // apply A
    let outcome = apply::apply(&t, &c, &OptLabel("A".into()), &deps)
        .await
        .expect("apply A must succeed");
    assert_eq!(
        outcome.status.state,
        TweakState::Active(OptLabel("A".into()))
    );
    assert_eq!(
        registry_service::read_dword(&RegistryHive::Hkcu, &scratch.path, "Demo").unwrap(),
        Some(1),
        "the real registry value must actually be 1"
    );
    assert_eq!(list_len(&h), 1, "one snapshot entry after the first apply");

    // apply B
    let outcome = apply::apply(&t, &c, &OptLabel("B".into()), &deps)
        .await
        .expect("apply B must succeed");
    assert_eq!(
        outcome.status.state,
        TweakState::Active(OptLabel("B".into()))
    );
    assert_eq!(
        registry_service::read_dword(&RegistryHive::Hkcu, &scratch.path, "Demo").unwrap(),
        Some(2)
    );
    assert_eq!(
        list_len(&h),
        2,
        "two snapshot entries after the second apply"
    );

    // restore -> Active(A)
    let restored = revert::restore(&t, &c, &deps)
        .await
        .expect("restore back to A must succeed");
    assert!(restored.consumed.is_some());
    assert_eq!(
        restored.status.state,
        TweakState::Active(OptLabel("A".into()))
    );
    assert_eq!(
        registry_service::read_dword(&RegistryHive::Hkcu, &scratch.path, "Demo").unwrap(),
        Some(1)
    );
    assert_eq!(list_len(&h), 1, "exactly one entry consumed");

    // restore -> System Default
    let restored = revert::restore(&t, &c, &deps)
        .await
        .expect("restore back to System Default must succeed");
    assert!(restored.consumed.is_some());
    assert_eq!(restored.status.state, TweakState::SystemDefault);
    assert_eq!(
        registry_service::read_dword(&RegistryHive::Hkcu, &scratch.path, "Demo").unwrap(),
        None,
        "the value must be truly absent again"
    );
    assert_eq!(list_len(&h), 0, "every snapshot entry consumed");

    // final detect confirms it, independent of the restore's own reported status
    let status = detect::detect(&t, &c, &deps);
    assert_eq!(status.state, TweakState::SystemDefault);
}

/// `apply both -> both Active, revert one -> other still Active, revert last -> original
/// restored` (brief's `shared_pair_lifecycle`) — two independent tweaks, each with its own private
/// marker Setting plus a Shared reference to the SAME corpus-level shared setting. This is also
/// the real-machine proof of the Task 15 revert.rs fix: without it, the *second* claimant's revert
/// (always a `Captured::Values` dump — see `engine::revert`'s module docs, Fix 2) would never
/// release its shared claim, permanently stranding the shared value away from its true original.
#[tokio::test]
async fn shared_pair_lifecycle() {
    let scratch_shared = Scratch::new("shared_pair_shared");
    let scratch_x = Scratch::new("shared_pair_x");
    let scratch_y = Scratch::new("shared_pair_y");
    // The true pre-existing original this address held before either tweak ever claimed it.
    registry_service::set_dword(&RegistryHive::Hkcu, &scratch_shared.path, "Flag", 0)
        .expect("seed the shared value's original");

    let h = Harness::new();
    let shared_id = "e2e_shared_dword";
    let shared = SharedDef {
        id: SharedId(shared_id.into()),
        setting: Setting::Registry(RegAddr {
            hive: Hive::Hkcu,
            path: scratch_shared.path.clone(),
            name: "Flag".into(),
            ty: RegType::Dword,
            field: None,
        }),
        value: Value::Reg(TypedRegValue::Dword(1)),
    };

    fn shared_pair_tweak(id: &str, marker_path: &str, shared_id: &str) -> Tweak {
        tweak(
            id,
            vec![
                dword_effect("marker", marker_path, "Marker"),
                shared_effect("shared_ref", shared_id),
            ],
            vec![
                opt(
                    "On",
                    vec![
                        ("marker", set_value(Value::Reg(TypedRegValue::Dword(1)))),
                        ("shared_ref", OptValue::Claim(None)),
                    ],
                ),
                opt(
                    "Off",
                    vec![
                        ("marker", set_value(Value::Reg(TypedRegValue::Dword(0)))),
                        ("shared_ref", OptValue::Unclaimed(None)),
                    ],
                ),
            ],
        )
    }

    let tx = shared_pair_tweak("e2e_shared_x", &scratch_x.path, shared_id);
    let ty = shared_pair_tweak("e2e_shared_y", &scratch_y.path, shared_id);
    let c = corpus(vec![tx.clone(), ty.clone()], vec![shared]);
    let deps = h.deps();
    let shared_key = SharedId(shared_id.into());
    let live_shared =
        || registry_service::read_dword(&RegistryHive::Hkcu, &scratch_shared.path, "Flag").unwrap();

    // Neither tweak's private marker has been touched yet, so neither option matches.
    assert_eq!(
        detect::detect(&tx, &c, &deps).state,
        TweakState::SystemDefault
    );
    assert_eq!(
        detect::detect(&ty, &c, &deps).state,
        TweakState::SystemDefault
    );

    // apply both -> both Active
    let ox = apply::apply(&tx, &c, &OptLabel("On".into()), &deps)
        .await
        .expect("X claims first");
    assert_eq!(ox.status.state, TweakState::Active(OptLabel("On".into())));
    assert_eq!(
        h.claims.holders(&shared_key),
        vec!["e2e_shared_x".to_string()]
    );
    assert_eq!(
        live_shared(),
        Some(1),
        "first claim drives the shared value"
    );

    let oy = apply::apply(&ty, &c, &OptLabel("On".into()), &deps)
        .await
        .expect("Y claims second (a verified no-op drive, still added as a claimant)");
    assert_eq!(oy.status.state, TweakState::Active(OptLabel("On".into())));
    assert_eq!(
        h.claims.holders(&shared_key),
        vec!["e2e_shared_x".to_string(), "e2e_shared_y".to_string()]
    );
    assert_eq!(
        detect::detect(&tx, &c, &deps).state,
        TweakState::Active(OptLabel("On".into()))
    );
    assert_eq!(
        detect::detect(&ty, &c, &deps).state,
        TweakState::Active(OptLabel("On".into()))
    );

    // revert one (X) -> other (Y) still Active; shared value untouched (still held by Y)
    let rx = revert::restore(&tx, &c, &deps)
        .await
        .expect("revert X must succeed");
    assert!(rx.consumed.is_some());
    assert_eq!(
        h.claims.holders(&shared_key),
        vec!["e2e_shared_y".to_string()],
        "X's release must leave Y's claim intact"
    );
    assert_eq!(
        live_shared(),
        Some(1),
        "the shared value is left alone while Y still claims it"
    );
    assert_eq!(
        registry_service::read_dword(&RegistryHive::Hkcu, &scratch_x.path, "Marker").unwrap(),
        None,
        "X's own private marker must be restored"
    );
    assert_eq!(
        detect::detect(&ty, &c, &deps).state,
        TweakState::Active(OptLabel("On".into()))
    );

    // revert last (Y) -> the true original is restored
    let ry = revert::restore(&ty, &c, &deps)
        .await
        .expect("revert Y must succeed");
    assert!(ry.consumed.is_some());
    assert!(
        !h.claims.is_claimed(&shared_key),
        "the last release must clear the claim entirely"
    );
    assert_eq!(
        live_shared(),
        Some(0),
        "the last release must restore the true captured original"
    );
    assert_eq!(
        registry_service::read_dword(&RegistryHive::Hkcu, &scratch_y.path, "Marker").unwrap(),
        None
    );
    assert_eq!(
        detect::detect(&tx, &c, &deps).state,
        TweakState::SystemDefault
    );
    assert_eq!(
        detect::detect(&ty, &c, &deps).state,
        TweakState::SystemDefault
    );
}

/// Hand-seeds an unknown field alongside the addressed one, runs a full apply + restore cycle,
/// and asserts the unknown field survives untouched throughout (brief's `packed_field_lifecycle`,
/// spec §5.2).
#[tokio::test]
async fn packed_field_lifecycle() {
    let scratch = Scratch::new("packed_lifecycle");
    // `Foo=1;` is the unknown field this tweak's effect never addresses; `MyFlag=0;` is the one it
    // does. Both are hand-seeded before the engine ever touches this value.
    registry_service::set_string(
        &RegistryHive::Hkcu,
        &scratch.path,
        "Packed",
        "Foo=1;MyFlag=0;",
    )
    .expect("hand-seed the packed value");

    let h = Harness::new();
    let effect = EffectDef {
        id: EffectId("my_flag".into()),
        kind: Effect::Setting(Setting::Registry(RegAddr {
            hive: Hive::Hkcu,
            path: scratch.path.clone(),
            name: "Packed".into(),
            ty: RegType::Sz,
            field: Some(FieldAddr {
                field: "MyFlag".into(),
                format: PackedFormat::KvSemicolon,
            }),
        })),
        elevation: None,
        optional: false,
        if_missing: None,
        windows: None,
    };
    let t = tweak(
        "e2e_packed_lifecycle",
        vec![effect],
        vec![
            opt(
                "On",
                vec![(
                    "my_flag",
                    set_value(Value::Reg(TypedRegValue::Sz("1".into()))),
                )],
            ),
            opt(
                "Off",
                vec![(
                    "my_flag",
                    set_value(Value::Reg(TypedRegValue::Sz("0".into()))),
                )],
            ),
        ],
    );
    let c = corpus(vec![t.clone()], vec![]);
    let deps = h.deps();
    let raw_value =
        || registry_service::read_string(&RegistryHive::Hkcu, &scratch.path, "Packed").unwrap();

    // The hand-seeded MyFlag=0 already matches the "Off" option.
    assert_eq!(
        detect::detect(&t, &c, &deps).state,
        TweakState::Active(OptLabel("Off".into()))
    );

    let outcome = apply::apply(&t, &c, &OptLabel("On".into()), &deps)
        .await
        .expect("apply On must succeed");
    assert_eq!(
        outcome.status.state,
        TweakState::Active(OptLabel("On".into()))
    );
    assert_eq!(
        raw_value(),
        Some("Foo=1;MyFlag=1;".to_string()),
        "the unknown field Foo=1; must survive the apply, in its original position"
    );

    let restored = revert::restore(&t, &c, &deps)
        .await
        .expect("restore back to Off must succeed");
    assert_eq!(
        restored.status.state,
        TweakState::Active(OptLabel("Off".into()))
    );
    assert_eq!(
        raw_value(),
        Some("Foo=1;MyFlag=0;".to_string()),
        "the unknown field Foo=1; must survive the full apply+restore round trip"
    );
}
