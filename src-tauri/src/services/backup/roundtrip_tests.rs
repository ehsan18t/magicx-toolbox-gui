//! End-to-end tests for the capture -> apply -> detect -> restore cycle.
//!
//! These run against the **real** Windows registry, not a mock, under a scratch
//! subtree of `HKCU\Software\MagicXToolboxTest`. That is deliberate:
//!
//! - `require_write_access` (registry_service.rs) only demands admin for HKLM, so
//!   HKCU works unelevated both on a developer machine and on the CI runner.
//! - Every bug this cycle can have is a bug about what Windows actually returns.
//!   A mock would be written against our *belief* about that, which is exactly the
//!   thing the audit found to be wrong in several places.
//!
//! Isolation: every test owns a unique subkey, so the default multi-threaded test
//! harness needs no serialisation. Snapshots are keyed by tweak id, which is also
//! unique per test. Cleanup runs through a `Drop` guard, which the test profile
//! honours even on panic (`panic = "abort"` applies to `[profile.release]` only).

use crate::models::{
    RegistryAction, RegistryChange, RegistryHive, RegistryValueType, RiskLevel, TweakDefinition,
    TweakOption,
};
use crate::services::backup::{
    capture_snapshot, delete_snapshot, detect_tweak_state, restore_from_snapshot, save_snapshot,
};
use crate::services::registry_service;
use std::sync::atomic::{AtomicU32, Ordering};

const SCRATCH_ROOT: &str = r"Software\MagicXToolboxTest";

/// Unique-per-test scratch namespace, so tests never collide under the parallel harness.
fn next_scratch() -> String {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!(
        "{}\\p{}_t{}_{}",
        SCRATCH_ROOT,
        std::process::id(),
        // Thread id has no stable numeric form on stable Rust; the counter alone is
        // unique within the process, and the pid disambiguates concurrent runs.
        n,
        n
    )
}

/// Deletes the scratch key and the tweak's snapshot when the test ends, including
/// on panic. The test profile unwinds, so `Drop` runs.
struct Scratch {
    key: String,
    tweak_id: String,
}

impl Scratch {
    fn new(tweak_id: &str) -> Self {
        Self {
            key: next_scratch(),
            tweak_id: tweak_id.to_string(),
        }
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = registry_service::delete_key(&RegistryHive::Hkcu, &self.key);
        let _ = delete_snapshot(&self.tweak_id);
    }
}

fn dword_change(key: &str, name: &str, value: u32) -> RegistryChange {
    RegistryChange {
        hive: RegistryHive::Hkcu,
        key: key.to_string(),
        value_name: name.to_string(),
        action: RegistryAction::Set,
        value_type: Some(RegistryValueType::Dword),
        value: Some(serde_json::json!(value)),
        windows_versions: None,
        skip_validation: false,
    }
}

fn option(label: &str, registry_changes: Vec<RegistryChange>) -> TweakOption {
    TweakOption {
        label: label.to_string(),
        registry_changes,
        service_changes: Vec::new(),
        scheduler_changes: Vec::new(),
        hosts_changes: Vec::new(),
        firewall_changes: Vec::new(),
        pre_commands: Vec::new(),
        post_commands: Vec::new(),
        pre_powershell: Vec::new(),
        post_powershell: Vec::new(),
        registry_missing_is_match: false,
        service_missing_is_match: false,
        scheduler_missing_is_match: false,
    }
}

fn tweak(id: &str, options: Vec<TweakOption>) -> TweakDefinition {
    TweakDefinition {
        id: id.to_string(),
        name: format!("Fixture {}", id),
        description: "round-trip fixture".to_string(),
        info: None,
        risk_level: RiskLevel::Low,
        requires_admin: false,
        requires_system: false,
        requires_ti: false,
        requires_reboot: false,
        force_dropdown: false,
        options,
        category_id: "test".to_string(),
    }
}

/// Applies an option the way the real engine does, so these tests exercise the
/// production write path rather than a parallel one.
fn apply(t: &TweakDefinition, index: usize) {
    crate::commands::tweaks::helpers::apply_all_changes_atomically(t, &t.options[index], 11)
        .unwrap_or_else(|e| panic!("apply of option {} failed: {}", index, e));
}

#[test]
fn a_value_that_did_not_exist_before_is_removed_again_by_revert() {
    let s = Scratch::new("rt_absent_value");
    let t = tweak(
        &s.tweak_id,
        vec![
            option("On", vec![dword_change(&s.key, "Flag", 1)]),
            option("Off", vec![dword_change(&s.key, "Flag", 0)]),
        ],
    );

    // Precondition: the value genuinely does not exist yet.
    assert!(
        !registry_service::value_exists(&RegistryHive::Hkcu, &s.key, "Flag").unwrap_or(false),
        "scratch key was not clean"
    );

    let snapshot = capture_snapshot(&t, 0, 11, None).expect("capture");
    save_snapshot(&snapshot).expect("save");
    apply(&t, 0);

    assert!(
        registry_service::value_exists(&RegistryHive::Hkcu, &s.key, "Flag").unwrap(),
        "apply did not write the value"
    );

    let result = restore_from_snapshot(&snapshot).expect("restore");
    assert!(
        result.success,
        "restore reported failures: {:?}",
        result.failures
    );

    // The value did not exist before, so restoring must DELETE it rather than
    // write a zero. Writing a default here would permanently add a registry value
    // the machine never had.
    assert!(
        !registry_service::value_exists(&RegistryHive::Hkcu, &s.key, "Flag").unwrap_or(false),
        "revert recreated a value that never existed before the tweak"
    );
}

#[test]
fn an_existing_value_is_restored_to_its_original_contents() {
    let s = Scratch::new("rt_existing_value");
    let t = tweak(
        &s.tweak_id,
        vec![
            option("On", vec![dword_change(&s.key, "Flag", 1)]),
            option("Off", vec![dword_change(&s.key, "Flag", 0)]),
        ],
    );

    // Pre-existing state the user had before we ever touched the machine.
    registry_service::set_dword(&RegistryHive::Hkcu, &s.key, "Flag", 7).expect("seed");

    let snapshot = capture_snapshot(&t, 0, 11, None).expect("capture");
    save_snapshot(&snapshot).expect("save");
    apply(&t, 0);
    assert_eq!(
        registry_service::read_dword(&RegistryHive::Hkcu, &s.key, "Flag").unwrap(),
        Some(1)
    );

    let result = restore_from_snapshot(&snapshot).expect("restore");
    assert!(
        result.success,
        "restore reported failures: {:?}",
        result.failures
    );

    assert_eq!(
        registry_service::read_dword(&RegistryHive::Hkcu, &s.key, "Flag").unwrap(),
        Some(7),
        "revert did not restore the user's original value"
    );
}

#[test]
fn detection_reports_the_option_that_was_actually_applied() {
    let s = Scratch::new("rt_detect");
    let t = tweak(
        &s.tweak_id,
        vec![
            option("On", vec![dword_change(&s.key, "Flag", 1)]),
            option("Off", vec![dword_change(&s.key, "Flag", 0)]),
        ],
    );

    // Nothing written yet: matches neither option, i.e. System Default.
    let state = detect_tweak_state(&t, 11).expect("detect");
    assert_eq!(
        state.current_option_index, None,
        "an untouched key must not match any option"
    );

    apply(&t, 1);
    assert_eq!(
        detect_tweak_state(&t, 11).unwrap().current_option_index,
        Some(1)
    );

    apply(&t, 0);
    assert_eq!(
        detect_tweak_state(&t, 11).unwrap().current_option_index,
        Some(0)
    );
}

#[test]
fn switching_options_leaves_the_original_snapshot_intact() {
    let s = Scratch::new("rt_switch_preserves_original");
    let t = tweak(
        &s.tweak_id,
        vec![
            option("On", vec![dword_change(&s.key, "Flag", 1)]),
            option("Off", vec![dword_change(&s.key, "Flag", 0)]),
        ],
    );

    registry_service::set_dword(&RegistryHive::Hkcu, &s.key, "Flag", 42).expect("seed");

    // First apply captures the ORIGINAL state.
    let original = capture_snapshot(&t, 0, 11, None).expect("capture");
    save_snapshot(&original).expect("save");
    apply(&t, 0);

    // Switching options must not replace the stored original with the tweaked value,
    // or revert would return the machine to option 0 rather than to 42 (ADR-0002).
    apply(&t, 1);

    let result = restore_from_snapshot(&original).expect("restore");
    assert!(
        result.success,
        "restore reported failures: {:?}",
        result.failures
    );
    assert_eq!(
        registry_service::read_dword(&RegistryHive::Hkcu, &s.key, "Flag").unwrap(),
        Some(42),
        "revert after an option switch did not reach the pre-tweak value"
    );
}

#[test]
fn a_key_created_by_the_tweak_is_removed_by_revert() {
    let s = Scratch::new("rt_created_key");
    // A subkey that does not exist at all until the tweak creates it.
    let nested = format!("{}\\Nested", s.key);
    let t = tweak(
        &s.tweak_id,
        vec![
            option("On", vec![dword_change(&nested, "Flag", 1)]),
            option("Off", vec![dword_change(&nested, "Flag", 0)]),
        ],
    );

    let snapshot = capture_snapshot(&t, 0, 11, None).expect("capture");
    save_snapshot(&snapshot).expect("save");
    apply(&t, 0);
    assert!(registry_service::value_exists(&RegistryHive::Hkcu, &nested, "Flag").unwrap());

    let result = restore_from_snapshot(&snapshot).expect("restore");
    assert!(
        result.success,
        "restore reported failures: {:?}",
        result.failures
    );
    assert!(
        !registry_service::value_exists(&RegistryHive::Hkcu, &nested, "Flag").unwrap_or(false),
        "revert left behind a value in a key the tweak created"
    );
}
