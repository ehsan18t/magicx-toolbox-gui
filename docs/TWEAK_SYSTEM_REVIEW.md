# Tweak System Review

Audit of the tweak system (Rust backend, `build.rs` compile-time validator, YAML corpus) against `docs/TWEAK_SYSTEM.md` + `docs/TWEAK_AUTHORING.md` as spec.

Method: 5 partitioned finder passes (code-defect / spec-conformance / design-critique lenses), each finding then handed to an independent adversarial verifier that defaulted to REFUTED and had to confirm against the code.

**47 findings — 41 confirmed, 5 plausible, 1 refuted.**

| Verdict | critical | high | medium | low |
|---|---|---|---|---|
| CONFIRMED | 1 | 14 | 18 | 8 |
| PLAUSIBLE | 0 | 0 | 1 | 4 |
| REFUTED | 0 | 0 | 0 | 1 |

---

## Index

| # | Sev | Verdict | Finding | Anchor |
|---|---|---|---|---|
| 1 | critical | CONFIRMED | First-apply failure discards the rollback result and deletes the snapshot unconditionally, permanently destroy | `src-tauri/src/commands/tweaks/apply.rs:160` |
| 2 | high | CONFIRMED | build.rs non-empty-option check counts skip_validation/version-filtered changes, shipping 8 tweaks whose statu | `src-tauri/build.rs:1044` |
| 3 | high | CONFIRMED | capture::read_registry_value converts access-denied/read errors into existed:false, so revert DELETES registry | `src-tauri/src/services/backup/capture.rs:588` |
| 4 | high | CONFIRMED | create_key / delete_key registry actions can never be reverted: key-level snapshots are misinterpreted as valu | `src-tauri/src/services/backup/restore.rs:170` |
| 5 | high | CONFIRMED | Restore paths that provably could not restore return Ok(()), so revert reports success and deletes the snapsho | `src-tauri/src/services/backup/restore.rs:388` |
| 6 | high | CONFIRMED | Any registry read failure is reported as "value missing", and *_missing_is_match then turns it into an inferre | `src-tauri/src/services/backup/detection.rs:234` |
| 7 | high | CONFIRMED | delete_key with a trailing/lone backslash calls RegDeleteTreeW(parent, NULL) and wipes the parent key's entire | `src-tauri/src/services/registry_service.rs:458` |
| 8 | high | CONFIRMED | run_schtasks_as_ti/as_system return a raw exit code that the caller discards, so elevated scheduler failures a | `src-tauri/src/services/elevation/ti_elevation.rs:468` |
| 9 | high | CONFIRMED | firewall_service::rule_exists ignores netsh's exit status and matches an English-only string, so create_firewa | `src-tauri/src/services/firewall_service.rs:25` |
| 10 | high | CONFIRMED | Service and task state detection parse English-only labels and values from sc.exe and schtasks.exe output | `src-tauri/src/services/scheduler_service.rs:44` |
| 11 | high | CONFIRMED | Each of the five effect services - plus a second, parallel elevated implementation of three of them - invents  | `src-tauri/src/services/elevation/service_ops.rs:30` |
| 12 | high | CONFIRMED | Seven tweaks mark every registry change skip_validation in every option, making their status permanently undet | `src-tauri/tweaks/services.yaml:176` |
| 13 | high | CONFIRMED | windows_update_mode disables UpdateOrchestrator tasks in one option and no other option re-enables them | `src-tauri/tweaks/windows_update.yaml:270` |
| 14 | high | CONFIRMED | Two gaming tweaks each overwrite the whole DirectXUserGlobalSettings composite string, silently erasing each o | `src-tauri/tweaks/gaming.yaml:790` |
| 15 | high | CONFIRMED | legacy_network_protocols silently re-enables SMBv1 in two options while its name, description and info mention | `src-tauri/tweaks/network.yaml:243` |
| 16 | medium | CONFIRMED | `aliases` receives zero build-time validation (no format, no uniqueness, no collision check) while alias resol | `src-tauri/build.rs:1086` |
| 17 | medium | CONFIRMED | models/tweak.rs carries a fully dead second copy of the YAML pipeline including a duplicated elevation-inferen | `src-tauri/src/models/tweak.rs:598` |
| 18 | medium | CONFIRMED | TweakSnapshot has no schema version and its nested structs have no serde defaults, so any future field additio | `src-tauri/src/models/tweak_snapshot.rs:70` |
| 19 | medium | CONFIRMED | Snapshot writes are truncate-in-place with no temp+rename and no fsync; a crash mid-write leaves an unparseabl | `src-tauri/src/services/backup/storage.rs:50` |
| 20 | medium | CONFIRMED | Nothing serializes apply_tweak, so two concurrent applies of the same tweak can persist the already-tweaked st | `src-tauri/src/commands/tweaks/apply.rs:88` |
| 21 | medium | CONFIRMED | inspection.rs compares registry values with raw JSON equality, so REG_BINARY authored as a hex string always r | `src-tauri/src/services/backup/inspection.rs:156` |
| 22 | medium | CONFIRMED | inspection.rs never reads registry_/service_/scheduler_missing_is_match, so it contradicts detection on every  | `src-tauri/src/services/backup/inspection.rs:120` |
| 23 | medium | CONFIRMED | calculate_overall_match returns true for an option with zero validatable items, so matched_option_index points | `src-tauri/src/services/backup/inspection.rs:98` |
| 24 | medium | CONFIRMED | A shipping tweak whose every change is skip_validation is permanently undetectable and no validation prevents  | `src-tauri/src/services/backup/detection.rs:125` |
| 25 | medium | CONFIRMED | TweakStatus.is_applied is documented as "has a snapshot" but is computed as current_option_index == Some(0) | `src-tauri/src/commands/tweaks/query.rs:102` |
| 26 | medium | CONFIRMED | set_registry_value_as_system double-quotes and %-mangles the value, so REG_SZ/REG_EXPAND_SZ values with a spac | `src-tauri/src/services/elevation/system_elevation.rs:194` |
| 27 | medium | CONFIRMED | run_powershell_as_system/as_ti escape quotes with a C-runtime escape that cmd.exe does not honor, letting a sc | `src-tauri/src/services/elevation/ti_elevation.rs:92` |
| 28 | medium | CONFIRMED | remove_hosts_entry deletes the entire hosts line for a multi-hostname entry, and revert restores only the sing | `src-tauri/src/services/hosts_service.rs:186` |
| 29 | medium | CONFIRMED | read_hosts_file and remove_hosts_entry parse the domain field differently, so entry_exists false-negatives cau | `src-tauri/src/services/hosts_service.rs:55` |
| 30 | medium | CONFIRMED | 98 "(Default)"-labelled options write explicit Group Policy values that stock Windows ships absent, leaving th | `src-tauri/tweaks/windows_update.yaml:187` |
| 31 | medium | CONFIRMED | ipv6_transition_mode consists solely of pre_commands, so it is never snapshotted, never detectable and never r | `src-tauri/tweaks/network.yaml:65` |
| 32 | medium | CONFIRMED | mouse_input_mode option 1 and option 3 declare byte-identical state, making option 3 permanently unreachable | `src-tauri/tweaks/gaming.yaml:477` |
| 33 | medium | CONFIRMED | Seven registry values are owned by two tweaks each, and four of those pairs disagree on the stock default | `src-tauri/tweaks/privacy.yaml:151` |
| 34 | low | CONFIRMED | REG_BINARY hex strings are accepted unconditionally at build time but strictly parsed at runtime, so a malform | `src-tauri/build.rs:828` |
| 35 | low | CONFIRMED | `aliases` is a live, profile-migration-critical YAML field but is absent from the authoring spec's tweak field | `docs/TWEAK_AUTHORING.md:189` |
| 36 | low | CONFIRMED | Docs mark `requires_reboot` as Required while simultaneously giving it a default, and the code makes it option | `docs/TWEAK_AUTHORING.md:197` |
| 37 | low | CONFIRMED | TWEAK_SYSTEM.md says rollback is per-phase, TWEAK_AUTHORING.md says rollback is cross-phase; the code is cross | `docs/TWEAK_SYSTEM.md:78` |
| 38 | low | CONFIRMED | batch.rs reports the number of individual failure entries as the number of failed tweaks | `src-tauri/src/commands/tweaks/batch.rs:79` |
| 39 | low | CONFIRMED | Docs say snapshots live in the app data directory; the code writes them next to the executable | `src-tauri/src/services/backup/storage.rs:19` |
| 40 | low | CONFIRMED | The hosts file is truncated and rewritten in place, converting the whole file's line endings and losing all en | `src-tauri/src/services/hosts_service.rs:201` |
| 41 | low | CONFIRMED | hosts file reads use read_to_string, so a non-UTF-8 hosts file makes every hosts operation fail, and the UTF-8 | `src-tauri/src/services/hosts_service.rs:35` |
| 42 | medium | PLAUSIBLE | schtasks query failures are turned into "task not found", which ignore_not_found/missing_is_match/Delete then  | `src-tauri/src/services/backup/detection.rs:384` |
| 43 | low | PLAUSIBLE | rollback_registry_operations hardcodes use_system=false, so rolling back a failed restore of a requires_system | `src-tauri/src/services/backup/restore.rs:424` |
| 44 | low | PLAUSIBLE | Firewall detection treats any netsh failure or non-English output as "rule exists", so Create-op firewall chan | `src-tauri/src/services/backup/detection.rs:487` |
| 45 | low | PLAUSIBLE | Elevated start/stop service treat net.exe exit code 2 as success, and net returns 2 for essentially every fail | `src-tauri/src/services/elevation/service_ops.rs:89` |
| 46 | low | PLAUSIBLE | disable_wdigest is risk_level low but offers an option that re-enables cleartext credential storage in LSASS | `src-tauri/tweaks/security.yaml:1013` |
| 47 | low | REFUTED | list_tasks_in_folder issues a schtasks query form that always fails, and swallows the failure as "empty folder | `src-tauri/src/services/scheduler_service.rs:204` |

---

## Apply / Rollback / Snapshot lifecycle

### [CRITICAL · CONFIRMED] First-apply failure discards the rollback result and deletes the snapshot unconditionally, permanently destroying the user's original state

`src-tauri/src/commands/tweaks/apply.rs:160` — lens: code-defect

**What is wrong.** On first-apply failure, apply_tweak ignores both the Err and the success:false result of restore_from_snapshot and then deletes the snapshot regardless, so a failed/partial rollback leaves the machine half-tweaked with no snapshot and no way to revert, and the user is never told rollback failed.

**Failing scenario.** Tweak T (first apply, no prior snapshot) with registry_changes + service_changes + firewall_changes. Registry sets succeed, service startup is set to Disabled, then firewall rule creation fails -> apply_all_changes_atomically returns Err. Rollback path runs restore_from_snapshot: Phase 1 registry restores OK, Phase 2 restore_service_state fails (service now locked / protected) -> restore.rs:101 pushes a failure -> returns Ok(RestoreResult{success:false}). apply.rs:162 `let _ =` throws that away, apply.rs:164 deletes the snapshot. Result: service left Disabled, original startup type gone forever, snapshot file gone, revert_tweak now returns "No snapshot found for this tweak". UI shows only "firewall failed" with zero indication that the machine is half-modified. Contrast revert_tweak (apply.rs:270) which correctly KEEPS the snapshot on partial failure.

**Evidence.**

```
src-tauri/src/commands/tweaks/apply.rs:156-165
        if let Some(ref current_state) = pre_apply_state {
            log::warn!("Rolling back to previous option state (switching options failed)...");
            let _ = backup_service::restore_from_snapshot(current_state);
        } else {
            if let Some(snapshot) = backup_service::load_snapshot(&tweak_id)? {
                log::warn!("Rolling back ALL changes to original state (first apply failed)...");
                let _ = backup_service::restore_from_snapshot(&snapshot);
            }
            backup_service::delete_snapshot(&tweak_id)?;
        }

(compare src-tauri/src/commands/tweaks/apply.rs:268-271)
    // Only delete snapshot if ALL operations succeeded
    // This allows the user to retry the revert if some operations failed
    if restore_result.success {
        backup_service::delete_snapshot(&tweak_id)?;
```

**Fix.** Bind the result: `let rr = backup_service::restore_from_snapshot(&snapshot);` Only call delete_snapshot when `matches!(rr, Ok(r) if r.success)`. Otherwise keep the snapshot and return an error/TweakResult that surfaces both the apply error AND the rollback failures to the UI (same shape revert_tweak already uses). Also stop using `?` on delete_snapshot at line 164 — it currently replaces the real apply error `e` with a file-deletion error.

<details><summary>Verifier reasoning</summary>

Verified verbatim at src-tauri/src/commands/tweaks/apply.rs:156-165: the first-apply branch does `let _ = backup_service::restore_from_snapshot(&snapshot);` then `backup_service::delete_snapshot(&tweak_id)?;` unconditionally. I tried to refute three ways and all failed. (1) 'The inner rollback already handles it' — helpers.rs:218-254 shows apply_all_changes_atomically only rolls back registry in-phase (apply_registry_changes:483-507); once registry commits and step 2/3/4/5 fails it returns Err with services/tasks/hosts/firewall already mutated, so the outer snapshot restore is the ONLY recovery. (2) 'restore_from_snapshot can't actually fail softly' — restore.rs:97-130 pushes failures for service/scheduler/hosts/firewall and restore.rs:132 `let success = failures.is_empty();` returns Ok(RestoreResult{success:false}); the Phase-1 registry path returns a hard Err at restore.rs:87 and aborts phases 2-5 entirely. Both outcomes are swallowed by `let _ =`. (3) 'Not reachable with shipped tweaks' — src-tauri/tweaks/windows_update.yaml:166 is requires_ti with service_changes (wuauserv, UsoSvc) plus scheduler_changes, so a scheduler-step failure after services were changed lands exactly here. Contrast revert_tweak at apply.rs:270 which correctly gates delete_snapshot on restore_result.success, proving the authors knew the right shape. The secondary claim also holds: `delete_snapshot(&tweak_id)?` at line 164 returns the file-deletion error instead of the real apply error `e`. Net effect: half-applied machine, snapshot destroyed, revert_tweak (apply.rs:241) then answers 'No snapshot found for this tweak'. Critical is correct — this is the state-loss class the brief weights highest.

</details>

---

### [HIGH · CONFIRMED] capture::read_registry_value converts access-denied/read errors into existed:false, so revert DELETES registry values that existed

`src-tauri/src/services/backup/capture.rs:588` — lens: code-defect — *severity adjusted by verifier: critical → high*

**What is wrong.** Any registry read error other than RegistryKeyNotFound (notably Error::RegistryAccessDenied) is silently mapped to `(None, false)`, fabricating an "this value did not exist" snapshot record; execute_registry_restore then deletes the real value on revert instead of restoring it.

**Failing scenario.** A requires_ti/requires_system tweak targets a key whose ACL denies read to the app's own token (e.g. HKLM\SOFTWARE\Microsoft\Windows Defender\* under Tamper Protection). registry_service::read_dword hits `open_subkey_with_flags(..., KEY_READ)` failing with something other than NotFound -> returns Err(Error::RegistryAccessDenied) (registry_service.rs:32-38). capture.rs:588 swallows it -> RegistrySnapshot{existed:false, value:None}. Apply succeeds via TI elevation (which CAN write). User clicks Revert -> restore.rs:170 `if !op.existed` -> delete_registry_value_as_system(...) -> the pre-existing value is DELETED rather than restored to its original data, and revert reports full success. Original value is unrecoverable.

**Evidence.**

```
src-tauri/src/services/backup/capture.rs:584-598
    match result {
        Ok(Some(value)) => Ok((Some(value), true)),
        Ok(None) => Ok((None, false)),
        Err(Error::RegistryKeyNotFound(_)) => Ok((None, false)),
        Err(e) => {
            log::warn!(
                "Failed to read {}\\{}\\{}: {}",
                hive.as_str(), key, value_name, e
            );
            Ok((None, false))
        }
    }

src-tauri/src/services/registry_service.rs:32-38
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                Error::RegistryKeyNotFound(format!("{}\\{}", key_path, value_name))
            } else {
                Error::RegistryAccessDenied(e.to_string())
            }
        })?;

src-tauri/src/services/backup/restore.rs:169-171
fn execute_registry_restore(op: &RegistryRestoreOp, use_system: bool) -> Result<(), Error> {
    if !op.existed {
        // Value didn't exist - delete it
```

**Fix.** Add a third state to the snapshot record (e.g. `readable: bool` or make `existed` an enum Absent|Present|Unknown). On a non-NotFound read error, either propagate the error so capture_snapshot fails loudly before anything is applied, or record Unknown and have execute_registry_restore skip-and-report-failure for Unknown rather than deleting. Retry the read via the elevated path when tweak.requires_system/requires_ti before giving up.

**Verifier correction.** The mechanism is exactly as claimed, but the finder's specific trigger (Defender/Tamper-Protection ACL denial) is NOT exercised by any shipped tweak — the only requires_ti/requires_system tweaks target HKLM\Software\Policies\... and HKLM\SOFTWARE\Policies\Microsoft\Dsh, which are user-readable. The demonstrably reachable trigger is a value-TYPE mismatch: winreg's get_value::<u32> on a value stored as REG_SZ returns io::ErrorKind::InvalidData, which registry_service.rs:47-51 maps to Error::RegistryOperation (not RegistryKeyNotFound), which capture.rs:588 then swallows into existed:false. Severity high, not critical: the loss is a single registry value, not a wedged machine.

<details><summary>Verifier reasoning</summary>

Anchors verified verbatim. capture.rs:584-598 has `Err(e) => { log::warn!(...); Ok((None, false)) }` catching every non-NotFound error. registry_service.rs:32-38 confirms open_subkey_with_flags failures other than NotFound become Error::RegistryAccessDenied, and read_dword's get_value arm (registry_service.rs:47-51) makes type mismatches Error::RegistryOperation — both hit the swallowing arm. restore.rs:170-193 confirms `if !op.existed` deletes the value (via trusted_installer::delete_registry_value_as_system when use_system). I attempted to refute by looking for a guard: there is none — capture_snapshot (capture.rs:71-79) only logs the fabricated `existed` flag, and no caller re-validates. I also attempted to refute on reachability; the ACL example is unproven for shipped YAML, but the type-mismatch path needs only a declared REG_DWORD over an existing REG_SZ, and the system is explicitly YAML-authorable per docs/TWEAK_AUTHORING.md. The finding survives with the trigger corrected.

</details>

---

### [HIGH · CONFIRMED] create_key / delete_key registry actions can never be reverted: key-level snapshots are misinterpreted as value operations

`src-tauri/src/services/backup/restore.rs:170` — lens: code-defect

**What is wrong.** capture stores key-level actions with value_name = "" and value/value_type = None; execute_registry_restore only knows how to write or delete a VALUE, so a created key is never deleted (it deletes the key's default value instead) and a recursively-deleted key is never recreated — while revert still reports success.

**Failing scenario.** (a) create_key: option has `{action: create_key, hive: HKCU, key: "Software\\Classes\\CLSID\\{...}\\InprocServer32"}`. capture.rs:181-193 records existed=false, value_name="". Apply creates the key. Revert -> restore.rs:170 `!op.existed` -> `registry_service::delete_value(&op.hive, &op.key, "")` deletes the key's DEFAULT VALUE; the key itself survives. The tweak's effect persists, revert says "Reverted", snapshot deleted.
(b) delete_key: option has `{action: delete_key, key: "Software\\Classes\\..."}`. capture.rs:167-179 records existed=true, value=None, value_type=None. Apply calls registry_service::delete_key -> delete_subkey_all (recursive, whole subtree gone). Revert -> restore.rs:210-218 falls to the final `else`, logs "existed but no value/type", returns Ok(()) -> RestoreResult.success=true -> apply.rs:271 deletes the snapshot. The subtree is permanently gone and the user is told the revert succeeded. Note apply_registry_changes' in-phase rollback DOES have a RecreateKey arm (helpers.rs:497-500), so the snapshot restore path is strictly weaker than the in-phase one.

**Evidence.**

```
src-tauri/src/services/backup/capture.rs:167-179
                RegistryAction::DeleteKey => {
                    // For DeleteKey, just note if the key existed
                    let existed = registry_service::key_exists(&change.hive, &change.key).unwrap_or(false);
                    Ok(RegistrySnapshot {
                        ...
                        value_name: String::new(), // Key-level operation, no specific value
                        value_type: None,
                        value: None,
                        existed,
                    })

src-tauri/src/services/backup/restore.rs:186
            match registry_service::delete_value(&op.hive, &op.key, &op.value_name) {

src-tauri/src/services/backup/restore.rs:210-218
    } else {
        log::warn!(
            "Skipping restore for {}\\{}\\{}: existed but no value/type",
            op.hive.as_str(), op.key, op.value_name
        );
        Ok(())
    }
```

**Fix.** Add an explicit `is_key_level: bool` (or a separate `key_snapshots` vec) to RegistrySnapshot. In execute_registry_restore, branch on it: key-level + !existed -> registry_service::delete_key; key-level + existed -> registry_service::create_key and return an explicit failure string noting subkey/value contents were not preserved. Longer term, for delete_key capture the full subtree (export .reg / recursive enumeration) or reject delete_key on keys with subkeys at build.rs validation time, since the docs (TWEAK_AUTHORING.md:421) advertise it as recursive with no revert caveat.

**Verifier correction.** Half (b), delete_key, is reachable with a SHIPPED tweak: src-tauri/tweaks/ui.yaml:78-83 ('Modern Menu (Default)' option of the classic-context-menu tweak) uses `action: delete_key` on HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae9a9}. Half (a), create_key, is latent — no shipped YAML uses create_key — but the action is supported and documented.

<details><summary>Verifier reasoning</summary>

All three anchors verified verbatim. capture.rs:167-179 (DeleteKey) and capture.rs:181-193 (CreateKey) both write value_name: String::new(), value_type: None, value: None. execute_registry_restore (restore.rs:169-218) has exactly three arms and none is key-aware: !existed -> delete_value with the empty name (deletes the default value, never the key); existed && Some(value) && Some(type) -> write; else -> restore.rs:210-217 warn + Ok(()). The 'strictly weaker than in-phase rollback' claim also checks out: helpers.rs:497-500 has a RecreateKey arm and helpers.rs:501-503 a DeleteKey arm that the snapshot path lacks. Refutation attempt on reachability: for the ui.yaml tweak the DeleteKey record only lands in a persisted snapshot on a FIRST apply of the delete option while the key exists — reachable when the user set the classic menu outside this app (or previously reverted), since detect_tweak_state then reports option 0 as current and apply.rs:64 does not short-circuit. Revert then hits restore.rs:210-217, returns Ok, failures stays empty, restore.rs:132 sets success=true, apply.rs:271 deletes the snapshot, UI says 'Reverted'. Nothing recreated. High is correct.

</details>

---

### [HIGH · CONFIRMED] Restore paths that provably could not restore return Ok(()), so revert reports success and deletes the snapshot

`src-tauri/src/services/backup/restore.rs:388` — lens: code-defect

**What is wrong.** Four restore branches log a warning and return Ok(()) when they know they failed to restore state; because RestoreResult.success is `failures.is_empty()`, revert_tweak treats these as full success and deletes the snapshot, permanently discarding the original state.

**Failing scenario.** (1) Firewall: option has `{operation: delete, name: "Core Networking - DHCP (In)"}`. capture records existed=true. Apply deletes the rule. Revert -> restore.rs:387-393 sees existed=true, currently_exists=false, logs "cannot recreate without original rule config", returns Ok(()) -> no failure pushed -> success=true -> apply.rs:271 deletes the snapshot. Inbound DHCP rule is gone forever and the UI says "Reverted: <tweak>".
(2) Service: requires_ti tweak disabling WinDefend. capture_service_state (capture.rs:534-537) gets startup_type=None because service_control::get_service_startup_type's unelevated `reg query` on the protected service key fails, so it stores the sentinel string "unknown". Apply disables the service as TI. Revert -> restore.rs:251-260 matches the `_` arm, logs "Unknown startup type: unknown", returns Ok(()) -> success=true -> snapshot deleted. WinDefend stays Disabled forever, reported as successfully reverted.
(3) restore.rs:211-217 (registry existed-but-no-value, see delete_key finding) and (4) restore.rs:344-350 (unknown scheduler state) behave identically.

**Evidence.**

```
src-tauri/src/services/backup/restore.rs:383-394
fn restore_firewall_state(snapshot: &FirewallSnapshot) -> Result<(), Error> {
    if snapshot.existed {
        // Rule existed before - we can't fully recreate it without storing the full rule config
        // Just log a warning if it's missing now
        let currently_exists = firewall_service::rule_exists(&snapshot.name)?;
        if !currently_exists {
            log::warn!(
                "Firewall rule '{}' existed before but is now missing; cannot recreate without original rule config",
                snapshot.name
            );
        }
    } else {

src-tauri/src/services/backup/restore.rs:256-261
        _ => {
            log::warn!("Unknown startup type: {}", snapshot.startup_type);
            return Ok(());
        }
    };

src-tauri/src/services/backup/capture.rs:534-537
    let startup_type = status
        .startup_type
        .map(|t| format!("{:?}", t).to_lowercase())
        .unwrap_or_else(|| "unknown".to_string());

src-tauri/src/services/backup/restore.rs:132
    let success = failures.is_empty();
```

**Fix.** Make every "cannot restore" branch return Err(...) (or have the caller push a failure string) so RestoreResult.success goes false and apply.rs:270 keeps the snapshot for retry. For firewall, capture the full rule config (netsh advfirewall show rule name=X verbose) at snapshot time so delete is actually reversible. For services, refuse to capture (fail the apply) when startup_type cannot be read, rather than persisting a "unknown" sentinel; escalate the read through the SYSTEM/TI path when the tweak declares those.

**Verifier correction.** Sub-claims (3) registry existed-but-no-value and (4) unknown scheduler state are demonstrably reachable; (4) is the strongest and the finder understated it. get_task_state parses with `line.starts_with("Status:")` (scheduler_service.rs:91) and falls back to TaskState::Unknown("Could not parse state") at scheduler_service.rs:101 — on any non-English Windows, or for a task reporting 'Queued', EVERY captured task state becomes Unknown, so reverting src-tauri/tweaks/windows_update.yaml:270-274 ('Disabled (Complete)', which disables UpdateOrchestrator tasks by pattern) silently re-enables nothing and still deletes the snapshot. Sub-claim (1) firewall is latent: no shipped YAML has firewall_changes, and docs/TWEAK_SYSTEM.md:62 already documents 'Delete rule (create N/A)'. Sub-claim (2) service 'unknown' is not demonstrated for shipped tweaks (wuauserv/UsoSvc service keys are readable) though the code path is exactly as described.

<details><summary>Verifier reasoning</summary>

All four anchors verified verbatim: restore.rs:383-394 (firewall existed-but-missing -> warn, Ok), restore.rs:256-261 (`_ => { log::warn!(...); return Ok(()); }`), restore.rs:210-217, restore.rs:344-350. restore.rs:132 `let success = failures.is_empty();` plus apply.rs:270-271 confirm the consequence chain: no failure pushed -> success -> snapshot deleted -> TweakResult{success:true, message:'Reverted: ...'}. capture.rs:534-537 confirms the 'unknown' sentinel, and service_control.rs:90 `let startup_type = get_service_startup_type(service_name).ok();` confirms any read error becomes None. Refutation attempts: I looked for a caller that re-checks post-restore state before deleting the snapshot (none — apply.rs:270 trusts the boolean) and for a docs line permitting silent success (docs/TWEAK_AUTHORING.md:933 claims firewall_changes 'Rolls back everything from snapshot', which contradicts, not excuses, the behavior). High confirmed.

</details>

---

### [MEDIUM · CONFIRMED] Snapshot writes are truncate-in-place with no temp+rename and no fsync; a crash mid-write leaves an unparseable snapshot that permanently wedges the tweak

`src-tauri/src/services/backup/storage.rs:50` — lens: code-defect — *severity adjusted by verifier: high → medium*

**What is wrong.** save_snapshot truncates the file (File::create) BEFORE acquiring the lock and then writes in place; update_snapshot_metadata does set_len(0) then write in place. Any interruption between truncate and write leaves a 0-byte or partial JSON file, and load_snapshot/snapshot_exists/delete_snapshot take no lock at all, so readers can observe that window.

**Failing scenario.** (1) User switches option on tweak T (snapshot already exists). update_snapshot_metadata calls file.set_len(0) at storage.rs:117, then the machine loses power / the user force-quits before write_all at :125. On next launch: snapshot_exists("T") == true (storage.rs:157 only checks path.exists()), so apply.rs:88 sets is_switching_options=true and the true original is NEVER re-captured; revert_tweak's load_snapshot at apply.rs:240 hits serde_json::from_str on "" -> Err(BackupFailed("Failed to parse snapshot")). The tweak is permanently unrevertable and un-recapturable; the only escape is manually deleting the file. detect_tweak_state (detection.rs:61) also propagates the parse error, so the whole status query for that tweak now errors on every refresh.
(2) Even without a crash: detect_tweak_state -> load_snapshot (storage.rs:146 `fs::read_to_string`, no lock) runs concurrently with update_snapshot_metadata's set_len(0)..write_all window and reads a truncated file -> parse error surfaced to the UI.

**Evidence.**

```
src-tauri/src/services/backup/storage.rs:49-59
    // Create/open file and acquire exclusive lock
    let file = File::create(&path)
        .map_err(|e| Error::BackupFailed(format!("Failed to create snapshot file: {}", e)))?;

    file.lock_exclusive()
        .map_err(|e| Error::BackupFailed(format!("Failed to acquire file lock: {}", e)))?;

src-tauri/src/services/backup/storage.rs:116-126
    // Truncate and rewrite while holding lock
    file.set_len(0)
        .map_err(|e| Error::BackupFailed(format!("Failed to truncate snapshot file: {}", e)))?;
    ...
    file.write_all(json.as_bytes())

src-tauri/src/services/backup/storage.rs:146-150 (no lock on the read path)
    let content = fs::read_to_string(&path)
        .map_err(|e| Error::BackupFailed(format!("Failed to read snapshot: {}", e)))?;
    let snapshot: TweakSnapshot = serde_json::from_str(&content)
```

**Fix.** Write to `<tweak_id>.json.tmp`, `file.sync_all()`, then `fs::rename` over the real path (rename is atomic on NTFS via ReplaceFile/MoveFileEx). Never truncate the live file. Take a shared lock in load_snapshot and hold the exclusive lock across the whole read-modify-write in update_snapshot_metadata (it already does the latter, but the lock is worthless while readers don't take one). Additionally, treat a snapshot that fails to parse as "corrupt" explicitly rather than letting snapshot_exists() return true for it — otherwise apply.rs:88 mistakes a corrupt file for a valid prior snapshot.

**Verifier correction.** The structural claim is right but the stated consequence chain is wrong in one step and the second mechanism does not hold. (a) After a corrupt/0-byte snapshot, apply_tweak never reaches the snapshot_exists check at apply.rs:88 — it dies earlier at apply.rs:63 `backup_service::detect_tweak_state(&tweak, version)?`, because detection.rs:61 `load_snapshot(&tweak.id)?` propagates the serde parse error. So the tweak is wedged for BOTH apply and revert (worse than stated), rather than being silently mis-treated as an option switch. (b) The 'concurrent reader sees the truncation window' mechanism is not supported: there is no polling status call in the frontend (src/lib/api/tweaks.ts has no interval) and batch_apply_tweaks (src-tauri/src/commands/tweaks/batch.rs:43-44) is a sequential await loop, so no concurrent detect during the write window was demonstrated. A real second trigger does exist though: the snapshots dir lives next to the exe (storage.rs:19-25), and File::create truncates BEFORE lock_exclusive, so a second portable instance truncates a file the first instance holds locked.

<details><summary>Verifier reasoning</summary>

Anchors verified verbatim: storage.rs:50-58 (File::create — write+create+truncate — then lock_exclusive, then write_all, no sync_all, no temp+rename), storage.rs:117-126 (set_len(0), seek, write_all in place), storage.rs:146-150 (fs::read_to_string with no lock), storage.rs:157-159 (snapshot_exists is bare path.exists()). I confirmed validate_all_snapshots (detection.rs:557-561) logs and skips on load error, so startup cleanup will never clear the corrupt file. Refutation succeeded against the concurrency mechanism and against the apply.rs:88 step of the narrative, but failed against the core: there is genuinely no atomic write, no fsync, and no corrupt-file recovery anywhere, and the only user escape is deleting the file by hand. Downgraded to medium because the crash window is narrow (single write between truncate and write_all) even though the consequence is severe and unrecoverable in-app.

</details>

---

### [MEDIUM · CONFIRMED] Nothing serializes apply_tweak, so two concurrent applies of the same tweak can persist the already-tweaked state as the "original" snapshot

`src-tauri/src/commands/tweaks/apply.rs:88` — lens: design-critique

**What is wrong.** The snapshot lifecycle is a classic check-then-act (snapshot_exists -> capture -> save_snapshot -> mutate system) with no mutex, no per-tweak lock, and no in-process registry of in-flight applies; the file lock in storage.rs only guards the individual write, not the read-modify-write of the whole lifecycle.

**Failing scenario.** Tauri commands run on a multi-threaded tokio runtime and `apply_tweak`'s body has no await points, so two invokes can execute on different threads in parallel (a double-click on the Apply button, or a batch_apply running while the user toggles the same tweak in the UI). Interleave: T1 snapshot_exists->false; T2 snapshot_exists->false; T1 captures ORIGINAL, saves, applies all changes; T2 (scheduled slightly later) captures state -> now reads the ALREADY-TWEAKED values, calls save_snapshot which File::create-truncates T1's snapshot and writes the tweaked state as "original". Revert now restores the tweaked state and reports success. The user's true original is gone with no error anywhere. A grep for Mutex/RwLock/Semaphore across src-tauri/src/commands/tweaks and src-tauri/src/services/backup returns nothing.

**Evidence.**

```
src-tauri/src/commands/tweaks/apply.rs:88-100
    let is_switching_options = backup_service::snapshot_exists(&tweak_id)?;
    let pre_apply_state = if is_switching_options {
        ...
        Some(backup_service::capture_current_state(&tweak, version)?)
    } else {
        let original_option_index = current_state.current_option_index;
        let snapshot =
            backup_service::capture_snapshot(&tweak, option_index, version, original_option_index)?;
        backup_service::save_snapshot(&snapshot)?;

src-tauri/src/services/backup/storage.rs:157-160 (existence check is unlocked and non-atomic w.r.t. the later save)
pub fn snapshot_exists(tweak_id: &str) -> Result<bool, Error> {
    let path = get_snapshot_path(tweak_id)?;
    Ok(path.exists())
}
```

**Fix.** Introduce a per-tweak-id async lock (e.g. a `Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>` in Tauri managed state) held for the entire apply_tweak/revert_tweak body, so the snapshot_exists -> capture -> save -> mutate -> update_metadata sequence is serialized per tweak. Additionally make save_snapshot fail (create_new) rather than overwrite when the caller believes no snapshot exists, turning the race into a loud error instead of silent original-state loss.

<details><summary>Verifier reasoning</summary>

Design claim matches the code exactly. apply.rs:88 `let is_switching_options = backup_service::snapshot_exists(&tweak_id)?;` then capture_snapshot/save_snapshot at apply.rs:98-100, against storage.rs:157-160 where snapshot_exists is a bare `path.exists()`, and storage.rs:50 `File::create(&path)` which truncates before the fs4 lock is taken - so the lock guards the write, never the read-modify-write of the lifecycle. `grep -rn "Mutex|RwLock|Semaphore" src-tauri/src/commands/ src-tauri/src/services/backup/` returns zero hits, confirming no in-process serialization. apply_tweak is `pub async fn` (apply.rs:19) dispatched by Tauri onto the multi-thread tokio runtime, and there are three independent entry points into it (direct invoke via src/lib/api/tweaks.ts:93, batch.rs:44, profile.rs:224). My best refutation was the frontend guard: TweakCard.svelte:32/313-323 disables that card's buttons while loadingStore.isLoading(tweak.id), which blocks the double-click case - but it is per-card and does NOT cover profile apply (profile.rs:224 loops in the backend without touching the per-tweak loading store), so a user toggling a card while a profile import applies the same tweak reaches two concurrent apply_tweak calls. The window is narrow (both must pass snapshot_exists before either save_snapshot, or one capture must overlap the other's registry writes), but the failure mode is silent loss of the true original snapshot, and the only thing preventing it is a UI convention.

</details>

---

### [LOW · CONFIRMED] TWEAK_SYSTEM.md says rollback is per-phase, TWEAK_AUTHORING.md says rollback is cross-phase; the code is cross-phase and the per-phase labels are wrong for 4 of 5 phases

`docs/TWEAK_SYSTEM.md:78` — lens: spec-conformance — *severity adjusted by verifier: medium → low*

**What is wrong.** TWEAK_SYSTEM.md labels each of the five change phases "atomic with rollback" and states rollback is scoped to the failing phase; TWEAK_AUTHORING.md states a failure in any phase rolls back ALL changes from the pre-step-1 snapshot. The code implements the authoring-doc semantics, and only registry_changes has any in-phase rollback at all — service, scheduler, hosts and firewall phases have zero rollback bookkeeping.

**Failing scenario.** An author reading TWEAK_SYSTEM.md:69-78 concludes that a firewall failure leaves earlier registry/service changes applied and writes a tweak that depends on that (e.g. relies on registry surviving so a post_command can detect partial application). In reality apply.rs:152-168 restores the full snapshot, undoing the registry changes too. Conversely an author reading "service_changes (atomic with rollback)" expects a mid-phase service failure to undo the services already reconfigured within that phase — apply_service_changes_atomic (helpers.rs:537-655) keeps no rollback list whatsoever and simply returns Err on the first non-skip_validation failure, relying entirely on the outer snapshot restore, which (per the other findings in this slice) can silently fail.

**Evidence.**

```
docs/TWEAK_SYSTEM.md:69-78
    → registry_changes (atomic with rollback)
      → service_changes (atomic with rollback)
        → scheduler_changes (atomic with rollback)
          → hosts_changes (atomic with rollback)
            → firewall_changes (atomic with rollback)
...
If any atomic change phase fails, all completed changes in that phase are rolled back.

docs/TWEAK_AUTHORING.md:908-911
Steps 3, 4, 5, 6, and 7 (registry, services, scheduler, hosts, firewall) are **atomic**:
- If **ANY** of these steps fails, **ALL** changes are rolled back
- Rollback uses the snapshot captured before step 1
- You get either complete success or complete rollback

src-tauri/src/commands/tweaks/helpers.rs:216-218
/// Apply ALL core changes atomically: registry, services, scheduler, hosts, firewall
/// If any step fails, caller is responsible for full rollback from snapshot
pub fn apply_all_changes_atomically(
```

**Fix.** Rewrite TWEAK_SYSTEM.md:69-78 to match the implementation: one ATOMIC SET spanning registry->service->scheduler->hosts->firewall, rolled back from the snapshot captured before pre_commands. Drop the per-phase "atomic with rollback" annotations, or restrict that annotation to registry_changes where an in-phase rollback list actually exists (helpers.rs:281, 483-505).

**Verifier correction.** Accurate as stated, but this is a documentation-only defect with no wrong runtime behavior for users; per the severity rubric ("low = cosmetic, doc-only") it is low, not medium. The author-misleading consequence is real but indirect.

<details><summary>Verifier reasoning</summary>

Both sides verified. docs/TWEAK_SYSTEM.md:69-73 annotates each phase "(atomic with rollback)" and line 78 states "If any atomic change phase fails, all completed changes in that phase are rolled back." docs/TWEAK_AUTHORING.md:908-911 states "Steps 3, 4, 5, 6, and 7 ... are **atomic** / If **ANY** of these steps fails, **ALL** changes are rolled back / Rollback uses the snapshot captured before step 1". The code follows the authoring doc: helpers.rs:216-217 comment "If any step fails, caller is responsible for full rollback from snapshot", and apply.rs:152-165 restores the whole snapshot on any failure. Per-phase rollback bookkeeping exists ONLY for registry (helpers.rs:281 `let mut rollbacks: Vec<RegistryRollback>`, replayed at helpers.rs:483-505); apply_service_changes_atomic (helpers.rs:537-651) keeps no rollback list and just returns Err, and the scheduler/hosts/firewall phases are the same shape (helpers.rs:228-251). So 4 of the 5 "atomic with rollback" labels are false. I could not find any doc text reconciling the two.

</details>

---

### [LOW · CONFIRMED] batch.rs reports the number of individual failure entries as the number of failed tweaks

`src-tauri/src/commands/tweaks/batch.rs:79` — lens: code-defect

**What is wrong.** failure_count is `failures.len()`, which accumulates one entry per failed sub-operation, but it is rendered in the message as a count of tweaks alongside operations.len()/tweak_ids.len(), producing counts that can exceed the total.

**Failing scenario.** batch_revert_tweaks(["tweak-a"]) where reverting tweak-a partially fails with 3 service restore failures. revert_tweak returns Ok(TweakResult{success:false, failures: 3 entries}). batch.rs:150-152 pushes all 3 into `failures`, so failure_count == 3 and the message is "Reverted 0/1 tweaks (3 failed, 1 partial)" — reporting 3 failed out of 1 total.

**Evidence.**

```
src-tauri/src/commands/tweaks/batch.rs:79-90
    let failure_count = failures.len();
    let message = if failure_count > 0 {
        format!(
            "Applied {}/{} tweaks ({} failed, {} partial)",
            success_count,
            operations.len(),
            failure_count,
            partial_success_count
        )

(identical shape at src-tauri/src/commands/tweaks/batch.rs:170-178 for revert)
```

**Fix.** Track a separate `failed_tweak_count` incremented once per tweak in the Err arm, and use that in the message; keep `failures.len()` only for the detail list.

<details><summary>Verifier reasoning</summary>

Verified end to end. revert_tweak returns Ok(TweakResult{success:false, failures}) with ONE entry per failed restore operation (apply.rs:322-338, built from restore_result.failures, which restore.rs:97-130 pushes per service/task/hosts/firewall item). batch.rs:150-152 flattens all of those into the batch-level `failures`, and batch.rs:170-178 formats `failures.len()` as the failed-tweak count next to `tweak_ids.len()`: "Reverted {}/{} tweaks ({} failed, {} partial)". With one tweak whose revert has 3 item failures the message reads "Reverted 0/1 tweaks (3 failed, 1 partial)" - 3 failures out of 1 total, and the same tweak counted as both failed and partial. Identical shape at batch.rs:79-90 for apply. No guard elsewhere normalizes the count. (Adjacent, unclaimed: the `if failures.is_empty()` checks at batch.rs:57 and 153 test the accumulated vector, not res.failures, so the fallback entry is skipped once any earlier tweak has failed.)

</details>

---

### [LOW · CONFIRMED] Docs say snapshots live in the app data directory; the code writes them next to the executable

`src-tauri/src/services/backup/storage.rs:19` — lens: spec-conformance

**What is wrong.** TWEAK_SYSTEM.md states snapshots are stored as JSON in the app data directory, but get_snapshots_dir resolves the directory from std::env::current_exe().parent().

**Failing scenario.** The app is installed under C:\Program Files\... and updated by an installer that replaces the install directory (or the user runs the portable exe from a temp/Downloads folder and later moves or deletes it). Every snapshot is destroyed, so every applied tweak becomes permanently unrevertable — a consequence the docs actively hide by promising app-data storage. It also means anyone reasoning about backup durability from the spec reaches the wrong conclusion.

**Evidence.**

```
docs/TWEAK_SYSTEM.md:114
Storage: JSON files in app data directory with file locking (fs4).

src-tauri/src/services/backup/storage.rs:17-25
/// Get the snapshots directory path (next to executable for portability)
pub fn get_snapshots_dir() -> Result<PathBuf, Error> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| Error::BackupFailed(format!("Failed to get executable path: {}", e)))?
        .parent()
        .ok_or_else(|| Error::BackupFailed("Failed to get executable directory".into()))?
        .to_path_buf();

    let snapshots_dir = exe_dir.join(SNAPSHOTS_DIR);
```

**Fix.** Pick one and make both sides agree. If portability is the intent, update TWEAK_SYSTEM.md:114 to say "next to the executable" and document the durability consequence; if durability is the intent, resolve via Tauri's app_data_dir with an opt-in portable override.

**Verifier correction.** The cited doc anchor is docs/TWEAK_SYSTEM.md:115, not :114. Scope should be widened: docs/APP_CONTEXT.md:40 and :52 also say app data, while docs/ARCHITECTURE.md:517 and docs/TWEAK_AUTHORING.md:1098 correctly say next to the executable - so the doc set contradicts itself, not just the code.

<details><summary>Verifier reasoning</summary>

Both sides verified, and the docs additionally contradict themselves. Code: storage.rs:17-25 `/// Get the snapshots directory path (next to executable for portability)` with `std::env::current_exe()...parent()`. Docs claiming app data: docs/TWEAK_SYSTEM.md:115 "Storage: JSON files in app data directory with file locking (fs4)." and docs/APP_CONTEXT.md:40 / :52 ("Snapshots are stored as JSON files in the app data `snapshots/` directory"). Docs claiming the opposite: docs/ARCHITECTURE.md:517 "Storage: `snapshots/` directory next to executable (portable app design)" and docs/TWEAK_AUTHORING.md:1098 "Location: `snapshots/` directory next to the executable". The durability consequence is real (an installer replacing the install dir removes every snapshot, making applied tweaks unrevertable), but that is a design consequence of an intentional portable design, not a code defect.

</details>

---

### [LOW · PLAUSIBLE] rollback_registry_operations hardcodes use_system=false, so rolling back a failed restore of a requires_system snapshot silently fails on every protected key

`src-tauri/src/services/backup/restore.rs:424` — lens: code-defect — *severity adjusted by verifier: medium → low*

**What is wrong.** restore_from_snapshot passes snapshot.requires_system into execute_registry_restore for the forward pass, but rollback_registry_operations hardcodes `false`, so the compensating writes use the unelevated path and fail for exactly the keys that needed elevation in the first place — and the failures are only logged.

**Failing scenario.** requires_system tweak with 3 HKLM registry snapshots under a key writable only via SYSTEM elevation. Revert: items 1 and 2 restore fine via restore_registry_with_system; item 3 fails (SYSTEM elevation helper unavailable / task scheduler busy). restore.rs:86 calls rollback_registry_operations, which re-applies items 1 and 2 with use_system=false -> registry_value::write_registry_json_value non-elevated -> ERROR_ACCESS_DENIED -> restore.rs:425 logs "Failed to rollback ..." and moves on. The registry is now in a third state: items 1-2 hold their ORIGINAL values (from the successful forward restore) while item 3 holds the tweaked value, and the caller is told "Failed to restore registry, rolled back 2 changes" — a claim the code did not honor.

**Evidence.**

```
src-tauri/src/services/backup/restore.rs:74
        match execute_registry_restore(&op, snapshot.requires_system) {

src-tauri/src/services/backup/restore.rs:424-432
        if let Err(e) = execute_registry_restore(&rollback_op, false) {
            log::error!(
                "Failed to rollback {}\\{}\\{}: {}",
                op.hive.as_str(), op.key, op.value_name, e
            );
        }

src-tauri/src/services/backup/restore.rs:87-91
                return Err(Error::BackupFailed(format!(
                    "Failed to restore registry, rolled back {} changes: {}",
                    completed_registry.len(), e
                )));
```

**Fix.** Thread `use_system` (or the whole snapshot) into rollback_registry_operations and pass `snapshot.requires_system`. Collect the rollback failures and include them in the returned error message instead of asserting "rolled back N changes" unconditionally.

**Verifier correction.** The elevation mismatch at restore.rs:424 and the unconditional "rolled back N changes" message are confirmed, but the described three-state registry outcome is not reachable with the shipped tweak set: in both elevated tweaks the SYSTEM-protected key is the only or the last registry entry, so the rollback list never contains a key that needs elevation. Treat as a latent robustness defect (wrong elevation + error message that asserts a rollback it did not verify), not a user-visible failure today.

<details><summary>Verifier reasoning</summary>

The code inconsistency is real and verbatim: src-tauri/src/services/backup/restore.rs:74 `match execute_registry_restore(&op, snapshot.requires_system)` vs restore.rs:424 `if let Err(e) = execute_registry_restore(&rollback_op, false)`, and restore.rs:88 unconditionally claims "rolled back {} changes" using completed_registry.len() while rollback failures are only log::error!'d (restore.rs:425-431). snapshot.requires_system is genuinely populated (capture.rs:44 passes tweak.requires_system into TweakSnapshot::new) and use_system truly switches paths (registry_value.rs:92 `if use_system { return write_registry_json_value_as_system(...) }`). My refutation attempt succeeded only on REACHABILITY: exactly two shipped tweaks are elevated - src-tauri/tweaks/ui.yaml:238 disable_widgets_win11 (requires_system, ONE registry change, so completed_registry is empty when it fails and the rollback loop is a no-op) and src-tauri/tweaks/windows_update.yaml:166 windows_update_mode (requires_ti, 4 registry changes where the only SYSTEM/TI-protected key, HKLM\SYSTEM\CurrentControlSet\Services\WaaSMedicSvc\Start, is LAST in every option). In that layout any elevation failure leaves only HKLM\Software\Policies\... entries in the rollback list, which an elevated-admin process CAN write non-elevated, so the hardcoded `false` still succeeds. The finder's 'items 1-2 protected, item 3 fails' input does not exist in the tweak set today; it becomes live the moment an author orders a protected key before another registry change. Latent bug, not a currently-reproducible harm; revert also keeps the snapshot on Err (apply.rs:266 uses `?`), so no state is lost and the user can retry.

</details>

---

## build.rs validator & type-mirror drift

### [HIGH · CONFIRMED] build.rs non-empty-option check counts skip_validation/version-filtered changes, shipping 8 tweaks whose status can never be detected

`src-tauri/build.rs:1044` — lens: code-defect

**What is wrong.** The build-time "option must have at least one change" check counts changes that are excluded from state detection (skip_validation:true, or windows_versions excluding the running OS), so YAML in which EVERY option is undetectable builds cleanly and ships a tweak whose status is permanently reported as "System Default".

**Failing scenario.** Real shipped data: `disable_geolocation_service` (src-tauri/tweaks/privacy.yaml:1138) has exactly two options, each with a single registry change carrying `skip_validation: true`. User clicks "Geolocation Disabled"; apply succeeds and HKLM\System\CurrentControlSet\Services\lfsvc\Start is set to 4. On the next status refresh, detection.rs:125-132 computes validatable_registry = [] for BOTH options and returns `not_matched()` for each, so current_option_index = null and the UI shows "System Default" even though the tweak IS applied. I enumerated the shipped YAML: 34 option-instances across `disable_xbox_services`, `ipv6_transition_mode`, `disable_geolocation_service`, `disable_dmwappushservice`, `disable_cdpusersvc`, `disable_homegroup_provider`, `disable_retail_demo`, `disable_wallet_service` have zero validatable changes for Win10 and/or Win11. In every one of these tweaks ALL options are affected, so the tweak can never report a current option.

**Evidence.**

```
src-tauri/build.rs:1044-1063:
'''
        let has_any_changes = !self.registry_changes.is_empty()
            || !self.service_changes.is_empty()
            ...
        if !has_any_changes {
            ctx.tweak_error(... "option '{}' has no changes ..."
'''
src-tauri/src/services/backup/detection.rs:101 `.filter(|c| !c.skip_validation && c.applies_to_version(windows_version))`
src-tauri/src/services/backup/detection.rs:124-132:
'''
    // If option has no validatable changes, it can't match
    if validatable_registry.is_empty()
        && validatable_services.is_empty()
        ...
    {
        return Ok(MatchResult::not_matched());
    }
'''
src-tauri/tweaks/privacy.yaml:1138-1147:
'''
      - label: "Geolocation Disabled"
        registry_missing_is_match: true
        registry_changes:
          - hive: HKLM
            key: "System\\CurrentControlSet\\Services\\lfsvc"
            value_name: "Start"
            value_type: "REG_DWORD"
            value: 4
            skip_validation: true
'''
```

**Fix.** In `TweakOption::validate`, in addition to `has_any_changes`, compute per-Windows-version validatable counts (registry changes with `!skip_validation && applies_to_version(v)`, plus non-skip_validation service/scheduler/hosts/firewall changes). Raise a build error (or at minimum a loud warning) when a tweak has zero options with any validatable change for a supported version, since such a tweak can never report a detected state. Then either drop `skip_validation` from the offending YAML entries or add a validatable companion change.

**Verifier correction.** The count is 18 option-instances across the 8 tweaks (2+4+2+2+2+2+2+2), not 34. Everything else in the claim holds. Note also that detection.rs is spec-conformant here (the distilled contract explicitly says an option with no validatable changes must be SKIPPED); the defect is the missing build-time guard plus the 8 shipped YAML entries, not the detection algorithm.

<details><summary>Verifier reasoning</summary>

Attempted refutations, all failed. (1) Build guard: src-tauri/build.rs:1044-1052 `has_any_changes` ORs `!self.registry_changes.is_empty()` etc. with no skip_validation/version filter, so an all-skip_validation option passes. (2) Detection escape hatch: src-tauri/src/services/backup/detection.rs:98-132 filters `!c.skip_validation && c.applies_to_version(...)` and returns `MatchResult::not_matched()` when all lists are empty; detect_tweak_state (detection.rs:67-87) then falls through to `current_option_index: None`. No fallback to snapshot_option_index exists. (3) UI fallback: src/lib/components/tweaks/TweakCard.svelte:111-114 maps null to segment -1 "Default" and :160-161 pushes `{value:-1,label:"System Default"}` — no snapshot-based override. The optimistic write at src/lib/stores/tweaksActions.svelte.ts:87 masks it only until the next status refresh/app restart. (4) Data: I parsed all 9 YAML files with the repo's own `yaml` package; exactly 8 tweaks have ZERO options with any non-skip_validation change: disable_xbox_services, ipv6_transition_mode, disable_geolocation_service, disable_dmwappushservice, disable_cdpusersvc, disable_homegroup_provider, disable_retail_demo, disable_wallet_service (8 of 189 tweaks). privacy.yaml:1138-1154 matches the quote verbatim. Wrong status, no user workaround, permanent for those tweaks.

</details>

---

### [MEDIUM · CONFIRMED] `aliases` receives zero build-time validation (no format, no uniqueness, no collision check) while alias resolution iterates a HashMap-derived Vec

`src-tauri/build.rs:1086` — lens: code-defect

**What is wrong.** build.rs validates tweak IDs for snake_case format and cross-file uniqueness, but never inspects the `aliases` vector at all, so duplicate aliases across two tweaks build cleanly and then resolve nondeterministically at profile-import time — importing a profile can apply the wrong tweak to the machine.

**Failing scenario.** Two tweaks in different YAML files each declare `aliases: ["disable_telemetry"]` (e.g. privacy.yaml's `disable_diagtrack` and services.yaml's `disable_dmwappushservice` after a rename). build.rs's ValidationContext only tracks `seen_tweak_ids`; nothing looks at `raw.aliases`, so the build succeeds. A user imports an old .mgx profile containing tweak_id "disable_telemetry": validation.rs:132-134 does `available_tweaks.iter().find(|t| t.aliases.contains(&selection.tweak_id))` over a Vec built at commands/profile.rs:110 via `tweaks.values().cloned().collect()` from a `std::collections::HashMap` with RandomState — iteration order differs on every process launch. The same profile therefore resolves to a different tweak on different app runs and silently applies registry/service changes the user never selected. Additionally `aliases: ["Old-Tweak Name"]` (not snake_case, colliding with a real tweak id) passes the build with no diagnostic.

**Evidence.**

```
src-tauri/build.rs:306-307:
'''
    /// Previous IDs this tweak was known by (for migration)
    #[serde(default)]
    aliases: Vec<String>,
'''
src-tauri/build.rs:1085-1094 (the only uniqueness check, IDs only):
'''
        // Check for duplicate ID
        if ctx.seen_tweak_ids.contains(&self.id) {
            ctx.tweak_error(file, &self.id, "duplicate tweak ID (already defined in another file)".to_string());
        } else {
            ctx.seen_tweak_ids.insert(self.id.clone());
        }
'''
src-tauri/src/services/profile/validation.rs:132-134:
'''
            if let Some(t) = available_tweaks
                .iter()
                .find(|t| t.aliases.contains(&selection.tweak_id))
'''
src-tauri/src/commands/profile.rs:110:
'''
    let available_tweaks: Vec<TweakDefinition> = tweaks.values().cloned().collect();
'''
```

**Fix.** Add an `seen_aliases: HashMap<String, String>` to `ValidationContext`. In `TweakDefinitionRaw::validate`, for each alias: (a) run `is_valid_tweak_id`, (b) error if the alias is already claimed by another tweak, (c) error if it equals any tweak's real id (requires a second pass or a deferred cross-check after all IDs are collected). Independently, make alias lookup deterministic by sorting `available_tweaks` by id at commands/profile.rs:110.

**Verifier correction.** Two supporting details need fixing: (a) the anchor src-tauri/src/commands/profile.rs:110 is the EXPORT path; the import/validate/apply paths are :141, :155 and :174, all building the Vec the same way from the HashMap, so the nondeterminism claim survives at the correct lines; (b) an alias colliding with a real tweak id is NOT nondeterministic — validation.rs:128 matches by `t.id` first and only falls through to the alias scan at :132, so the real id always wins. The duplicate-alias-across-two-tweaks case is the nondeterministic one. Also note zero shipped YAML currently declares `aliases`, so this is latent, not live.

<details><summary>Verifier reasoning</summary>

Refutation attempts failed. `grep -n aliases src-tauri/build.rs` returns only 307, 334, 1271 — struct field, mirror field, and passthrough; there is no validate() call touching aliases anywhere, while ids get both format (build.rs:1076) and uniqueness (build.rs:1086-1094) checks. Nondeterminism confirmed: src-tauri/src/services/tweak_loader.rs:37 returns `HashMap<String, TweakDefinition>` (std, RandomState) and commands/profile.rs:141/155/174 do `tweaks.values().cloned().collect()`; `Vec::find` over that order is arbitrary per process. I also checked whether the harm chain breaks at apply: it does not — validation.rs:222 sets `tweak_id: tweak.id.clone()` (the RESOLVED id), and commands/profile.rs:215/226 applies by `preview.tweak_id`, so the arbitrarily-chosen tweak really is written to the registry.

</details>

---

### [MEDIUM · CONFIRMED] models/tweak.rs carries a fully dead second copy of the YAML pipeline including a duplicated elevation-inference implementation

`src-tauri/src/models/tweak.rs:598` — lens: design-critique

**What is wrong.** `TweakDefinitionRaw`, `TweakFile`, `TweakDefinition::from_raw` (which reimplements ti=>system=>admin inference) and `TweakDefinition::validate` are never called outside this file's own `#[cfg(test)]` module — build.rs owns the only live copy — creating two apparent sources of truth for elevation and option-count rules with nothing enforcing agreement.

**Failing scenario.** A maintainer fixing an elevation bug greps for the inference logic, finds `from_raw` at tweak.rs:598 (a `pub fn` with a detailed doc comment describing the hierarchy), edits it, writes a unit test against it, and ships — while the shipped binary keeps using the untouched duplicate at build.rs:1262-1264. Because `from_raw` is dead, no test failure and no compiler warning signals the mistake. Same trap for `TweakDefinition::validate()`'s min-2-options rule (tweak.rs:623), which is proven by tests at tweak.rs:846-857 but never runs in production. The build.rs header comment at lines 16-19 ("These types MUST stay in sync... you MUST update it here too") is the only enforcement mechanism, and it is a comment. I diffed all 14 mirrored types field-by-field and enum-variant-by-variant: they currently agree exactly (names, serde rename/rename_all, defaults, Option-ness), so the round-trip through tweaks.json is sound today — the risk is that nothing keeps it that way.

**Evidence.**

```
src-tauri/src/models/tweak.rs:598-603 (dead):
'''
    pub fn from_raw(raw: TweakDefinitionRaw, category_id: &str) -> Self {
        let requires_ti = raw.requires_ti;
        let requires_system = raw.requires_system || requires_ti;
        let requires_admin = raw.requires_admin || requires_system;
'''
src-tauri/build.rs:1262-1264 (live):
'''
            let requires_ti = raw.requires_ti;
            let requires_system = raw.requires_system || requires_ti;
            let requires_admin = raw.requires_admin || requires_system;
'''
src-tauri/build.rs:16-19:
'''
// These types MUST stay in sync with their counterparts in src/models/tweak.rs.
// build.rs runs BEFORE the crate compiles, so we cannot import crate types.
// If you add/remove/rename a field in tweak.rs, you MUST update it here too.
'''
Only callers of `.validate()` are src-tauri/src/models/tweak.rs:846, :853, :857, :870 — all inside `mod tests`.
```

**Fix.** Delete `TweakDefinitionRaw`, `TweakFile`, `from_raw`, and `TweakDefinition::validate` from models/tweak.rs (and the tests that exercise them) so build.rs is unambiguously the single source of truth. To catch mirror drift mechanically, add a build.rs step that serializes one synthetic fully-populated `TweakDefinition` to JSON and a runtime `#[test]` in models/tweak.rs that deserializes that same fixture with `deny_unknown_fields` — any added/renamed/removed field on either side then fails a test rather than a code review.

<details><summary>Verifier reasoning</summary>

Verified every factual leg. `TweakDefinitionRaw` (src-tauri/src/models/tweak.rs:530), `TweakFile` (:698) and `from_raw` (:598) have zero references anywhere in src-tauri/src outside their own definitions — a repo-wide grep for `from_raw|TweakDefinitionRaw|TweakFile` returns only build.rs's independent copies (build.rs:299, :354, :1072, :1204, :1222) and the tweak.rs definition sites. `.validate()` has exactly four callers, all at tweak.rs:846/853/857/870 inside `mod tests` (module starts at :769). There is no runtime YAML deserialization at all: grep for `serde_yml|serde_yaml` across src-tauri/src returns nothing, so no live path can ever construct a `TweakDefinitionRaw`. I tried to refute via 'maybe it's reachable as public library API' — Cargo.toml:21 `crate-type = ["staticlib", "cdylib", "rlib"]` means these pub items are treated as reachable API, which suppresses the dead_code lint rather than exempting them; that strengthens the finding's 'no compiler warning' point instead of killing it. Both duplicated rules confirmed live in build.rs: elevation inference at build.rs:1261-1264 (`let requires_system = raw.requires_system || requires_ti;`) and the min-2-options rule at build.rs:1097 (`if self.options.len() < 2`). The only sync mechanism is the comment at build.rs:16-21. Design-critique lens: the criticized design is exactly what the code does, and the risk (a maintainer editing the dead elevation-inference copy with no test or compiler signal) is real for a tool that runs SYSTEM/TrustedInstaller-level changes. Severity medium is correct — no user harm today (I spot-checked that the two copies agree), the harm is a latent maintenance trap.

</details>

---

### [MEDIUM · CONFIRMED] TweakSnapshot has no schema version and its nested structs have no serde defaults, so any future field addition makes existing snapshots unloadable and revert impossible

`src-tauri/src/models/tweak_snapshot.rs:70` — lens: design-critique

**What is wrong.** Snapshot JSON is the only record of a user's original pre-tweak state, yet the format carries no version tag and `RegistrySnapshot`/`ServiceSnapshot`/`SchedulerSnapshot`/`HostsSnapshot`/`FirewallSnapshot` declare zero `#[serde(default)]` fields, so a strictly-additive-optional discipline is the only thing standing between a schema change and permanently unrevertable machines.

**Failing scenario.** The schema has already grown five times (`requires_system`, `original_option_index`, `scheduler_snapshots`, `hosts_snapshots`, `firewall_snapshots` all carry retrofitted `#[serde(default)]`), so growth is expected. The next author adds, say, `pub previous_permissions: String` to `RegistrySnapshot` without `#[serde(default)]`. Every snapshot on disk from the prior version now fails `serde_json::from_str` at storage.rs:149; `load_snapshot` returns `Err(BackupFailed("Failed to parse snapshot: missing field ..."))`. The user's original registry values are still sitting in the JSON file but the app cannot read them, so revert is impossible and the tweak is stuck applied — the exact loss-of-original-state failure mode the snapshot exists to prevent. There is no migration path because nothing in the file records which schema wrote it.

**Evidence.**

```
src-tauri/src/models/tweak_snapshot.rs:69-104 — no version field; contrast the retrofitted defaults on later additions:
'''
pub struct TweakSnapshot {
    pub tweak_id: String,
    ...
    /// Whether SYSTEM elevation was used for this tweak
    #[serde(default)]
    pub requires_system: bool,
    ...
    #[serde(default)]
    pub scheduler_snapshots: Vec<SchedulerSnapshot>,
'''
src-tauri/src/models/tweak_snapshot.rs:10-24 — nested struct, every field mandatory:
'''
pub struct RegistrySnapshot {
    pub hive: String,
    pub key: String,
    pub value_name: String,
    pub value_type: Option<String>,
    pub value: Option<Value>,
    pub existed: bool,
}
'''
src-tauri/src/services/backup/storage.rs:149-150:
'''
    let snapshot: TweakSnapshot = serde_json::from_str(&content)
        .map_err(|e| Error::BackupFailed(format!("Failed to parse snapshot: {}", e)))?;
'''
```

**Fix.** Add `#[serde(default = "schema_v1")] pub schema_version: u32` to `TweakSnapshot` and write it on every save, so a future loader can branch. Add `#[serde(default)]` to every non-identifying field of the five nested snapshot structs as a standing policy. Optionally, on a parse failure in `load_snapshot`, fall back to `serde_json::Value` and salvage the registry/service arrays rather than returning a hard error, so revert survives a schema mistake.

**Verifier correction.** The consequence is overstated as 'permanently unrevertable machines'. The snapshot JSON is never deleted on a parse failure — detection.rs:558-563 logs `log::warn!("Error loading snapshot for {}: {}")` and returns None (no delete), and every delete_snapshot call site (apply.rs:116/136/164/271, detection.rs:570) is gated behind a successful load or a successful restore. So the original state survives on disk and revert is blocked only until a patched build ships — bad, but recoverable, not permanent data loss. Also note the team has a 5-for-5 track record of adding `#[serde(default)]` on retrofitted fields, so the discipline is currently being observed.

<details><summary>Verifier reasoning</summary>

Verified verbatim: tweak_snapshot.rs:69-104 defines `TweakSnapshot` with no version field, while `requires_system` (:84), `original_option_index` (:89), `scheduler_snapshots` (:96), `hosts_snapshots` (:99), `firewall_snapshots` (:102) all carry retrofitted `#[serde(default)]` — confirming the schema has in fact grown five times. The five nested structs (RegistrySnapshot :10-24, ServiceSnapshot :27-35, SchedulerSnapshot :38-46, HostsSnapshot :49-57, FirewallSnapshot :60-66) declare zero defaults, so any added non-Option field there is a hard load break. The strict load path is confirmed at storage.rs:149-150: `serde_json::from_str(&content).map_err(|e| Error::BackupFailed(...))?` with no fallback, and the revert entry point commands/backup.rs:32 propagates it with `?`. My refutation attempt ('this is generic serde nitpicking, not a project-specific risk') failed on internal precedent: src-tauri/src/models/profile.rs:8-47 defines `PROFILE_SCHEMA_VERSION`, a `schema_version: u32` field and a `needs_migration()` check for the .mgx profile format — the project already established versioning for its persisted artifacts and omitted it on the one artifact that is the sole record of the user's original pre-tweak state. That asymmetry is specific and consequential, so the design-critique survives at medium; the mitigations above cap it below high.

</details>

---

### [LOW · CONFIRMED] REG_BINARY hex strings are accepted unconditionally at build time but strictly parsed at runtime, so a malformed hex string ships as a permanently broken tweak

`src-tauri/build.rs:828` — lens: code-defect — *severity adjusted by verifier: medium → low*

**What is wrong.** build.rs validates a REG_BINARY value only as "is it a JSON string"; the runtime hex parser enforces even length, <=2 hex digits per token, and valid hex digits — so several string forms pass the build and fail at apply and at status-detection time.

**Failing scenario.** Author writes `value_type: "REG_BINARY"` with `value: "00,000,FF"` (a one-character typo in the 40-byte mouse-curve blobs at src-tauri/tweaks/gaming.yaml:403). build.rs:828 sees `value.is_string()` == true and emits no error; the build succeeds and the binary ships. At apply time `parse_binary_hex_string` splits on ',' and hits token "000" with len 3 -> `Err("Invalid REG_BINARY byte [1]: '000'")`, the registry write fails, and because registry is in the ATOMIC SET the entire tweak rolls back. At status-detection time `registry_values_match` (detection.rs:243) parses the same expected value and returns the same Err, so the tweak surfaces a permanent error state. Same for the no-comma odd-length form `value: "0A0"` (compact len 3 -> "must contain an even number of digits") and for non-hex tokens like `value: "00,GG"`.

**Evidence.**

```
src-tauri/build.rs:828-838:
'''
                } else if !value.is_string() {
                    ctx.tweak_error(
                        file,
                        tweak_id,
                        format!(
                            "{}: REG_BINARY requires array of bytes or hex string, got {}",
'''
(no further inspection of the string contents)
src-tauri/src/services/registry_value.rs:211-215:
'''
        if !compact.len().is_multiple_of(2) {
            return Err(Error::ValidationError(format!(
                "REG_BINARY hex string must contain an even number of digits, got {}",
'''
src-tauri/src/services/registry_value.rs:233-242:
'''
            if token.is_empty() || token.len() > 2 {
                return Err(Error::ValidationError(format!(
                    "Invalid REG_BINARY byte [{}]: '{}'", index, token )));
            }
            u8::from_str_radix(token, 16).map_err(|_| { ... })
'''
docs/TWEAK_AUTHORING.md:1691 claims "All YAML files are validated for structural and semantic correctness before compilation."
```

**Fix.** In `RegistryChange::validate_value_type`'s `RegistryValueType::Binary` string arm, run the exact same tokenization/decoding logic as `parse_binary_hex_string` (ideally factor it into a tiny shared routine duplicated verbatim in build.rs) and emit a `ctx.tweak_error` on any decode failure. Same treatment would let build.rs report the byte index, matching the runtime message.

**Verifier correction.** Severity medium -> low. No shipped YAML triggers it (I scanned all 9 files: 8 REG_BINARY string values, 0 array values, 0 malformed tokens), the failure mode is loud and fail-safe (registry write error -> full atomic rollback), and the detection-time Err is contained per-tweak at src-tauri/src/commands/tweaks/query.rs:126 (`match backup_service::detect_tweak_state(...)`) rather than poisoning the whole tweak list. This is a build-time validation gap that would only bite on a future author typo, not a live defect.

<details><summary>Verifier reasoning</summary>

Tried to find a second build-time check: `grep -n -i 'binary|from_str_radix|hex' src-tauri/build.rs` returns only lines 90-91, 798-838 — build.rs:828 is genuinely the only string check and it is just `!value.is_string()`. Runtime strictness confirmed at src-tauri/src/services/registry_value.rs:211-215 (even-length) and :233-242 (token len<=2, from_str_radix). Detection propagation confirmed: detection.rs:242-247 calls `registry_value::registry_values_match(...)?` inside the par_iter and detection.rs:284 `let (matched, inferred) = result?;` re-raises. So the asymmetry is real; only the impact was overstated.

</details>

---

### [LOW · CONFIRMED] `aliases` is a live, profile-migration-critical YAML field but is absent from the authoring spec's tweak field table

`docs/TWEAK_AUTHORING.md:189` — lens: spec-conformance

**What is wrong.** TWEAK_AUTHORING.md's "Tweak Field Details" table enumerates every tweak-level field except `aliases`, even though both the build.rs mirror and the runtime type accept it and profile import depends on it.

**Failing scenario.** An author renaming a tweak id follows TWEAK_AUTHORING.md (the designated authoring spec), sees no `aliases` field documented, renames the id, and ships. Every previously exported .mgx profile referencing the old id now fails validation with ErrorCode::TweakNotFound (validation.rs:146-155) — exactly the breakage `aliases` exists to prevent. The mechanism is documented only in TWEAK_SYSTEM.md:122 ("`aliases` field on TweakDefinition maps old IDs to current ID for migration"), a doc aimed at system architecture, not authoring.

**Evidence.**

```
docs/TWEAK_AUTHORING.md:188-199 (spec anchor — complete table, no `aliases` row):
'''
| Field             | Type    | Required | Default | Description  |
| `id`              | string  | ✅        | -       | ...
| `info`            | string  | ❌        | -       | ...
| `risk_level`      | enum    | ✅        | -       | ...
| `force_dropdown`  | boolean | ❌        | `false` | ...
| `options`         | array   | ✅        | -       | ...
'''
code anchor src-tauri/build.rs:305-307:
'''
    /// Previous IDs this tweak was known by (for migration)
    #[serde(default)]
    aliases: Vec<String>,
'''
docs/TWEAK_SYSTEM.md:122: "`aliases` field on TweakDefinition maps old IDs to current ID for migration"
```

**Fix.** Add an `| `aliases` | array | ❌ | `[]` | Previous IDs this tweak was known by; required when renaming an id so existing profiles keep resolving. |` row to the table at docs/TWEAK_AUTHORING.md:188, plus a short "Renaming a tweak" subsection.

<details><summary>Verifier reasoning</summary>

Verified both sides. `grep -n aliases docs/TWEAK_AUTHORING.md` returns ZERO hits in the entire authoring spec, while the field table at docs/TWEAK_AUTHORING.md:188-199 otherwise enumerates every tweak-level field including all optional ones (`info`, `requires_admin`, `force_dropdown`), so it reads as exhaustive. The field is live: src-tauri/build.rs:305-307 and src-tauri/src/models/tweak.rs:538/568 both declare `aliases: Vec<String>` with `#[serde(default)]`, build.rs:1271 propagates it, and profile import depends on it (validation.rs:132-134). It is documented only in docs/TWEAK_SYSTEM.md:120,122 and docs/PROFILE_SYSTEM.md:36. Doc-only gap, correctly rated low.

</details>

---

### [LOW · CONFIRMED] Docs mark `requires_reboot` as Required while simultaneously giving it a default, and the code makes it optional

`docs/TWEAK_AUTHORING.md:197` — lens: spec-conformance

**What is wrong.** The tweak field table marks `requires_reboot` Required ✅ and also lists Default `false` — self-contradictory — and both the build.rs mirror and the runtime type declare it `#[serde(default)]`, i.e. optional.

**Failing scenario.** An author omits `requires_reboot` from a new tweak. Per the spec table it is a required field, so they expect a build error; the build silently succeeds with `requires_reboot: false`. Conversely a reviewer enforcing the table rejects valid YAML. Every other boolean in the same table (`requires_admin`, `requires_system`, `requires_ti`, `force_dropdown`) is correctly marked ❌ with default `false`, so the ✅ on this one row is the outlier.

**Evidence.**

```
spec anchor docs/TWEAK_AUTHORING.md:197:
'''
| `requires_reboot` | boolean | ✅        | `false` | Changes require restart to fully apply.                             |
'''
code anchor src-tauri/build.rs:316-317:
'''
    #[serde(default)]
    requires_reboot: bool,
'''
code anchor src-tauri/src/models/tweak.rs:577-578:
'''
    #[serde(default)]
    pub requires_reboot: bool,
'''
```

**Fix.** Change the Required column for `requires_reboot` at docs/TWEAK_AUTHORING.md:197 from ✅ to ❌ to match `#[serde(default)]`.

**Verifier correction.** The self-contradiction is broader than the table: the YAML skeleton at docs/TWEAK_AUTHORING.md:180 also says `requires_reboot: boolean # Required: Needs restart to take effect`. Mitigating fact worth noting: all 189 shipped tweaks do specify requires_reboot explicitly, so nothing is currently mis-built.

<details><summary>Verifier reasoning</summary>

Verified both sides verbatim. docs/TWEAK_AUTHORING.md:197 reads `| requires_reboot | boolean | ✅ | false | ...` — Required and Default populated simultaneously, which is internally inconsistent, and it is the only ✅ boolean in a table where requires_admin/requires_system/requires_ti/force_dropdown are all ❌/`false`. Code side: src-tauri/build.rs:316-317 and src-tauri/src/models/tweak.rs (TweakDefinitionRaw and TweakDefinition) both carry `#[serde(default)] pub requires_reboot: bool`, and no ctx.tweak_error enforces presence, so omission builds silently as false. Doc-only, no runtime harm.

</details>

---

## State detection & value comparison

### [HIGH · CONFIRMED] Any registry read failure is reported as "value missing", and *_missing_is_match then turns it into an inferred MATCH

`src-tauri/src/services/backup/detection.rs:234` — lens: code-defect

**What is wrong.** read_registry_value collapses every non-NotFound error (ACCESS_DENIED, wrong stored value type, corrupt data) into (None, false) = "does not exist", and detection's Set branch then applies registry_missing_is_match to it, reporting an inferred match for a value that is present and wrong.

**Failing scenario.** Tweak disable_telemetry (src-tauri/tweaks/privacy.yaml:108-114) declares option 0 with `registry_missing_is_match: true` and HKLM\System\CurrentControlSet\Services\DiagTrack\Start as REG_DWORD. On a machine where `Start` exists but is stored with a different vtype (e.g. REG_QWORD/REG_SZ written by another tool), winreg's `FromRegValue for u32` returns ERROR_BAD_FILE_TYPE (winreg-0.55.0/src/types.rs:126 `_ => werr!(ERROR_BAD_FILE_TYPE)`), read_dword maps it to Error::RegistryOperation, read_registry_value swallows it to (None,false) -> existed=false -> missing_is_match -> Ok((true,true)). Result: UI reports "Telemetry Disabled (Inferred)" while DiagTrack Start is actually 2 and telemetry is running. The identical swallow in capture.rs also records `existed: false` in the snapshot, so a later revert DELETES the value instead of restoring it.

**Evidence.**

```
src-tauri/src/services/backup/capture.rs:587-598:
        Err(Error::RegistryKeyNotFound(_)) => Ok((None, false)),
        Err(e) => {
            log::warn!(
                "Failed to read {}\\{}\\{}: {}",
                ...
            );
            Ok((None, false))
        }
src-tauri/src/services/backup/detection.rs:234-240:
                    if !existed {
                        // Item doesn't exist - check missing_is_match flag
                        if missing_is_match {
                            return Ok((true, true)); // Inferred match
                        }
                        return Ok((false, false));
                    }
```

**Fix.** Make read_registry_value return a tri-state (Present(value) | Absent | Unreadable(err)) or propagate the error. Only Absent may feed *_missing_is_match; Unreadable must make the option not-match (and must abort snapshot capture rather than record existed:false).

**Verifier correction.** The mechanism is confirmed, but the cited DiagTrack scenario is contrived (service Start values are REG_DWORD in practice, and no shipped tweak declares a type that conflicts with a Windows default type — src-tauri/tweaks/performance.yaml:609/674 and gaming.yaml:379 all use the correct REG_SZ). The reachable triggers are: a value written with a foreign type by a third-party tool, ERROR_INVALID_DATA on malformed data, or Error::RegistryAccessDenied from open_subkey (registry_service.rs:36). The more severe half of the finding is the capture path, not detection: capture.rs:124-141 stores existed:false and restore.rs:170 then DELETES a value that was actually present, destroying the user's original state.

<details><summary>Verifier reasoning</summary>

Verified src-tauri/src/services/backup/capture.rs:584-598 — only Error::RegistryKeyNotFound is distinguished; every other Err is logged and returned as (None, false). registry_service.rs:49-52 maps a non-NotFound get_value error to Error::RegistryOperation, and winreg-0.55.0/src/types.rs:126 (`_ => werr!(ERROR_BAD_FILE_TYPE)` for u32) confirms a wrong stored vtype produces exactly such an error, whose io::ErrorKind is not NotFound. detection.rs:227-240 then feeds existed=false straight into missing_is_match and returns Ok((true,true)) (inferred match); privacy.yaml:109 does set registry_missing_is_match: true, so the inferred-match path is live. I searched for a guard on the read path and found none — no caller inspects the error, and the same swallow is used by capture_registry_snapshots (capture.rs:124) and capture_current_state (capture.rs:390), so restore.rs:170 (`if !op.existed { // delete }`) turns an unreadable-but-present value into a deletion. Refutation failed.

</details>

---

### [MEDIUM · CONFIRMED] inspection.rs compares registry values with raw JSON equality, so REG_BINARY authored as a hex string always reports a mismatch that detection calls a match

`src-tauri/src/services/backup/inspection.rs:156` — lens: code-defect — *severity adjusted by verifier: high → medium*

**What is wrong.** inspect_registry_changes uses helpers::values_match (plain serde_json equality with an int fallback) while detection uses registry_value::registry_values_match (type-normalizing), so the two disagree for every REG_BINARY value authored in the supported "00,A0,FF" hex-string form.

**Failing scenario.** gaming.yaml:402-403 declares `value_type: "REG_BINARY"` with `value: "00,00,...,00"` (hex string; build.rs:828 explicitly allows this form). read_registry_value returns the stored bytes as a JSON array [0,0,...]. Detection: registry_values_match parses both sides to RegistryValue::Binary and returns true (see the registry_value.rs:282-291 unit test). Inspection: values_match sees Array vs String -> as_i64/as_u64 both None -> `va == vb` false -> is_match=false, all_match=false, matched_option_index=None. So for a correctly applied mouse-curve tweak the status badge says the option is current while the inspection modal lists every binary value as a mismatch.

**Evidence.**

```
src-tauri/src/services/backup/inspection.rs:156:
                let is_match = exists && values_match(&current_val, &Some(expected_val.clone()));
src-tauri/src/services/backup/helpers.rs:39-46:
            if let (Some(na), Some(nb)) = (va.as_i64(), vb.as_i64()) {
                return na == nb;
            }
            ...
            // Standard comparison
            va == vb
src-tauri/src/services/registry_value.rs:74-76:
            let normalized_current = parse_registry_value(value_type, current)?.to_json();
            let normalized_expected = parse_registry_value(value_type, expected)?.to_json();
            Ok(normalized_current == normalized_expected)
```

**Fix.** Delete helpers::values_match and have inspection call registry_value::registry_values_match(value_type, &current_val, &Some(expected)) — one comparison implementation shared by detection, inspection and snapshot validation.

**Verifier correction.** Downgraded from high: the divergence is confined to the details modal (wrong red rows, all_match=false, matched_option_index=None). The tweak card's status badge comes from detect_tweak_state and stays correct, and no registry write or snapshot depends on values_match, so there is no state-loss or half-apply risk.

<details><summary>Verifier reasoning</summary>

Verified inspection.rs:2 imports helpers::values_match and inspection.rs:156 uses it, while detection.rs:242-247 uses registry_value::registry_values_match. helpers.rs:39-46 does plain serde_json equality with only an int fallback. capture.rs:574-575 turns read_binary's Vec<u8> into a JSON array, and gaming.yaml:402-403/436-437/470-471 author REG_BINARY as the comma-hex string form that parse_binary_hex_string (registry_value.rs:203) supports. For Array vs String, as_i64/as_u64 are both None and `va == vb` is false, so is_match=false; registry_value.rs:282-291's own unit test asserts the opposite result for the same pair. Divergence is real; I found no normalization step between them.

</details>

---

### [MEDIUM · CONFIRMED] inspection.rs never reads registry_/service_/scheduler_missing_is_match, so it contradicts detection on every inferred match

`src-tauri/src/services/backup/inspection.rs:120` — lens: spec-conformance — *severity adjusted by verifier: high → medium*

**What is wrong.** inspect_registry_changes / inspect_service_changes / inspect_scheduler_changes take only the change lists and never consult the option's *_missing_is_match flags that detection.rs honours, so a missing item that detection reports as an (inferred) match is reported by inspection as a hard mismatch.

**Failing scenario.** Spec anchor: docs/TWEAK_AUTHORING.md:346 — "missing items are treated as matching that option. The UI will show an 'Inferred' badge". Code anchor: inspection.rs:120-168 has no access to `option.registry_missing_is_match` (the field is only read at detection.rs:145/151/159). Concrete: privacy.yaml:109 disable_telemetry option 0 sets `registry_missing_is_match: true`; on an edition/image where HKLM\...\Services\DiagTrack\Start is absent, detect_tweak_state returns current_option_index=Some(0), status_inferred=true (badge shown), while get_tweak_inspection returns that same option with is_match=false on every row, all_match=false and matched_option_index=None — the details modal tells the user the option they are shown as running does not match.

**Evidence.**

```
src-tauri/src/services/backup/inspection.rs:120-123:
fn inspect_registry_changes(
    option: &TweakOption,
    windows_version: u32,
) -> Result<Vec<RegistryMismatch>, Error> {
src-tauri/src/services/backup/inspection.rs:156:
                let is_match = exists && values_match(&current_val, &Some(expected_val.clone()));
src-tauri/src/services/backup/detection.rs:143-146:
                                check_registry_matches(
                                    &validatable_registry,
                                    option.registry_missing_is_match,
                                )
```

**Fix.** Pass the option's missing_is_match flags into the inspect_* helpers and mark absent items as is_match=true with an `inferred: true` field on the Mismatch structs, mirroring MatchResult; better still, have inspection and detection share one per-change evaluator.

**Verifier correction.** Downgraded from high: the doc line quoted (TWEAK_AUTHORING.md, "The `*_missing_is_match` Flags" section) describes status detection, and detection.rs honours it correctly. The defect is the UI self-contradiction between the badge and the details modal, not a wrong applied/reverted state.

<details><summary>Verifier reasoning</summary>

Grepped inspection.rs end to end: `missing_is_match` appears nowhere; inspect_registry_changes(inspection.rs:120-123) and inspect_service_changes/inspect_scheduler_changes take only the option/change lists, whereas detection.rs:143-160 passes option.registry_missing_is_match / service_ / scheduler_. inspection.rs:156 `let is_match = exists && values_match(...)` makes an absent value a hard mismatch. Doc side verified: "By setting the appropriate flag on an option, missing items are treated as matching that option. The UI will show an 'Inferred' badge". privacy.yaml:109 is a live instance. Frontend consumes the contradiction at TweakDetailsModal.svelte:122 (`inspection.options.find((o) => o.all_match)`) and :144 `hasCustomState: !matchedOption`. No compensating code path found.

</details>

---

### [MEDIUM · CONFIRMED] calculate_overall_match returns true for an option with zero validatable items, so matched_option_index points at an option detection can never select

`src-tauri/src/services/backup/inspection.rs:98` — lens: spec-conformance

**What is wrong.** inspection's `iter().filter(!skip_validation).all(...)` over an empty/entirely-skipped collection is vacuously true, contradicting the spec rule that an option with no validatable changes cannot be current — a rule detection.rs does implement.

**Failing scenario.** Spec anchor: docs/TWEAK_AUTHORING.md:1053 "3. If no validatable changes remain → skip option" and :1071 "**Empty Options**: If an option has no validatable changes (all filtered out), it cannot be detected as current". Code anchor: inspection.rs:98-118. Concrete shipping case: gaming.yaml:621-648 `disable_xbox_services` — every registry change in BOTH options carries `skip_validation: true` and there are no other change types. detect_tweak_state correctly returns current_option_index=None (detection.rs:125-132), but inspect_tweak returns all_match=true for BOTH options and matched_option_index=Some(0). get_tweak_inspection therefore tells the UI option 0 matches while get_tweak_status says "System Default". The same vacuity occurs when a task_name_pattern matches zero tasks with ignore_not_found=false (inspection.rs:276 iterates an empty list and pushes no rows). The unit test at inspection.rs:503-510 asserts this vacuous behaviour as intended.

**Evidence.**

```
src-tauri/src/services/backup/inspection.rs:98-101:
    registry_results
        .iter()
        .filter(|r| !r.skip_validation)
        .all(|r| r.is_match)
src-tauri/src/services/backup/inspection.rs:503-510:
        // All skipped -> OK (technically vacuous truth)
        assert!(calculate_overall_match(
            std::slice::from_ref(&registry_fail_skip),
            &[],
...
src-tauri/src/services/backup/detection.rs:125-132 (correct behaviour):
    if validatable_registry.is_empty()
        && validatable_services.is_empty()
        ...
        return Ok(MatchResult::not_matched());
```

**Fix.** Add the same emptiness guard to calculate_overall_match: count validatable rows across all five categories and return false when the count is zero, so inspection's matched_option_index can never disagree with detection.

<details><summary>Verifier reasoning</summary>

Verified calculate_overall_match(inspection.rs:98-118) filters out skip_validation rows and calls .all(), which is vacuously true on an empty remainder; inspect_tweak:37 then sets matched_option_index = position(|opt| opt.all_match). Detection implements the opposite rule at detection.rs:125-132 (returns MatchResult::not_matched() when all five validatable vecs are empty). The shipped instance holds: gaming.yaml disable_xbox_services has exactly two registry changes per option, every one carrying skip_validation: true, and no service/scheduler/hosts/firewall changes, so inspection yields all_match=true for option 0 while detection yields current_option_index=None. Doc side verified: TWEAK_AUTHORING.md "3. If no validatable changes remain → skip option" and "**Empty Options**: If an option has no validatable changes (all filtered out), it cannot be detected as current". The unit test at inspection.rs:503-510 does assert the vacuous behaviour. Impact is UI-only (green check + suppressed "Custom Configuration"), which matches medium.

</details>

---

### [MEDIUM · CONFIRMED] A shipping tweak whose every change is skip_validation is permanently undetectable and no validation prevents it

`src-tauri/src/services/backup/detection.rs:125` — lens: code-defect

**What is wrong.** disable_xbox_services has skip_validation:true on 100% of its changes in 100% of its options, so option_matches_current_state always returns not_matched and the tweak's status is stuck at "System Default" even immediately after a successful apply; build.rs's "each option must have at least one change" rule counts raw changes, not validatable ones, so nothing catches this.

**Failing scenario.** Apply gaming.yaml `disable_xbox_services` option 0 "Xbox Services Disabled". The apply writes XboxGipSvc\Start=4 and xbgm\Start=4 successfully and a snapshot is created. Immediately afterwards get_tweak_status -> detect_tweak_state -> for both options validatable_registry is empty (both changes are skip_validation) and services/scheduler/hosts/firewall are empty -> detection.rs:131 returns not_matched for both -> current_option_index=None. The UI segmented switch shows "System Default" / not-applied for a tweak that was just applied, and is_applied is false while has_backup is true.

**Evidence.**

```
src-tauri/tweaks/gaming.yaml:621-634:
      - label: "Xbox Services Disabled"
        registry_missing_is_match: true
        registry_changes:
          - hive: HKLM
            key: "System\\CurrentControlSet\\Services\\XboxGipSvc"
            ...
            skip_validation: true
          - hive: HKLM
            key: "System\\CurrentControlSet\\Services\\xbgm"
            ...
            skip_validation: true
src-tauri/src/services/backup/detection.rs:124-132:
    // If option has no validatable changes, it can't match
    if validatable_registry.is_empty()
        && validatable_services.is_empty()
        ...
        return Ok(MatchResult::not_matched());
docs/TWEAK_AUTHORING.md:1706: | **Empty Options** | Error | Each option must have at least one change (registry, service, etc.) |
```

**Fix.** Add a build.rs validation error when an option has zero validatable (non-skip_validation, version-applicable) changes, and fix disable_xbox_services so at least one registry change per option is validated.

<details><summary>Verifier reasoning</summary>

Tried to refute three ways, all failed. (1) Data check: src-tauri/tweaks/gaming.yaml:620-649 — disable_xbox_services has exactly 2 options, 2 registry_changes each, and all four carry `skip_validation: true`; there are no service/scheduler/hosts/firewall changes and no pre/post commands to look at. (2) Code check: detection.rs:98-122 filters every category on `!c.skip_validation`, so all five validatable vectors are empty for both options, and detection.rs:125-132 `return Ok(MatchResult::not_matched())` fires unconditionally — no earlier guard, no missing_is_match escape hatch (registry_missing_is_match on option 0 is never consulted because check_registry_matches is never reached). detect_tweak_state (detection.rs:67-87) then falls through to `current_option_index: None`. (3) Build-time guard check: build.rs:1043-1063 computes `has_any_changes` from raw `!self.registry_changes.is_empty() || ...` — it counts raw changes, exactly as the finding claims, so nothing rejects an all-skip_validation option. Post-apply the frontend masks it briefly (tweaksActions.svelte.ts:84-88 optimistically sets current_option_index=optionIndex) but any get_all_tweak_statuses refresh/app restart resets it to None. Note the runtime code itself is spec-correct here (the distilled contract mandates skipping options with no validatable changes); the defect is the shipped YAML plus the missing build.rs check. Severity medium stands: permanent wrong status (is_applied=false, has_backup=true, UI 'System Default') on one low-risk tweak, with revert still functional.

</details>

---

### [MEDIUM · CONFIRMED] TweakStatus.is_applied is documented as "has a snapshot" but is computed as current_option_index == Some(0)

`src-tauri/src/commands/tweaks/query.rs:102` — lens: spec-conformance

**What is wrong.** The model documents is_applied as "Whether the tweak has been applied by this app (has snapshot)", but query.rs derives it purely from matching option index 0, so tweaks whose option 0 is the stock Windows state report as applied on a machine that has never run the app.

**Failing scenario.** Spec/doc anchor: src-tauri/src/models/tweak.rs:739-740 `/// Whether the tweak has been applied by this app (has snapshot)`. Code anchor: query.rs:102 and query.rs:134. Concrete: power.yaml:37 tweak `sleep_mode` option 0 is "Modern Standby (Default)" (PlatformAoAcOverride absent, HibernateEnabled=1, HiberbootEnabled=1) — exactly stock Windows 11 laptop state. On a fresh install, detection matches option 0 -> is_applied=true with has_backup=false. Consequences: SettingsModal.svelte:14 and CategoryView.svelte:44 applied counters are inflated, TweakDetailsModal.svelte:179 shows "Applied", and ProfileExportModal.svelte:35 (`tweaksStore.list.filter((t) => t.status.is_applied)`) exports never-touched default-state tweaks into the .mgx profile. gaming.yaml:217 "Fully Enabled (Default)" is a second instance.

**Evidence.**

```
src-tauri/src/models/tweak.rs:739-740:
    /// Whether the tweak has been applied by this app (has snapshot)
    pub is_applied: bool,
src-tauri/src/commands/tweaks/query.rs:100-104:
    Ok(TweakStatus {
        tweak_id,
        is_applied: state.current_option_index == Some(0),
        last_applied,
        has_backup: state.has_snapshot,
```

**Fix.** Either compute is_applied from state.has_snapshot (matching its doc comment and the profile-export use case) or rename it to matches_option_zero and update the doc comment plus the frontend counters/export filter.

**Verifier correction.** The code-vs-doc contradiction is confirmed exactly as stated, but two mechanism details are off: the frontend mirrors the backend rule deliberately (tweaksActions.svelte.ts:85 `is_applied: optionIndex === 0`), so the stale item is most likely the doc comment rather than the computation; and ProfileExportModal only *pre-selects* the false-positive tweaks (resetSelection, line 51-56) rather than silently exporting them — the user can uncheck them before export.

<details><summary>Verifier reasoning</summary>

Both sides verified verbatim. Doc side: src-tauri/src/models/tweak.rs:739-740 `/// Whether the tweak has been applied by this app (has snapshot)` immediately above `pub is_applied: bool`. Code side: query.rs:102 and query.rs:134 both `is_applied: state.current_option_index == Some(0)` — has_snapshot is available in the same struct (used at query.rs:104 `has_backup: state.has_snapshot`) and deliberately not used. Attempted refutations: (a) maybe option 0 is always the 'tweaked' state — refuted by power.yaml:37 `- label: "Modern Standby (Default)"` (PlatformAoAcOverride delete_value, HibernateEnabled=1, HiberbootEnabled=1) and gaming.yaml:217 `- label: "Fully Enabled (Default)"`, both stock-Windows states; detection.rs:251-260 treats DeleteValue as matching when the value is absent, so a fresh machine genuinely matches option 0. (b) maybe the frontend compensates — refuted: TweakCard.svelte:245/254, TweakDetailsModal.svelte:178-179 ('Applied'), CategoryView.svelte:44, SnapshotsView.svelte:56, FavoritesView.svelte:61, tweaksData.svelte.ts:59-60/71 and ProfileExportModal.svelte:35 (`filter((t) => t.status.is_applied)`, feeding resetSelection() at line 51-56 which pre-selects all of them for export) all consume it raw. Medium is the right weight: misleading counters/badges and inflated default profile-export selection, but no state loss and the user can deselect in the export wizard.

</details>

---

### [MEDIUM · PLAUSIBLE] schtasks query failures are turned into "task not found", which ignore_not_found/missing_is_match/Delete then convert into a MATCH

`src-tauri/src/services/backup/detection.rs:384` — lens: code-defect — *severity adjusted by verifier: high → medium*

**What is wrong.** check_scheduler_matches uses `.unwrap_or_default()` / `.unwrap_or(TaskState::NotFound)` on scheduler queries, so a failed query is indistinguishable from a genuinely absent task, and the not-found branches immediately return a match.

**Failing scenario.** windows_update.yaml:271-275 option "Disabled (Complete)" uses `task_name_pattern: "Schedule Scan|USO|MusNotification|Reboot|Refresh"` with `ignore_not_found: true`. scheduler_service::list_tasks_in_folder only treats the ENGLISH strings "does not exist"/"cannot find" as absence (scheduler_service.rs:212) and returns Err for anything else — a localized (German/French) Windows error, an "Access is denied" on the SYSTEM-owned UpdateOrchestrator folder, or an RPC failure when the Task Scheduler service is unavailable. detection.rs:384 swallows that Err to an empty Vec, line 392 sees `ignore_not_found` and returns Ok((true,false)). Net effect: with all UpdateOrchestrator tasks Ready/enabled, the option reports as fully applied and the UI shows Windows Update as "Disabled (Complete)".

**Evidence.**

```
src-tauri/src/services/backup/detection.rs:383-399:
                let tasks = scheduler_service::find_tasks_by_pattern(&change.task_path, pattern)
                    .unwrap_or_default();

                if tasks.is_empty() {
                    ...
                    if change.ignore_not_found {
                        // Tasks not found but ignore_not_found is set
                        return Ok((true, false));
                    }
src-tauri/src/services/backup/detection.rs:411-412:
                let current_state = scheduler_service::get_task_state(&change.task_path, task_name)
                    .unwrap_or(scheduler_service::TaskState::NotFound);
```

**Fix.** Propagate the Err (or map it to a distinct TaskState::Unreadable) instead of unwrap_or_default/unwrap_or(NotFound); only a genuine "task absent" result may satisfy ignore_not_found / missing_is_match / Delete. Also make scheduler_service detect absence via exit code rather than English stderr text.

**Verifier correction.** The code claim is exactly right, but the named triggers mostly do NOT produce a wrong answer. A localized "folder does not exist" error means the tasks genuinely are absent, so unwrap_or_default -> ignore_not_found -> match is coincidentally correct. Access-denied on UpdateOrchestrator is not reproducible: `schtasks /Query /TN '\Microsoft\Windows\UpdateOrchestrator\' /FO LIST /V` exits 0 with 18 tasks on this machine, and the English absence message ("The system cannot find the file specified.") is already matched by scheduler_service.rs:212 via "cannot find". The residual real failure mode is narrower: Task Scheduler/RPC unavailable or schtasks failing to spawn while the tasks exist -> option falsely reported as applied.

<details><summary>Verifier reasoning</summary>

Confirmed detection.rs:383-384 `.unwrap_or_default()` and detection.rs:411-412 `.unwrap_or(TaskState::NotFound)`, and that scheduler_service.rs:210-219 / 74-85 return Err for any stderr not containing "does not exist"/"cannot find". windows_update.yaml:271-275 does use task_name_pattern + ignore_not_found: true as claimed. But I attacked the failing scenario and it largely dies: I executed the exact query the code runs and it succeeded (exit=0, 18 TaskName lines), and probes of other folders showed the missing-folder message is already handled. Note the asymmetry that keeps it alive: apply propagates the same Err (scheduler_service.rs:297 uses `?`), so apply fails loudly while detection silently reports the option as current.

</details>

---

### [LOW · PLAUSIBLE] Firewall detection treats any netsh failure or non-English output as "rule exists", so Create-op firewall changes always report as matching

`src-tauri/src/services/backup/detection.rs:487` — lens: code-defect — *severity adjusted by verifier: medium → low*

**What is wrong.** check_firewall_matches trusts firewall_service::rule_exists, which never checks netsh's exit status and decides existence by the absence of one hard-coded English sentence in stdout, so a failed or localized netsh invocation yields exists=true.

**Failing scenario.** Given any tweak option with `firewall_changes: [{ operation: create, name: "X" }]`: if the Windows Firewall service (MpsSvc) is stopped/disabled — which this app's own service tweaks can cause — `netsh advfirewall firewall show rule name=X` exits non-zero and prints its error on stderr, leaving stdout empty. rule_exists returns `!"".contains("No rules match the specified criteria")` = true, so check_firewall_matches sees exists==expected_exists==true and reports the option as applied even though no firewall rule exists. Identical false positive on any non-English Windows, where the "no rules match" message is localized.

**Evidence.**

```
src-tauri/src/services/backup/detection.rs:486-494:
    for change in validatable_firewall {
        let exists = firewall_service::rule_exists(&change.name)?;
        let expected_exists = matches!(
            change.operation,
            crate::models::tweak::FirewallOperation::Create
        );
src-tauri/src/services/firewall_service.rs:25-28:
    let stdout = String::from_utf8_lossy(&output.stdout);

    // If the rule doesn't exist, netsh returns "No rules match the specified criteria"
    Ok(!stdout.contains("No rules match the specified criteria"))
```

**Fix.** In rule_exists, check output.status: treat exit code 1 with the "no rules" condition as absent, any other non-zero exit as Err, and prefer a locale-independent probe (e.g. parse for a "Rule Name:"/"name=" record or use the PowerShell Get-NetFirewallRule cmdlet with a structured result).

**Verifier correction.** Confirmed: rule_exists ignores exit status and relies on a hard-coded English string, so any non-'no rules match' output yields exists=true. Corrections: (a) executed netsh shows errors go to STDOUT, not stderr, and exit code 1 is also the legitimate not-found code, so the stderr premise and the exit-status-based fix sketch are both wrong — a locale-independent probe (Get-NetFirewallRule) is the only real fix; (b) no tweak in src-tauri/tweaks declares firewall_changes, so this is latent, not currently user-reachable; (c) the impact is broader than detection when it does fire: firewall_service.rs:42 silently skips rule creation and capture.rs:295 snapshots a wrong 'existed' flag, leaving a created rule behind after revert (restore.rs:387-398).

<details><summary>Verifier reasoning</summary>

The code claim is verbatim-accurate: firewall_service.rs:13-29 never touches `output.status` and returns `Ok(!stdout.contains("No rules match the specified criteria"))`; detection.rs:486-494 consumes it as ground truth. But I attacked the failing scenario and two parts broke. (1) Reachability: `Grep firewall_changes` over src-tauri/tweaks returns *no matches* — zero shipped tweaks use firewall changes, so no user can hit this today; it is latent in an author-facing feature (documented in docs/TWEAK_AUTHORING.md, 7 hits). (2) Mechanism: I executed netsh on this Windows 11 box. Missing rule -> EXIT=1, stdout `\nNo rules match the specified criteria.\n`, stderr EMPTY. Error cases (`bogusparam=1`, missing args) -> EXIT=1, error text on STDOUT, stderr EMPTY. So netsh does not write errors to stderr as the finding asserts, and exit code 1 is *also* the not-found code, so the proposed fix ('treat exit 1 + no-rules as absent, any other non-zero as Err') would not catch the MpsSvc case either. The localization half survives intact and is the real defect: on a non-English Windows the message is translated, `contains` fails, rule_exists returns true, and the false positive propagates beyond detection into firewall_service.rs:42 (create silently no-ops and returns Ok) and capture.rs:295 (snapshot records existed=true), so a created rule would survive revert at restore.rs:387-398. Downgraded to low because no shipping tweak data can reach any of it.

</details>

---

## Windows service layer & elevation

### [HIGH · CONFIRMED] delete_key with a trailing/lone backslash calls RegDeleteTreeW(parent, NULL) and wipes the parent key's entire contents

`src-tauri/src/services/registry_service.rs:458` — lens: code-defect — *severity adjusted by verifier: critical → high*

**What is wrong.** delete_key splits the key path on the last backslash without rejecting an empty child component, and winreg maps an empty subkey name to a NULL path, which RegDeleteTreeW documents as "delete the subkeys and values of THIS key" - so a trailing backslash deletes the parent key's whole subtree instead of one child, and a lone backslash targets the hive root.

**Failing scenario.** YAML: `action: delete_key`, `hive: HKLM`, `key: "SOFTWARE\\"` (trailing backslash). rsplit_once('\\') -> parent="SOFTWARE", child="". open_subkey_with_flags("SOFTWARE", KEY_WRITE) succeeds; delete_subkey_all("") passes path_ptr=NULL -> RegDeleteTreeW(HKLM\SOFTWARE, NULL) -> every subkey and value under HKLM\SOFTWARE is deleted. Worse: `key: "\\"` (a single backslash) -> rsplit_once returns Some(("","")) -> parent="" opens a second handle to HKLM itself (winreg: "Will open another handle to itself if path is an empty string") -> RegDeleteTreeW(HKLM, NULL) -> the entire HKLM hive contents are deleted, unbootable machine. build.rs only rejects a whitespace-empty key, so both pass compile-time validation.

**Evidence.**

```
src-tauri/src/services/registry_service.rs:458-466:
'''
    let (parent_path, child_name) = match key_path.rsplit_once('\\') {
        Some((parent, child)) => (parent, child),
        None => {
            // No parent - trying to delete a top-level key (not allowed)
            return Err(Error::RegistryOperation(
                "Cannot delete top-level registry key".into(),
            ));
'''
src-tauri/src/services/registry_service.rs:468-479:
'''
    let parent_key = RegKey::predef(hive_key)
        .open_subkey_with_flags(parent_path, KEY_WRITE)
...
    parent_key.delete_subkey_all(child_name).map_err(|e| {
'''
winreg-0.55.0/src/reg_key.rs:598-610 (confirmed in vendored source):
'''
    pub fn delete_subkey_all<P: AsRef<OsStr>>(&self, path: P) -> io::Result<()> {
        let c_path;
        let path_ptr = if path.as_ref().is_empty() {
            ptr::null()
...
                path_ptr, //If this parameter is NULL, the subkeys and values of this key are deleted.
'''
winreg-0.55.0/src/reg_key.rs:173: `/// Will open another handle to itself if `path` is an empty string.`
src-tauri/build.rs:518 (only guard): `if self.key.trim().is_empty() {`
```

**Fix.** In delete_key, after rsplit_once, reject empty parent_path AND empty child_name explicitly. Also normalize/reject key paths with leading or trailing backslashes, and add the same rejection to build.rs RegistryChange::validate so the typo fails at compile time rather than at runtime on a user's machine.

**Verifier correction.** The mechanism is exactly as described, but the trigger requires an authoring typo in a compile-time-embedded YAML: NO shipped tweak currently has a trailing-backslash delete_key key (grepped all of src-tauri/tweaks/ - the only delete_key entry is ui.yaml:82 with a well-formed CLSID path). Registry paths are never user- or profile-supplied at runtime (profiles resolve tweaks by id/alias/content-hash, not raw keys), so this is an unguarded latent footgun rather than a live catastrophe. Downgraded critical -> high.

<details><summary>Verifier reasoning</summary>

Attempted refutation on three fronts, all failed. (1) Guard upstream: src-tauri/src/commands/tweaks/helpers.rs:419 calls registry_service::delete_key(&change.hive, &change.key) with zero normalization or trailing-backslash rejection. (2) Guard in delete_key: registry_service.rs:458-466 only rejects the no-backslash case; empty parent_path and empty child_name both fall through. require_write_access (registry_service.rs:222-229) only checks admin for HKLM, no path shape check. (3) Guard in winreg: read the vendored crate at C:/Users/Ehsan/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/winreg-0.55.0/src/reg_key.rs:598-610 - `let path_ptr = if path.as_ref().is_empty() { ptr::null() }` passed to RegDeleteTreeW with the crate's own comment `//If this parameter is NULL, the subkeys and values of this key are deleted.`, and reg_key.rs:173 `/// Will open another handle to itself if path is an empty string.` Both quoted lines exist verbatim. build.rs:518 `if self.key.trim().is_empty() {` is the only key validation, confirmed - it does not catch "SOFTWARE\\" or "\\". Aggravating factor the finder understated: the DeleteKey rollback is RecreateKey (helpers.rs:428-431), which recreates an EMPTY key - helpers.rs:267 admits the contents cannot be restored, so a mis-triggered delete is irreversible.

</details>

---

### [HIGH · CONFIRMED] run_schtasks_as_ti/as_system return a raw exit code that the caller discards, so elevated scheduler failures are reported as success

`src-tauri/src/services/elevation/ti_elevation.rs:468` — lens: code-defect

**What is wrong.** Unlike every sibling elevated helper (set_service_startup_elevated, set_registry_value_as_system, run_command_as_*), the schtasks helpers do not check the exit code themselves - they return Result<i32> where Ok merely means "cmd.exe ran" - and the only caller drops it with `.map(|_| ())`, so a failing schtasks under requires_system/requires_ti is indistinguishable from success.

**Failing scenario.** A requires_system tweak with `scheduler_changes: [{task_path: "\\Microsoft\\Windows\\Foo", task_name: "NoSuchTask", action: disable}]`. run_schtasks_as_system builds `schtasks /Change /TN "..." /Disable`, cmd.exe runs it, schtasks exits 1 ("ERROR: The system cannot find the file specified."), execute_command_as_system returns Ok(1). helpers.rs:876-881 does `.map(|_| ())` -> Ok(()). The atomic set completes, the snapshot metadata is updated, the UI shows the option applied - but the task was never disabled, and status detection on the next load disagrees, so the tweak flip-flops. Same at helpers.rs:774-780 for the pattern path. Note this also makes the `ignore_not_found` string check at helpers.rs:887 dead code on the elevated path, because no error string is ever produced.

**Evidence.**

```
src-tauri/src/services/elevation/ti_elevation.rs:468-472:
'''
pub fn run_schtasks_as_ti(args: &str) -> Result<i32, Error> {
    log::info!("Running schtasks as TrustedInstaller: {}", args);
    let command = format!("schtasks {}", args);
    execute_command_as_trusted_installer(&command)
}
'''
src-tauri/src/services/elevation/ti_elevation.rs:115-119 (same shape for run_schtasks_as_system)
Caller, src-tauri/src/commands/tweaks/helpers.rs:875-881:
'''
        if use_ti {
            trusted_installer::run_schtasks_as_ti(&schtasks_args)
        } else {
            trusted_installer::run_schtasks_as_system(&schtasks_args)
        }
        .map(|_| ())
        .map_err(|e| Error::CommandExecution(e.to_string()))
'''
Contrast, src-tauri/src/services/elevation/service_ops.rs:54-64:
'''
    let exit_code = execute(&command)?;
    if exit_code == 0 { ... } else {
        Err(Error::ServiceControl(format!("sc config failed with exit code: {}", exit_code)))
'''
```

**Fix.** Change run_schtasks_as_ti/run_schtasks_as_system to return Result<(), Error>, checking exit_code == 0 internally and mapping schtasks' not-found exit to a distinguishable Error variant (not a substring). That removes the caller's ability to drop the signal and makes ignore_not_found work on the elevated path.

<details><summary>Verifier reasoning</summary>

Every quoted line exists verbatim and no guard rescues it. ti_elevation.rs:468-472 `pub fn run_schtasks_as_ti(args: &str) -> Result<i32, Error> { ... execute_command_as_trusted_installer(&command) }` and ti_elevation.rs:114-118 (run_schtasks_as_system) both tail-return the executor with NO exit-code check. Both call sites drop it: helpers.rs:779 and helpers.rs:880 `.map(|_| ())`. I confirmed the contrast is real and deliberate elsewhere - service_ops.rs:54-64 `let exit_code = execute(&command)?; if exit_code == 0 {...} else { Err(Error::ServiceControl(...)) }` and system_elevation.rs:199-205 do check; even run_powershell_as_system (ti_elevation.rs:100-110) at least inspects it, and helpers.rs:104-134 errors on nonzero. schtasks is the sole outlier. Exit code does propagate (execute_command_as_system wraps in `cmd.exe /c` at system_elevation.rs:31, and cmd /c returns the child's code), so Ok(1) is genuinely reachable and genuinely discarded. Tried to refute on reachability and FAILED: windows_update.yaml:160-166 `id: windows_update_mode` / `requires_ti: true` owns the option at line 270-275 with the scheduler pattern change, and helpers.rs:741 `if use_elevated {` routes it into exactly the branch that drops the code. This is live in shipped content. The finder's corollary is also correct: ignore_not_found at helpers.rs:887-890 matches on error substrings that can never be produced on the elevated path.

</details>

---

### [HIGH · CONFIRMED] firewall_service::rule_exists ignores netsh's exit status and matches an English-only string, so create_firewall_rule silently no-ops

`src-tauri/src/services/firewall_service.rs:25` — lens: code-defect

**What is wrong.** rule_exists never inspects output.status and decides existence purely by whether stdout contains the literal English phrase "No rules match the specified criteria"; any locale change or any netsh failure makes the phrase absent, which is interpreted as "the rule exists".

**Failing scenario.** Non-English Windows (a large share of users): `netsh advfirewall firewall show rule name=MagicX_BlockTelemetry` prints the localized equivalent of "No rules match the specified criteria" and exits 1 - verified on this host that the exit code is 1 and the message is the only signal:
```
netsh advfirewall firewall show rule name=__MagicXNoSuchRule__
No rules match the specified criteria.
exit=1
```
With a localized message, `!stdout.contains(...)` -> rule_exists returns true -> create_firewall_rule takes the early return at line 42-45 and returns Ok(()) WITHOUT creating the rule. The atomic set succeeds, the snapshot is written, the UI shows the tweak applied, and no firewall rule exists. The mirror failure: delete_firewall_rule believes a nonexistent rule exists, runs netsh delete, which fails, and the whole option rolls back. The same false-positive occurs locale-independently whenever netsh itself fails (mpssvc stopped, third-party firewall, access denied), since stdout is then empty.

**Evidence.**

```
src-tauri/src/services/firewall_service.rs:22-28:
'''
        .output()
        .map_err(|e| Error::CommandExecution(format!("Failed to query firewall rule: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // If the rule doesn't exist, netsh returns "No rules match the specified criteria"
    Ok(!stdout.contains("No rules match the specified criteria"))
'''
src-tauri/src/services/firewall_service.rs:41-45:
'''
    // Check if rule already exists
    if rule_exists(&change.name)? {
        log::debug!("Firewall rule already exists: {}", change.name);
        return Ok(());
    }
'''
```

**Fix.** Decide existence from netsh's exit status (0 = at least one rule matched, non-zero = none), and treat any other failure mode as an error rather than as "exists". If stdout must be inspected, use `netsh -f`/PowerShell `Get-NetFirewallRule -Name` which returns structured, locale-independent results. Also: a name collision with a user-created rule of the same name means create silently skips and revert deletes the user's rule - the existence check should compare rule identity, not just the name.

**Verifier correction.** The English-string dependency is only one of two triggers; the locale-independent one is stronger and should lead: because output.status is never inspected, any netsh invocation that fails or emits nothing to stdout is interpreted as "the rule exists". Additionally the false positive is not confined to create/delete -- rule_exists is the only oracle for firewall status detection, inspection, snapshot capture and restore, so a stuck-true also produces a wrong 'applied' status and makes revert silently leave the created rule behind.

<details><summary>Verifier reasoning</summary>

Opened src-tauri/src/services/firewall_service.rs:13-29. The quoted lines exist verbatim: the function binds only `output.stdout` (line 25) and returns `Ok(!stdout.contains("No rules match the specified criteria"))` (line 28). `output.status` is never read anywhere in the function; `status.success()` is only used later at lines 123 and 155 for add/delete, not here. I looked for the guard the finder might have missed and there is none: create_firewall_rule:41-45 early-returns Ok(()) on a true result, and the caller apply_firewall_changes_atomic (src-tauri/src/commands/tweaks/helpers.rs:994-1010) does NOT discard the Result -- it propagates non-skip_validation errors, so the create path really does silently succeed-without-creating while the atomic set reports success. Executed on this host: `netsh advfirewall firewall show rule name=__MagicXNoSuchRule__` -> "No rules match the specified criteria." with exit=1, confirming netsh signals absence via a non-zero exit that the code throws away, so ANY netsh failure with empty stdout (mpssvc stopped, access denied) is read as "exists" locale-independently. Impact is broader than the finder stated: rule_exists is the sole oracle for status detection (detection.rs:487, 684), inspection (inspection.rs:408), snapshot capture (capture.rs:295, 489) and restore (restore.rs:387, 396), so a stuck-true result also yields a wrong "applied" status and a capture of existed=true that makes restore_firewall_state (restore.rs:384-389) merely warn instead of removing the rule the tweak created. Not critical: no pre-existing user state is destroyed and the machine is not wedged. Note on reachability: no shipped tweak YAML in src-tauri/tweaks/*.yaml currently uses firewall_changes (grep count 0 across all 9 files), but it is a documented authoring feature (docs/TWEAK_AUTHORING.md:755, 784, 793), so this is live for any authored rule.

</details>

---

### [HIGH · CONFIRMED] Service and task state detection parse English-only labels and values from sc.exe and schtasks.exe output

`src-tauri/src/services/scheduler_service.rs:44` — lens: code-defect — *severity adjusted by verifier: medium → high*

**What is wrong.** TaskState::from_str only recognizes "ready"/"disabled"/"running" and get_task_state only recognizes a line starting with "Status:"; parse_service_state only recognizes a line starting with "STATE" - all of these are localized strings emitted by the OS tools, so on non-English Windows every state resolves to Unknown.

**Failing scenario.** German/French/Japanese Windows. schtasks /Query /FO LIST /V prints the Status field localized (e.g. `Status: Bereit` / `État : Prêt`). get_task_state either fails the `starts_with("Status:")` test entirely and returns TaskState::Unknown("Could not parse state"), or matches the label and produces TaskState::Unknown("Bereit"). Either way a scheduler_change with `action: disable` can never be detected as satisfied, so per the state-detection contract the option never becomes current_option_index and the UI permanently shows "System Default" for every scheduler tweak, prompting the user to re-apply a tweak that is already applied. Parallel effect for services: parse_service_state returns None -> ServiceState::Unknown -> is_service_running() returns false -> stop_service takes the early return at service_control.rs:243-246 and never issues `net stop`, so a service the tweak promised to stop keeps running while the apply reports success.

**Evidence.**

```
src-tauri/src/services/scheduler_service.rs:43-50:
'''
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "ready" => TaskState::Ready,
            "disabled" => TaskState::Disabled,
            "running" => TaskState::Running,
            _ => TaskState::Unknown(s.to_string()),
'''
src-tauri/src/services/scheduler_service.rs:92: `if line.starts_with("Status:") {`
src-tauri/src/services/service_control.rs:102-110:
'''
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("STATE") {
'''
src-tauri/src/services/service_control.rs:243-246 (the consequence):
'''
    if let Ok(false) = is_service_running(service_name) {
        log::info!("Service '{}' is not running, skipping stop.", service_name);
        return Ok(());
    }
'''
```

**Fix.** Replace the sc.exe/schtasks.exe text scraping with API calls: QueryServiceStatusEx (already imported in elevation/common.rs) for service state, and the Task Scheduler COM API (or `Get-ScheduledTask | Select State` with `-OutputFormat`/JSON) for task state. If shelling out must stay, key on the locale-invariant numeric state code that sc prints before the word (`STATE : 4 RUNNING` -> parse the 4) rather than the English word, and never key on the localized field label.

**Verifier correction.** The finding understates the impact. Beyond wrong status, REVERT silently loses the captured task state on non-English Windows: capture.rs:254 stores `task.state.as_str()` (for a localized parse that is the raw localized word, e.g. "Bereit"), and restore.rs:301 matches only `"Ready" | "Running"` / "Disabled" / "NotFound", falling through to restore.rs:344 `_ => log::warn!("Unknown scheduler state '{}' for task '{}', skipping restore")` and returning `Ok(())`. Because that Ok keeps `restore_result.success` true, apply.rs:271 then calls `delete_snapshot(&tweak_id)` — the task stays disabled and the snapshot is destroyed. Also note the service half does NOT break status detection: check_services_match (detection.rs:314-326) compares only `startup_type`, which comes from `reg query ... /v Start` parsed on the locale-invariant tokens "Start"/"REG_DWORD" (service_control.rs:134). The service consequence is limited to the stop/start no-op the finding describes.

<details><summary>Verifier reasoning</summary>

Verified every anchor. scheduler_service.rs:44-48 matches only "ready"/"disabled"/"running"; :92 and :246 key on the literal "Status:" label; service_control.rs:104 keys on "STATE" and :110 on the English word (parts[1]), not the locale-invariant numeric code in parts[0]. Grepped the whole src tree for chcp/LANG/locale/OutputEncoding: no matches, so nothing forces English output. Consumption chain confirmed: detection.rs:411-431 -> task_state_matches (backup/helpers.rs:58-65) where Unknown(x) vs Disabled is never equal, so a disable option can never be current -> permanent "System Default". Service chain confirmed: parse_service_state returns None -> ServiceState::Unknown (service_control.rs:87) -> is_service_running false -> stop_service early-returns at :243-246, and helpers.rs:627 discards the result with `let _ =` anyway. Attempted refutations (a guard forcing English, a numeric-code fallback, an alternate API path, a caller that ignores TaskState) all failed.

</details>

---

### [HIGH · CONFIRMED] Each of the five effect services - plus a second, parallel elevated implementation of three of them - invents its own definition of "did it work", and the atomic-rollback guarantee is only as strong as the weakest one

`src-tauri/src/services/elevation/service_ops.rs:30` — lens: design-critique

**What is wrong.** There is no shared apply/verify/rollback trait. The layer above promises "complete success or complete rollback" over registry+service+scheduler+hosts+firewall, but the five services signal success through five incompatible mechanisms, and registry/service/scheduler each have a SECOND elevated implementation with a sixth mechanism, selected at runtime by tweak.requires_system.

**Failing scenario.** Concretely, the success signals in this layer are: registry_service - typed winreg io::Error with a NotFound discriminant (reliable); service_control - process exit status plus English-and-numeric stderr substrings; scheduler_service - exit status plus stderr substrings, with query failures silently downgraded to "empty"; firewall_service - stdout English substring with exit status ignored entirely; hosts_service - filesystem errors only, with no verification that the entry landed. The elevated mirror is worse: execute_command_as_system/execute_command_as_trusted_installer hard-code hStdOutput/hStdError to null (system_elevation.rs:55-57, ti_elevation.rs:338-340) and return only an i32, so an elevated caller structurally cannot distinguish "not found" from "access denied" from "succeeded" beyond a single number - which is exactly why service_ops.rs collapsed net's error code 2 into success and why the schtasks helpers leaked a raw i32 that the caller then dropped. The practical consequence is that four of the findings above (elevated schtasks, net exit 2, netsh rule_exists, list_tasks_in_folder) are the SAME bug re-derived in four places: a per-tool ad-hoc guess at what failure looks like. Every one of them converts a failed change into a reported success, which is the precise condition that makes the atomic-set guarantee vacuous - rollback cannot fire for a failure nobody detected, and the snapshot is then written as if the option were applied. This will keep recurring as new change types are added, because there is no single place that defines the contract.

**Evidence.**

```
Divergent success signals in one layer:
src-tauri/src/services/firewall_service.rs:28: `Ok(!stdout.contains("No rules match the specified criteria"))` (exit status never read)
src-tauri/src/services/scheduler_service.rs:212: `if stderr.contains("does not exist") || stderr.contains("cannot find") {` -> `return Ok(Vec::new());`
src-tauri/src/services/service_control.rs:225: `if !stderr.contains("already been started") && !stderr.contains("2182") {`
src-tauri/src/services/elevation/service_ops.rs:89: `if exit_code == 0 || exit_code == 2 {`
src-tauri/src/services/elevation/service_ops.rs:30: `pub type CommandExecutor = fn(&str) -> Result<i32, Error>;` (the only shared abstraction is "a string in, an int out")
src-tauri/src/services/elevation/system_elevation.rs:55-57 (child output is discarded by construction):
'''
            hStdInput: ptr::null_mut(),
            hStdOutput: ptr::null_mut(),
            hStdError: ptr::null_mut(),
'''
src-tauri/src/services/hosts_service.rs:127-131 (no post-write verification at all):
'''
    file.write_all(content.as_bytes())
        .map_err(|e| Error::WindowsApi(format!("Failed to write to hosts file: {}", e)))?;

    log::info!("Added hosts entry: {} -> {}", domain, ip);
    Ok(())
'''
```

**Fix.** Introduce a `trait Change { fn capture(&self) -> Snapshot; fn apply(&self) -> Result<()>; fn verify(&self) -> Result<bool>; fn restore(&self, Snapshot) -> Result<()>; }` and implement it once per change kind, with the elevated/non-elevated split hidden behind the impl rather than exposed to helpers.rs. Make apply() unconditionally call verify() before returning Ok - that single rule would have caught the netsh no-op, the elevated schtasks no-op and the net-exit-2 no-op without needing per-tool error taxonomies. Also give the elevated executors real stdout/stderr pipes (CreatePipe + bInheritHandles) so the elevated path has the same diagnostic fidelity as the direct path, and return a structured result rather than a bare i32.

**Verifier correction.** One mechanism claim is imprecise: hStdOutput/hStdError being null is NOT what discards child output — `dwFlags: STARTF_USESHOWWINDOW` (system_elevation.rs:51, ti_elevation.rs:334) omits STARTF_USESTDHANDLES, so those fields are ignored by the OS. The correct statement is that no pipes are created at all, so `execute_command_as_system` / `execute_command_as_trusted_installer` structurally can only return `Result<i32, Error>` and the elevated path has no stderr to key on. The conclusion is unchanged. The sharpest concrete instance is helpers.rs:774-780: `trusted_installer::run_schtasks_as_ti(&schtasks_args) ... .map(|_| ())` — the i32 exit code from the elevated schtasks is discarded, so a failed elevated task disable is reported as applied and never triggers rollback (restore.rs:307-309 does check the same exit code, proving the pattern is known elsewhere in the codebase).

<details><summary>Verifier reasoning</summary>

Every cited divergence is real and I verified each line: firewall_service.rs:25-28 reads only stdout and never `output.status`; scheduler_service.rs:210-215 downgrades a failed query to `Ok(Vec::new())`; service_control.rs:225 and :261 key on English stderr substrings; service_ops.rs:30 is the only shared abstraction (`fn(&str) -> Result<i32, Error>`) and :89/:125 collapse `net`'s generic error code 2 into success; hosts_service.rs:127-131 writes with no read-back. `rg '^\s*(pub )?trait \w+' src-tauri/src` returns no matches — there is literally no trait in the backend, confirming "no shared apply/verify/rollback contract". The duplicate-elevated-implementation claim also holds (set_registry_value_as_system at system_elevation.rs:158 shells `reg add`, run_schtasks_as_* at ti_elevation.rs:115/468, service_ops.rs for services). Refutation attempt: I looked for a post-apply verification step in apply.rs — steps 4-9 (apply.rs:152-198) go straight from `apply_all_changes_atomically` Ok to snapshot-metadata update and success, with no re-detection, so an undetected no-op is indistinguishable from success. Design-critique lens, so no failing input required, but a concrete one exists at helpers.rs:779.

</details>

---

### [MEDIUM · CONFIRMED] set_registry_value_as_system double-quotes and %-mangles the value, so REG_SZ/REG_EXPAND_SZ values with a space or %VAR% are written wrong or fail

`src-tauri/src/services/elevation/system_elevation.rs:194` — lens: code-defect — *severity adjusted by verifier: high → medium*

**What is wrong.** reg_exe_data() already wraps string values in double quotes, escape_shell_arg then doubles every quote and every percent, and the result is interpolated after `/d` with no quoting of its own - producing `/d ""C:\Program Files\x""` which reg.exe's argv parser splits at the space, and `%%SystemRoot%%` which cmd.exe still expands.

**Failing scenario.** Any tweak with `requires_system: true` (or requires_ti, which implies it) and `value_type: REG_SZ`, `value: "C:\\Program Files\\App\\x.exe"`. helpers.rs:339-346 passes tweak.requires_system to write_registry_value, which routes to set_registry_value_as_system. Command becomes `reg add "HKLM\..." /v "Name" /t REG_SZ /d ""C:\Program Files\App\x.exe"" /f`. Verified live on this host that the doubled-quote form breaks reg.exe's argument parsing:
```
== control ==
reg query "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion" /v ProductName  -> exit=0
== doubled-quote (escape_shell_arg output shape) ==
reg query ""HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion"" /v ProductName
ERROR: Invalid syntax.  exit=1
```
So the write fails, the atomic set rolls back, and the tweak can never be applied. Separately for REG_EXPAND_SZ `value: "%SystemRoot%\\foo"`, escape_shell_arg emits `%%SystemRoot%%` and cmd still expands the inner pair - verified: `cmd /c "echo A%%SystemRoot%%B"` prints `A%C:\WINDOWS%B` - so the stored value becomes the literal `%C:\WINDOWS%\foo` instead of `%SystemRoot%\foo`. Also verified `cmd /c "echo 100%%"` prints `100%%`, i.e. the %-doubling is pure corruption at command-line (non-batch) scope. The non-elevated winreg path writes all of these correctly, so the same YAML behaves differently depending on requires_system.

**Evidence.**

```
src-tauri/src/services/elevation/system_elevation.rs:187-197:
'''
    let escaped_key = escape_shell_arg(&full_key);
    let escaped_value_name = escape_shell_arg(value_name);
    let escaped_value_data = escape_shell_arg(value_data);
...
    let command = format!(
        "reg add \"{}\" /v \"{}\" /t {} /d {} /f",
        escaped_key, escaped_value_name, value_type, escaped_value_data
    );
'''
src-tauri/src/services/registry_value.rs:32-34 (value already quoted before escaping):
'''
            RegistryValue::String(value) | RegistryValue::ExpandString(value) => {
                format!("\"{}\"", value)
            }
'''
src-tauri/src/services/elevation/common.rs:75-87:
'''
            '"' => escaped.push_str("\"\""), // Double quotes
...
            '%' => {
                escaped.push('%'); // Escape percent with percent
                escaped.push('%');
            }
'''
```

**Fix.** Stop hand-building a cmd.exe string for registry writes. Either (a) have the elevated helper spawn reg.exe directly via CreateProcessWithTokenW with a properly MSVC-quoted command line (backslash-escape embedded quotes, wrap each argument in a single pair of quotes, and do not touch %), or (b) drop reg.exe entirely and impersonate the SYSTEM token around the existing winreg calls. At minimum: remove the quoting from reg_exe_data, quote /d exactly once, and delete the %-doubling branch from escape_shell_arg.

**Verifier correction.** Both failure modes reproduce exactly as claimed, but the path is currently latent: no shipped tweak with requires_system/requires_ti uses a string-typed registry value (the only elevated tweaks are ui.yaml disable_widgets_win11 - REG_DWORD only - and windows_update.yaml windows_update_mode - no string values). The exposed Tauri command apply_registry_as_system (lib.rs:128) is not called from the frontend (no matches in src/). Downgraded high -> medium as a latent infrastructure defect, not something a user hits today.

<details><summary>Verifier reasoning</summary>

Verified the whole chain by reading, then proved it by execution. registry_value.rs:32-34 `RegistryValue::String(value) | RegistryValue::ExpandString(value) => { format!("\"{}\"", value) }` feeds registry_value.rs:122 `&value.reg_exe_data()` into set_registry_value_as_system, which at system_elevation.rs:189 does `let escaped_value_data = escape_shell_arg(value_data);` and at system_elevation.rs:193-196 interpolates it bare after `/d`. common.rs:76 `'"' => escaped.push_str("\"\"")` and common.rs:81-84 `'%' => { push('%'); push('%'); }` confirmed verbatim. Executed both halves live against a scratch HKCU key: (a) `reg add "KEY" /v P1 /t REG_SZ /d ""C:\Program Files\App\x.exe"" /f` -> `ERROR: Invalid syntax.` EXIT=1 while the correctly single-quoted form -> EXIT=0; (b) `/d ""%%SystemRoot%%\foo""` for REG_EXPAND_SZ -> EXIT=0 but READ BACK AS `%C:\WINDOWS%\foo` - silent corruption, the worse half. Also established the blast radius the finder did not state: a doubled-quote value with NO space succeeds and stores correctly (`""NoSpaces""` -> `NoSpaces`), so only values containing a space (hard failure + rollback) or a percent (silent corruption) are affected. Note the key itself is safe - the format string wraps it in its own quote pair. Scratch key deleted.

</details>

---

### [MEDIUM · CONFIRMED] run_powershell_as_system/as_ti escape quotes with a C-runtime escape that cmd.exe does not honor, letting a script with quotes and & break out into a separate SYSTEM/TI command

`src-tauri/src/services/elevation/ti_elevation.rs:92` — lens: code-defect — *severity adjusted by verifier: high → medium*

**What is wrong.** The script is escaped with `replace('"', "\\\"")` and wrapped in quotes, but the resulting string is first parsed by cmd.exe (execute_command_as_system prepends `cmd.exe /c`), and cmd.exe treats every `"` as a quote-state toggle regardless of a preceding backslash - so an odd arrangement of quotes leaves shell metacharacters outside quoted state.

**Failing scenario.** A tweak with `post_powershell: 'Write-Output "a&b"'` and requires_system. escaped_script = `Write-Output \"a&b\"`. Full command line handed to cmd: `cmd.exe /c powershell.exe ... -Command "Write-Output \"a&b\""`. cmd's quote toggling: quote #1 (wrapper) opens, quote #2 (from the first \") closes -> the following `a` and then `&` are OUTSIDE quoted state -> cmd splits at `&` and runs two commands: a truncated `powershell ... -Command "Write-Output \"a` (which hangs or errors on an unterminated string) and then `b\""` as a second command executed with the SYSTEM/TrustedInstaller token. The PowerShell script is silently truncated and an unintended process is launched at SYSTEM. The same defect is in run_powershell_as_ti. Note escape_shell_arg - the module's own escaping helper - is not used on this path at all.

**Evidence.**

```
src-tauri/src/services/elevation/ti_elevation.rs:91-98:
'''
    // Escape double quotes in the script for the command line
    let escaped_script = script.replace('"', "\\\"");

    // Build the full command to run PowerShell as SYSTEM
    let command = format!(
        "powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command \"{}\"",
        escaped_script
    );
'''
src-tauri/src/services/elevation/ti_elevation.rs:457-462 (identical bug for TI)
src-tauri/src/services/elevation/system_elevation.rs:31 (the cmd.exe wrapper that does the parsing):
'''
    let full_command = format!("cmd.exe /c {}", command_line);
'''
```

**Fix.** Do not route PowerShell through cmd.exe. Give execute_command_as_system/execute_command_as_trusted_installer an argv-style entry point that builds the command line with MSVC quoting rules (CreateProcess needs no shell), invoking powershell.exe directly. Better still, pass the script as a Base64 `-EncodedCommand` so no quoting is required at any layer.

**Verifier correction.** The parsing defect and the stray second command are real and I reproduced them exactly, but the 'attacker-controlled / privilege escalation' framing is wrong: the script source is the app's own compile-time-embedded YAML, and that script is ALREADY executing at SYSTEM/TI by design, so no privilege boundary is crossed - an author who wanted a second SYSTEM command could simply write one. Additionally NO shipped tweak uses pre_powershell or post_powershell at all (zero matches across src-tauri/tweaks/), so this is a latent correctness defect in elevated command construction. Downgraded high -> medium.

<details><summary>Verifier reasoning</summary>

Lines are verbatim: ti_elevation.rs:92 `let escaped_script = script.replace('"', "\\\"");` with the wrapper at :95-98, the identical construct at ti_elevation.rs:458-462 for TI, and the cmd.exe wrapper at system_elevation.rs:31 `let full_command = format!("cmd.exe /c {}", command_line);`. I tried to refute by arguing cmd honors the backslash escape - it does not. Executed the exact constructed line for script `Write-Output "a&b"`: cmd truncated the inner script (PowerShell reported `The string is missing the terminator: ".`) AND split at the ampersand, attempting a second command - `'b\""' is not recognized as an internal or external command`. That second token is executed with the elevated token, confirming the mechanism. Partial mitigation the finder omitted: unlike the schtasks path, helpers.rs:104-134 DOES check the PowerShell exit code and returns Error::CommandExecution on nonzero, so the tweak fails loudly and rolls back - but only after the stray command has already run. escape_shell_arg is indeed unused on this path (confirmed, the module re-exports it at mod.rs:31 and only service_ops/system_elevation use it).

</details>

---

### [MEDIUM · CONFIRMED] remove_hosts_entry deletes the entire hosts line for a multi-hostname entry, and revert restores only the single ip/domain pair

`src-tauri/src/services/hosts_service.rs:186` — lens: code-defect

**What is wrong.** The removal match keys on the FIRST hostname on the line but removes the whole line, so every additional alias the user had on that line is destroyed; the restore path only re-adds `ip<TAB>domain`, so the aliases are permanently lost.

**Failing scenario.** User's hosts contains `127.0.0.1  localhost  localhost.localdomain  devbox.internal`. A tweak option with `hosts_changes: [{action: remove, ip: "127.0.0.1", domain: "localhost"}]` is applied. At hosts_service.rs:183-192, parts[0]=="127.0.0.1" and parts[1].split_whitespace().next()=="localhost" match, so `i += 1; continue;` drops the ENTIRE line including localhost.localdomain and devbox.internal - the user's dev hostname stops resolving. On revert, restore_hosts_state sees existed=true and calls add_hosts_entry("127.0.0.1", "localhost", None), which appends only `127.0.0.1\tlocalhost` under a MagicX marker. localhost.localdomain and devbox.internal are gone with no record of them anywhere.

**Evidence.**

```
src-tauri/src/services/hosts_service.rs:182-192:
'''
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            let parts: Vec<&str> = trimmed.splitn(2, |c: char| c.is_whitespace()).collect();
            if parts.len() >= 2 {
                let line_ip = parts[0];
                let line_domain = parts[1].split_whitespace().next().unwrap_or("");

                if line_ip == ip && line_domain.eq_ignore_ascii_case(domain) {
                    log::info!("Removing hosts entry: {} -> {}", domain, ip);
                    i += 1;
                    continue;
'''
src-tauri/src/services/backup/restore.rs:360-361:
'''
        if !currently_exists {
            hosts_service::add_hosts_entry(&snapshot.ip, &snapshot.domain, None)?;
'''
```

**Fix.** When the target domain is one of several on a line, rewrite the line with that hostname removed rather than deleting the line; only delete the line when it becomes hostname-empty. And snapshot the original line text (not just an existed flag) so restore can put the exact bytes back.

<details><summary>Verifier reasoning</summary>

Traced the real code. src-tauri/src/services/hosts_service.rs:183 `let parts: Vec<&str> = trimmed.splitn(2, |c: char| c.is_whitespace()).collect();` splits into exactly two pieces, :186 `let line_domain = parts[1].split_whitespace().next().unwrap_or("");` keys on the FIRST hostname only, and the match at :188 leads to `i += 1; continue;` at :190-191, which drops the whole line -- there is no code path anywhere in the function that rewrites a line with one hostname stripped. For `127.0.0.1  localhost  localhost.localdomain  devbox.internal` with a remove of 127.0.0.1/localhost, parts[0]=="127.0.0.1" and line_domain=="localhost" match, so localhost.localdomain and devbox.internal are destroyed. The unrecoverability half also checks out: capture.rs:278 stores only `existed`, and restore.rs:356-367 re-adds via `add_hosts_entry(&snapshot.ip, &snapshot.domain, None)` which formats `format!("{}\t{}", ip, domain)` (hosts_service.rs:108) -- the aliases exist nowhere in the snapshot. I looked for a guard requiring exactly two tokens and there is none (`parts.len() >= 2`). Severity stays medium rather than high because the trigger is narrower than 'every multi-hostname entry': it requires the tweak's target domain to be the FIRST hostname on a user line that carries additional aliases, and no shipped tweak uses hosts_changes at all (0 hits across src-tauri/tweaks/*.yaml).

</details>

---

### [MEDIUM · CONFIRMED] read_hosts_file and remove_hosts_entry parse the domain field differently, so entry_exists false-negatives cause duplicate appends and a wrong "not applied" status

`src-tauri/src/services/hosts_service.rs:55` — lens: code-defect

**What is wrong.** read_hosts_file's inline-comment branch takes everything before the '#' as the domain without splitting on whitespace, while remove_hosts_entry takes only the first whitespace-delimited token - the two views of the same file disagree for any line that has both multiple hostnames and a trailing comment.

**Failing scenario.** Hosts contains `0.0.0.0 www.tracker.com tracker.com   # added by my adblocker`. Tweak: `hosts_changes: [{action: add, ip: "0.0.0.0", domain: "www.tracker.com"}]`. read_hosts_file finds '#' in `rest`, so domain = `"www.tracker.com tracker.com"` (the whole span before the comment, only trimmed). entry_exists("0.0.0.0", "www.tracker.com") compares against that concatenation -> false. add_hosts_entry therefore appends a duplicate `# MagicX Toolbox` / `0.0.0.0\twww.tracker.com` block even though the mapping already existed. The same false negative drives status: inspection.rs:380 uses entry_exists, so the option shows as not-applied while the effective DNS behavior is already what the tweak wants (UI reports the wrong state). Meanwhile remove_hosts_entry, which uses the other parser, WILL match and delete the user's original commented line - the two functions disagree about the same line.

**Evidence.**

```
src-tauri/src/services/hosts_service.rs:54-59:
'''
            // Extract domain (strip inline comments)
            let domain = if let Some(hash_pos) = rest.find('#') {
                rest[..hash_pos].trim().to_string()
            } else {
                rest.split_whitespace().next().unwrap_or("").to_string()
            };
'''
vs src-tauri/src/services/hosts_service.rs:186:
'''
                let line_domain = parts[1].split_whitespace().next().unwrap_or("");
'''
Consumer of the divergence, src-tauri/src/services/hosts_service.rs:71-76:
'''
pub fn entry_exists(ip: &str, domain: &str) -> Result<bool, Error> {
    let entries = read_hosts_file()?;
    Ok(entries
        .iter()
        .any(|e| e.ip == ip && e.domain.eq_ignore_ascii_case(domain)))
'''
```

**Fix.** Extract one parse_hosts_line(&str) -> Option<(ip, Vec<hostname>)> helper that strips the inline comment first and then splits all hostnames, and have read_hosts_file, entry_exists and remove_hosts_entry all use it. entry_exists should match if the domain appears anywhere in the hostname list.

**Verifier correction.** The wrong-status consequence is wider than the single anchor given: entry_exists also drives backup/detection.rs:467 (option matching) and detection.rs:672 (startup stale-snapshot detection), not just inspection.rs:380.

<details><summary>Verifier reasoning</summary>

Both sides of the divergence are verbatim at the cited lines. read_hosts_file, hosts_service.rs:55-59, takes `rest[..hash_pos].trim().to_string()` when a '#' is present -- no whitespace split -- while remove_hosts_entry, hosts_service.rs:186, takes `parts[1].split_whitespace().next()`. Hand-traced `0.0.0.0 www.tracker.com tracker.com   # added by my adblocker`: rest = "www.tracker.com tracker.com   # added by my adblocker", find('#') is Some, so domain becomes the concatenation "www.tracker.com tracker.com"; entry_exists (hosts_service.rs:71-76) compares with `e.domain.eq_ignore_ascii_case(domain)` against "www.tracker.com" -> false. add_hosts_entry:89 therefore falls through and appends a duplicate marker+entry block. The status half is confirmed at three consumers, not one: backup/inspection.rs:380, backup/detection.rs:467 (`if exists != expected_exists { return not_matched }`) and detection.rs:672 stale-snapshot matching all call entry_exists, so the option reads as not-applied while the mapping is already effective. The remove-side asymmetry is real too -- remove_hosts_entry's first-token parse DOES match that same line and deletes it. I checked for a normalization step between the two parsers and there is none; they are independent inline parses. Medium is right: the trigger requires a line with BOTH multiple hostnames AND a trailing inline comment, the duplicate append is idempotent after the first apply, and no shipped tweak currently uses hosts_changes.

</details>

---

### [LOW · CONFIRMED] The hosts file is truncated and rewritten in place, converting the whole file's line endings and losing all entries on a crash mid-write

`src-tauri/src/services/hosts_service.rs:201` — lens: code-defect — *severity adjusted by verifier: medium → low*

**What is wrong.** remove_hosts_entry rebuilds the whole file with content.lines()/join("\n") and writes it back with fs::write, which opens with TRUNCATE - there is no temp-file-plus-rename, no backup of the original file bytes, and the round trip silently rewrites the file from CRLF to LF and drops the trailing newline.

**Failing scenario.** Verified on this host that the real hosts file is CRLF with a UTF-8 BOM: `od -c` on C:\Windows\System32\drivers\etc\hosts shows `357 273 277 # C o p y r i g h t ... \r \n`. Applying any hosts_changes option with `action: remove` rewrites all ~20 lines of the user's file with bare LF terminators. Worse, if the process is killed (or the machine loses power) between fs::write's truncate and the write completing - a real window during an elevated apply that may also be spawning 30s-timeout child processes - the user is left with an empty or half-written hosts file and every custom entry they ever added is gone. capture.rs only snapshots per-entry existence booleans (capture.rs:278 `let existed = hosts_service::entry_exists(...)`), never the file contents, so the loss is unrecoverable by revert.

**Evidence.**

```
src-tauri/src/services/hosts_service.rs:200-203:
'''
    // Write back the modified content
    let new_content = new_lines.join("\n");
    fs::write(&hosts_path, new_content.as_bytes())
        .map_err(|e| Error::WindowsApi(format!("Failed to write hosts file: {}", e)))?;
'''
src-tauri/src/services/hosts_service.rs:145: `let lines: Vec<&str> = content.lines().collect();` (strips \r, so the join reintroduces only \n)
src-tauri/src/services/backup/capture.rs:278: `let existed = hosts_service::entry_exists(&change.ip, &change.domain)?;` (only a boolean is snapshotted)
```

**Fix.** Write to a sibling temp file in the same directory, flush+sync it, then ReplaceFileW/rename over the original. Detect the dominant line ending from the input and preserve it (and the trailing newline). Additionally, snapshot the original hosts file bytes into the tweak snapshot on first apply so revert can restore it verbatim.

**Verifier correction.** The deterministic, verified defect is the CRLF->LF normalization of the entire user file plus loss of the trailing newline on every remove; that is cosmetic for the Windows resolver. The 'loses all entries on a crash' claim is a real robustness gap (truncating write, no temp-file+rename, no full-file backup) but was not demonstrated and the window is a single fs::write, not the multi-second span the scenario implies.

<details><summary>Verifier reasoning</summary>

Lines quoted are verbatim: src-tauri/src/services/hosts_service.rs:201-203 `let new_content = new_lines.join("\n"); fs::write(&hosts_path, new_content.as_bytes())`, and :145 `let lines: Vec<&str> = content.lines().collect();`. `lines()` strips the \r, `join("\n")` reintroduces only \n and emits no trailing terminator, and fs::write opens with truncate -- so the round-trip mutation is deterministic, not hypothetical. I verified the input bytes on this host: `od -c C:\Windows\System32\drivers\etc\hosts` shows `357 273 277 # C o p y r i g h t ... \r \n` and ends `# End of section \r \n`, i.e. BOM + CRLF + trailing newline. So every `action: remove` rewrites the user's whole file to LF and drops the final newline. capture.rs:278 does store only a bool, confirmed. I tried to refute via an elevated/indirect write path and found none -- helpers.rs:944 calls hosts_service::apply_hosts_change directly in-process. What does NOT survive is the severity framing: the Windows resolver parses LF-terminated hosts lines fine, the BOM is preserved (it rides on the first line's text), and the 'killed between truncate and write' window is a single fs::write call that does not interleave with the 30s child-process spawns the finder invokes -- that harm is asserted, not demonstrated. No shipped tweak uses hosts_changes (grep count 0 across src-tauri/tweaks/*.yaml), though it is a documented feature (docs/TWEAK_AUTHORING.md:703).

</details>

---

### [LOW · CONFIRMED] hosts file reads use read_to_string, so a non-UTF-8 hosts file makes every hosts operation fail, and the UTF-8 BOM produces a bogus parsed entry

`src-tauri/src/services/hosts_service.rs:35` — lens: code-defect

**What is wrong.** fs::read_to_string rejects any byte sequence that is not valid UTF-8, and the code never strips the UTF-8 BOM that Windows ships on this file, so the first line is mis-parsed as a data line.

**Failing scenario.** (a) A user on a CJK or Cyrillic locale who added an ANSI-encoded comment to hosts: read_to_string returns InvalidData, so read_hosts_file, entry_exists, add_hosts_entry and remove_hosts_entry all return Error::WindowsApi. Inside an atomic set that means the hosts step fails and the entire option rolls back - the tweak is simply unusable and the error message ("Failed to read hosts file: stream did not contain valid UTF-8") gives no hint why. (b) BOM: verified this host's hosts file begins `357 273 277` (EF BB BF). The first line becomes "\u{FEFF}# Copyright (c) 1993-2009 Microsoft Corp."; U+FEFF is not Unicode White_Space so `trimmed.starts_with('#')` at line 44 is false, the line is parsed as data, and a junk HostsEntry{ip: "\u{FEFF}#", domain: "Copyright"} is pushed into the entry list.

**Evidence.**

```
src-tauri/src/services/hosts_service.rs:35-36:
'''
    let content = fs::read_to_string(&hosts_path)
        .map_err(|e| Error::WindowsApi(format!("Failed to read hosts file: {}", e)))?;
'''
src-tauri/src/services/hosts_service.rs:41-46:
'''
        let trimmed = line.trim();

        // Skip empty lines and comments (including MagicX markers)
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
'''
Observed file header on this machine: `0000000 357 273 277   #       C   o   p   y   r   i   g   h   t`
```

**Fix.** Read with fs::read and decode lossily (or with an encoding fallback), and strip a leading U+FEFF/EF BB BF before parsing - re-emitting the same BOM on write.

**Verifier correction.** Part (b) is a real mis-parse but has no observable consequence. `read_hosts_file` has exactly one consumer — `entry_exists` (hosts_service.rs:71-76), which requires BOTH `e.ip == ip` and a case-insensitive domain match. The junk entry is `{ip: "\u{FEFF}#", domain: "Copyright"}`, which can never equal a tweak's ip/domain pair, and `remove_hosts_entry` re-parses the file independently and keeps the BOM line (parts[0] != ip). So the surviving defect is only part (a): a non-UTF-8 hosts file makes every hosts read/write path return Error::WindowsApi, which inside an atomic set rolls the whole option back with an opaque message. No data loss, workaround exists (re-save hosts as UTF-8).

<details><summary>Verifier reasoning</summary>

Verified hosts_service.rs:35-36 and :44 verbatim, and independently confirmed this machine's hosts file starts with EF BB BF via `od -c` (`0000000 357 273 277 # C o p y r i g h t`). U+FEFF is not Unicode White_Space, so `str::trim` does not strip it and `starts_with('#')` is false — the BOM line is parsed as data exactly as claimed. Refutation attempt: I grepped for every consumer of read_hosts_file and found only entry_exists, so the junk entry is inert; that kills the practical bite of (b) but not the factual claim, and (a) stands on the literal read_to_string calls at :35, :98 and :142.

</details>

---

### [LOW · PLAUSIBLE] Elevated start/stop service treat net.exe exit code 2 as success, and net returns 2 for essentially every failure

`src-tauri/src/services/elevation/service_ops.rs:89` — lens: code-defect — *severity adjusted by verifier: high → low*

**What is wrong.** stop_service_elevated and start_service_elevated accept exit codes 0 and 2 on the theory that 2 means "already stopped/running", but net.exe returns 2 for invalid service names, access denied, dependency refusal and start failures alike - so these functions can essentially never report a failure.

**Failing scenario.** Verified live. `net start __MagicXNoSuch__` -> "The service name is invalid." exit=2, and `net start Schedule` (already running) -> exit=2. Both are indistinguishable to this code. Concrete tweak failure: a requires_system option with `service_changes: [{name: "WaaSMedicSvc", startup: automatic, start_service: true}]` where the service fails to start (disabled dependency, missing binary, or a typo'd name) - net exits 2 - start_service_elevated logs "Service started (or was already running) as SYSTEM" and returns Ok(()). The atomic set completes, the snapshot metadata records the option as applied, and the service is not running. Because the same code path also swallows a failed `net stop`, a requires_system "disable and stop" option can report full success while the service keeps running.

**Evidence.**

```
src-tauri/src/services/elevation/service_ops.rs:86-99:
'''
    let exit_code = execute(&command)?;

    // net stop returns 0 on success, 2 if already stopped
    if exit_code == 0 || exit_code == 2 {
        log::info!(
            "Service stopped (or was already stopped) as {}",
            elevation.label()
        );
        Ok(())
'''
src-tauri/src/services/elevation/service_ops.rs:122-130 (identical for start):
'''
    // net start returns 0 on success, 2 if already running
    if exit_code == 0 || exit_code == 2 {
'''
Contrast the non-elevated path, src-tauri/src/services/service_control.rs:222-231, which correctly disambiguates with the locale-independent HELPMSG number:
'''
        if !stderr.contains("already been started") && !stderr.contains("2182") {
            return Err(Error::ServiceControl(...))
'''
```

**Fix.** Do not use `net` for the elevated path - use `sc start`/`sc stop`, whose exit code is the Win32 error (1056 = already running, 1062 = not started, 1060 = no such service), so success and the two benign cases are distinguishable. Alternatively capture the child's stdout/stderr (see the design finding) and apply the same 2182/3521 numeric check the non-elevated path already uses.

**Verifier correction.** The exit-code conflation is real and confirmed by live exit codes, but it is currently masked, not amplified: all six call sites discard the Result (helpers.rs:623, 625, 627, 634, 636, 638; restore.rs:274, 279), so start/stop failure never reaches rollback or status regardless of this code. The observable consequence is limited to a log line that falsely claims success. The stated harm (apply reporting success while the service still runs) is caused by the `let _ =` call sites, not by service_ops.rs:89.

<details><summary>Verifier reasoning</summary>

The code claim checks out exactly: src-tauri/src/services/elevation/service_ops.rs:89 `if exit_code == 0 || exit_code == 2 {` and :125 the identical clause for start. I verified the exit codes are genuinely ambiguous by running them: `net start __MagicXNoSuch__` -> exit 2, `net stop __MagicXNoSuch__` -> exit 2, `net start Schedule` (already running) -> exit 2. I also confirmed the exit code reaching this code is the real child's code, not a launcher's: system_elevation.rs:31 builds `cmd.exe /c <command>` and lines 111-121 return GetExitCodeProcess of that process, and cmd /c propagates net's code. The contrast cited is real too -- service_control.rs:222-231 checks stderr for "already been started"/"2182", and I confirmed net.exe writes those messages to stderr (`net start Schedule 2>&1 1>/dev/null` printed "The requested service has already been started. ... NET HELPMSG 2182."), so that check functions. BUT the refutation succeeds on impact: every caller discards the Result. src-tauri/src/commands/tweaks/helpers.rs:623 `let _ = trusted_installer::stop_service_as_ti(&change.name);`, :625, :634, :636, and the non-elevated :627/:638 are all `let _ =`; backup/restore.rs:274 and :279 likewise. Service start/stop is best-effort by construction at the call site. Therefore the finding's concrete scenario -- 'the atomic set completes, the snapshot records the option as applied, and the service is not running' -- is true but is NOT caused by the exit-code-2 acceptance; it happens identically if these functions returned Err. The only observable delta attributable to line 89/125 is a misleading log line ("Service started (or was already running) as SYSTEM"), since the discarded Err would not even be logged. Latent defect, not a user-visible one.

</details>

---

### [LOW · REFUTED] list_tasks_in_folder issues a schtasks query form that always fails, and swallows the failure as "empty folder" - every task_name_pattern tweak is a silent no-op

`src-tauri/src/services/scheduler_service.rs:204` — lens: code-defect — *severity adjusted by verifier: critical → low*

**What is wrong.** schtasks /Query /TN only accepts a task name, never a folder; passing "<folder>\" always exits 1 with "The system cannot find the path specified.", which the code's stderr substring check treats as "folder not found" and returns Ok(empty), so find_tasks_by_pattern can never match anything.

**Failing scenario.** Verified live on this machine. The folder \Microsoft\Windows\Defrag demonstrably exists (`schtasks /Query /FO LIST` shows `Folder: \Microsoft\Windows\Defrag` / `TaskName: \Microsoft\Windows\Defrag\ScheduledDefrag`), yet:
```
schtasks /Query /TN "\Microsoft\Windows\Defrag\" /FO LIST /V
ERROR: The system cannot find the path specified.
exitC=1
```
stderr contains "cannot find" -> line 212-214 returns Ok(Vec::new()). Concrete impact on a shipped tweak: src-tauri/tweaks/windows_update.yaml:271-274 option "Disabled (Complete)" declares `task_name_pattern: "Schedule Scan|USO|MusNotification|Reboot|Refresh"` with `ignore_not_found: true` -> apply_action_to_pattern returns Ok((0,0,[])) -> the UI reports the option applied while zero UpdateOrchestrator tasks are disabled. Any pattern tweak WITHOUT ignore_not_found instead hard-errors ("No tasks found matching pattern") and rolls back the whole atomic set.

**Evidence.**

```
src-tauri/src/services/scheduler_service.rs:203-207:
'''
    let output = Command::new("schtasks")
        .args(["/Query", "/TN", &format!("{}\\", path), "/FO", "LIST", "/V"])
'''
src-tauri/src/services/scheduler_service.rs:210-215:
'''
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("does not exist") || stderr.contains("cannot find") {
            log::debug!("Task folder not found: {}", path);
            return Ok(Vec::new());
        }
'''
src-tauri/src/services/scheduler_service.rs:299-306:
'''
    if tasks.is_empty() {
        if ignore_not_found {
            log::warn!(
                "No tasks found matching pattern '{}' in '{}' (ignore_not_found=true)",
...
            return Ok((0, 0, Vec::new()));
'''
```

**Fix.** Run `schtasks /Query /FO LIST /V` with no /TN, parse the `Folder:` / `TaskName:` full paths from the output, and filter to tasks whose parent path equals task_path. Separately, do not conflate "query failed" with "folder is empty": only ignore_not_found should suppress an empty result, and a hard schtasks failure must propagate.

**Verifier correction.** The premise is false: `schtasks /Query /TN "<folder>\" /FO LIST /V` is a VALID and working folder-enumeration form. The finder's 'verified live' evidence was a shell-escaping artifact, not schtasks behavior.

<details><summary>Verifier reasoning</summary>

I reproduced the finder's exact failure first (`ERROR: The system cannot find the file specified.`) and then proved it was my own shell mangling backslashes - the same command on a KNOWN-GOOD full task name (\Microsoft\Windows\Defrag\ScheduledDefrag) also 'failed' under that harness. Re-run through a clean PowerShell invocation with no path translation, the exact form the code builds at scheduler_service.rs:203-207 succeeds: `schtasks /Query /TN '\Microsoft\Windows\Defrag\' /FO LIST /V` -> EXIT=0, output `TaskName: \Microsoft\Windows\Defrag\ScheduledDefrag / Status: Ready`. Verified against the specific tweak the finding cites: `/TN '\Microsoft\Windows\UpdateOrchestrator\'` returns Schedule Scan, Schedule Scan Static Task, Schedule Maintenance Work, Report policies - so the regex at windows_update.yaml:273 ('Schedule Scan|USO|MusNotification|Reboot|Refresh') DOES match live tasks. Rust's Command::new("schtasks").args([...]) bypasses cmd.exe entirely and std's make_command_line correctly doubles the trailing backslash before the closing quote, so schtasks receives the literal folder path. find_tasks_by_pattern is not a no-op. (Unclaimed side observation, not promoted: the /V parse at scheduler_service.rs:229-250 emits duplicate TaskInfo entries for multi-trigger tasks, e.g. 'Report policies' appeared twice.)

</details>

---

## YAML tweak corpus

### [HIGH · CONFIRMED] Seven tweaks mark every registry change skip_validation in every option, making their status permanently undetectable

`src-tauri/tweaks/services.yaml:176` — lens: code-defect

**What is wrong.** In 7 tweaks, every registry change in every option carries skip_validation: true, so detection filters out all changes and no option can ever be reported as current.

**Failing scenario.** User applies disable_retail_demo -> "Disabled" on a machine that has RetailDemo. The registry write DOES happen (Start=4). On next status query, option_matches_current_state() filters out the only change (skip_validation), hits the empty-set guard and returns not_matched() for BOTH options, so current_option_index = null and the UI shows "System Default" forever. The tweak can never display as applied, and the user re-applies it repeatedly believing it failed. Affected: disable_homegroup_provider (services.yaml:176), disable_retail_demo (services.yaml:234), disable_wallet_service (services.yaml:275), disable_xbox_services (gaming.yaml:598), disable_geolocation_service (privacy.yaml:1114), disable_dmwappushservice (privacy.yaml:1156), disable_cdpusersvc (privacy.yaml:1195).

**Evidence.**

```
src-tauri/tweaks/services.yaml:257-266 (disable_retail_demo, BOTH options):
      - label: "Disabled"
        registry_missing_is_match: true
        registry_changes:
          - hive: HKLM
            key: "System\\CurrentControlSet\\Services\\RetailDemo"
            value_name: "Start"
            value_type: "REG_DWORD"
            value: 4
            skip_validation: true
      - label: "Enabled (Default)"
        ... value: 3
            skip_validation: true

src-tauri/src/services/backup/detection.rs:123-131:
    // If option has no validatable changes, it can't match
    if validatable_registry.is_empty()
        && validatable_services.is_empty()
        && validatable_scheduler.is_empty()
        && validatable_hosts.is_empty()
        && validatable_firewall.is_empty()
    {
        return Ok(MatchResult::not_matched());
    }
```

**Fix.** Drop skip_validation from these changes and rely on registry_missing_is_match: true (already present on the "Disabled" option) to handle editions where the service key is absent. Add registry_missing_is_match to the "Enabled (Default)" options too. Optionally add a build.rs validator that rejects any option whose changes are all skip_validation.

**Verifier correction.** Understated: it is 8 tweaks, not 7. A mechanical scan of all option/change categories finds every option's every change carrying skip_validation in: gaming.yaml disable_xbox_services, network.yaml ipv6_transition_mode (network.yaml:65, missed by the finder, 4 options), privacy.yaml disable_geolocation_service / disable_dmwappushservice / disable_cdpusersvc, services.yaml disable_homegroup_provider / disable_retail_demo / disable_wallet_service.

<details><summary>Verifier reasoning</summary>

Verified both sides verbatim. services.yaml:257-273 shows disable_retail_demo's "Disabled" (value 4) and "Enabled (Default)" (value 3) options each carrying the sole registry change with skip_validation: true; same pattern confirmed at services.yaml:198-229 (homegroup), gaming.yaml:621-650 (xbox), privacy.yaml:1136-1152 / 1176-1191 / 1215+ . detection.rs:98-102 filters `!c.skip_validation`, and detection.rs:123-131 returns MatchResult::not_matched() when all validatable sets are empty, so detect_tweak_state (detection.rs:65-88) falls through every option to current_option_index: None. I looked for an escape hatch and found none: TweakCard.svelte:96 `const currentOptionIndex = $derived(tweak.status.current_option_index)` and :106-110 map null to the -1 "Default" segment; snapshot_option_index is carried but is not used as a status fallback. tweaksActions.svelte.ts:87 only optimistically sets the index in-session, so the wrong status returns on the next status query/restart. skip_validation is also redundant here: the "Disabled" options already carry registry_missing_is_match: true, which is the spec's mechanism for absent service keys, so the flag buys nothing and only breaks detection.

</details>

---

### [HIGH · CONFIRMED] windows_update_mode disables UpdateOrchestrator tasks in one option and no other option re-enables them

`src-tauri/tweaks/windows_update.yaml:270` — lens: code-defect

**What is wrong.** Only the "Disabled (Complete)" option carries scheduler_changes; the three restore options have none, so switching back leaves Windows Update's scan/reboot tasks disabled while the UI reports the default state.

**Failing scenario.** User selects "Disabled (Complete)" -> all UpdateOrchestrator tasks matching /Schedule Scan|USO|MusNotification|Reboot|Refresh/ are set to Disabled. User changes their mind and selects "Automatic Install (Default)" (windows_update.yaml:183). That option writes only registry + service changes; it has no scheduler_changes, so the tasks stay Disabled. Detection then matches "Automatic Install (Default)" (its validatable set is satisfied), so the UI claims Windows Update is on Automatic while Schedule Scan never runs and the machine silently stops receiving security updates. Snapshot does not help: it is only consulted on explicit Revert, and on option-switch its metadata is updated rather than restored.

**Evidence.**

```
src-tauri/tweaks/windows_update.yaml:270-275 (only option with scheduler_changes):
      - label: "Disabled (Complete)"
        scheduler_changes:
          - task_path: "\\Microsoft\\Windows\\UpdateOrchestrator"
            task_name_pattern: "Schedule Scan|USO|MusNotification|Reboot|Refresh"
            action: disable
            ignore_not_found: true

src-tauri/tweaks/windows_update.yaml:183 / :212 / :241 - "Automatic Install (Default)", "Download Only", "Notify Only" declare only registry_changes and service_changes; mechanical union diff shows each is MISSING the task target T|\Microsoft\Windows\UpdateOrchestrator|Schedule Scan|USO|MusNotification|Reboot|Refresh.
```

**Fix.** Add the mirrored scheduler_changes block with action: enable and the same task_path/task_name_pattern/ignore_not_found to all three restore options, so every option declares the complete target state as the spec requires.

<details><summary>Verifier reasoning</summary>

Read windows_update.yaml:183-297 in full. "Disabled (Complete)" (line 269) is the only option with a scheduler_changes block (lines 270-274: task_path \\Microsoft\\Windows\\UpdateOrchestrator, task_name_pattern "Schedule Scan|USO|MusNotification|Reboot|Refresh", action: disable). "Automatic Install (Default)" (:183), "Download Only" (:212) and "Notify Only" (:241) declare only registry_changes + service_changes, so nothing re-enables the tasks. I tried to refute via an implicit restore-before-switch: apply.rs:88 `let is_switching_options = backup_service::snapshot_exists(&tweak_id)?;` and apply.rs:170-172 show the switch path only calls update_snapshot_metadata — restore_from_snapshot is reached solely on failure (apply.rs:158-164) or explicit revert (apply.rs:265). Detection then does match the restore option (its registry values are all written, UsoSvc is validatable and set), so the UI reports "Automatic Install (Default)" while Schedule Scan stays disabled. This also directly contradicts the spec line "Each option declares the COMPLETE target state for that state."

</details>

---

### [HIGH · CONFIRMED] Two gaming tweaks each overwrite the whole DirectXUserGlobalSettings composite string, silently erasing each other

`src-tauri/tweaks/gaming.yaml:790` — lens: code-defect

**What is wrong.** variable_refresh_rate and gpu_preference_default both write the single REG_SZ value DirectXUserGlobalSettings with a full replacement string containing only their own key, so applying one destroys the other's setting.

**Failing scenario.** On a gaming laptop: user applies gpu_preference_default -> "High Performance GPU", writing DirectXUserGlobalSettings = "GpuPreference=2;". User then applies variable_refresh_rate -> "VRR Windowed Enabled", writing DirectXUserGlobalSettings = "VRROptimizeEnable=1;". The GpuPreference=2 setting is now gone, the discrete GPU is no longer the default, and gpu_preference_default's detection matches neither "GpuPreference=2;" nor "GpuPreference=0;", so it reports "System Default" even though the user explicitly set it. Windows itself stores these as one combined string (e.g. "VRROptimizeEnable=0;GpuPreference=0;"), so any pre-existing user value is also destroyed by whichever tweak is applied first.

**Evidence.**

```
src-tauri/tweaks/gaming.yaml:786-793 (variable_refresh_rate):
          - hive: HKCU
            key: "Software\\Microsoft\\DirectX\\UserGpuPreferences"
            value_name: "DirectXUserGlobalSettings"
            value_type: "REG_SZ"
            value: "VRROptimizeEnable=1;"

src-tauri/tweaks/gaming.yaml:823-830 (gpu_preference_default):
          - hive: HKCU
            key: "Software\\Microsoft\\DirectX\\UserGpuPreferences"
            value_name: "DirectXUserGlobalSettings"
            value_type: "REG_SZ"
            value: "GpuPreference=2;"
```

**Fix.** Merge the two tweaks into one multi-option tweak whose options write the full combined string (e.g. "VRROptimizeEnable=1;GpuPreference=2;") for each cell of the 2x2 matrix, or add a registry action that merges into a delimited string rather than replacing it.

**Verifier correction.** Broader than stated: the wrong-status half does not even require applying both tweaks. Windows itself writes DirectXUserGlobalSettings as a combined string (e.g. "VRROptimizeEnable=0;GpuPreference=0;") once the user opens Graphics settings, and registry_values_match does exact normalized-string equality, so on such an untouched machine neither tweak matches any option and both already show "System Default".

<details><summary>Verifier reasoning</summary>

Both anchors verified verbatim: gaming.yaml:786-793 writes DirectXUserGlobalSettings = "VRROptimizeEnable=1;" and gaming.yaml:823-830 writes the same HKCU\\Software\\Microsoft\\DirectX\\UserGpuPreferences value name = "GpuPreference=2;", both REG_SZ action set (full replacement). I tried to refute by looking for merge semantics in the write path: registry_value.rs:96-106 dispatches RegistryValue::String straight to registry_service::set_string — plain overwrite, no merge. Detection is exact equality (registry_value.rs:66-80, `normalized_current == normalized_expected`), so after one tweak writes its string the other matches neither of its options. Snapshots make it worse rather than better: applying the second tweak captures the first tweak's clobbered string as its "original", so reverting in the wrong order restores a value that was never the user's.

</details>

---

### [HIGH · CONFIRMED] legacy_network_protocols silently re-enables SMBv1 in two options while its name, description and info mention only LLMNR/WPAD/NetBIOS

`src-tauri/tweaks/network.yaml:243` — lens: code-defect

**What is wrong.** The "NetBIOS Only Enabled" and "All Enabled (Default)" options set LanmanServer\Parameters\SMB1=1 and mrxsmb10\Start=3, turning the SMBv1 protocol back on, which is disclosed nowhere in the tweak's user-facing text and is rated only risk_level: medium.

**Failing scenario.** User has an old network printer, reads "NetBIOS Only Enabled: Legacy device compatibility, other protocols disabled" and selects it. In addition to the advertised NetBIOS change, the option writes SMB1=1 (network.yaml:243) and mrxsmb10 Start=3 (network.yaml:247), re-enabling the SMBv1 server and client driver -- the EternalBlue/WannaCry protocol that Windows 10 1709+ and Windows 11 ship removed/disabled. The user is given no indication that SMBv1 was touched. The same happens for anyone selecting "All Enabled (Default)" (network.yaml:313/317) to "restore defaults", which is not the stock state on any supported Windows build.

**Evidence.**

```
src-tauri/tweaks/network.yaml:151-152 (user-facing scope):
    description: "Configure legacy protocols (LLMNR, WPAD, NetBIOS) for security"
    risk_level: medium

src-tauri/tweaks/network.yaml:159-162 (info block, no SMB1 mention):
      - **NetBIOS Only Enabled**: Legacy device compatibility, other protocols disabled

src-tauri/tweaks/network.yaml:241-249 ("NetBIOS Only Enabled"):
          - hive: HKLM
            key: "System\\CurrentControlSet\\Services\\LanmanServer\\Parameters"
            value_name: "SMB1"
            value_type: "REG_DWORD"
            value: 1
          - hive: HKLM
            key: "System\\CurrentControlSet\\Services\\mrxsmb10"
            value_name: "Start"
            value_type: "REG_DWORD"
            value: 3
```

**Fix.** Remove the SMB1/mrxsmb10 changes from this tweak entirely and leave SMBv1 to the dedicated disable_smbv1 tweak; if they must stay, rename/redescribe the tweak to disclose SMBv1, raise risk_level to high, and never set SMB1=1 in an option labelled "(Default)".

**Verifier correction.** Add the cross-tweak effect: security.yaml:194 defines a separate disable_smbv1 tweak whose "SMBv1 Disabled" option writes the exact same LanmanServer\\Parameters\\SMB1 value (security.yaml:222). Selecting legacy_network_protocols -> "NetBIOS Only Enabled" or "All Enabled (Default)" silently undoes disable_smbv1 and flips its displayed status to "SMBv1 Enabled", with no conflict warning anywhere.

<details><summary>Verifier reasoning</summary>

Verified verbatim. network.yaml:151 `description: "Configure legacy protocols (LLMNR, WPAD, NetBIOS) for security"`, :152 `risk_level: medium`, and the info block at :159-162 list only LLMNR/WPAD/NetBIOS — SMB is mentioned nowhere in any user-facing field. Yet network.yaml:241-249 ("NetBIOS Only Enabled") writes LanmanServer\\Parameters\\SMB1 = 1 and mrxsmb10\\Start = 3, and network.yaml:311-319 ("All Enabled (Default)") does the same. I tried to refute on the grounds that SMBv1 is removed on modern builds so the writes are inert: they are not — mrxsmb10 Start=3 arms the SMBv1 client driver and SMB1=1 the server wherever the component is still present (upgraded machines, Server SKUs, anyone who installed the optional feature), and even where inert the writes still corrupt disable_smbv1's status. The undisclosed-scope half stands unconditionally.

</details>

---

### [MEDIUM · CONFIRMED] 98 "(Default)"-labelled options write explicit Group Policy values that stock Windows ships absent, leaving the machine policy-managed

`src-tauri/tweaks/windows_update.yaml:187` — lens: code-defect — *severity adjusted by verifier: high → medium*

**What is wrong.** 33 registry writes inside options labelled "(Default)" target Software\Policies\... or CurrentVersion\Policies\... keys that unconfigured Windows does not have, so choosing the "default" option configures a policy instead of removing one.

**Failing scenario.** User applies windows_update_mode -> "Disabled (Complete)", then returns to "Automatic Install (Default)". That option writes AUOptions=4, NoAutoUpdate=0 and SetDisableUXWUAccess=0 under HKLM\Software\Policies\Microsoft\Windows\WindowsUpdate. On a stock Home/Pro machine these values do not exist; creating them puts Windows Update permanently under Group Policy control, so the Settings page shows "Some settings are managed by your organization" and the user can no longer change update behaviour from the UI. The corpus proves it knows the correct idiom: for the very same EnableMulticast value, legacy_network_protocols "All Enabled (Default)" uses action: delete_value (network.yaml:287) while disable_llmnr "LLMNR Enabled (Default)" writes value: 1 (security.yaml:964). Across "(Default)"-labelled options there are 128 set writes vs only 22 deletes.

**Evidence.**

```
src-tauri/tweaks/windows_update.yaml:183-199 ("Automatic Install (Default)"):
      - label: "Automatic Install (Default)"
        registry_changes:
          - hive: HKLM
            key: "Software\\Policies\\Microsoft\\Windows\\WindowsUpdate\\AU"
            value_name: "AUOptions"
            value_type: "REG_DWORD"
            value: 4

Contradictory idiom for one identical value:
src-tauri/tweaks/network.yaml:287 "All Enabled (Default)" -> EnableMulticast action: delete_value
src-tauri/tweaks/security.yaml:964 "LLMNR Enabled (Default)" -> EnableMulticast value: 1

Other confirmed instances: privacy.yaml:58 (TurnOffWindowsAI=0, DisableSavingSnapshots=0), windows_update.yaml:397 (DeferFeatureUpdates=0), windows_update.yaml:459 (DeferQualityUpdates=0), windows_update.yaml:733 (TargetReleaseVersionInfo="").
```

**Fix.** For policy-hive values, make every "(Default)"/restore option use action: delete_value (or delete_key for a policy subkey the tweak created) instead of writing an explicit value, matching the pattern already used at network.yaml:287.

**Verifier correction.** Two overstatements. (1) The premise "unconfigured Windows does not have" these values is false for a chunk of the 33: security.yaml uac_level "Standard (Default)" writes ConsentPromptBehaviorAdmin / ConsentPromptBehaviorUser / PromptOnSecureDesktop / FilterAdministratorToken under CurrentVersion\\Policies\\System, which a stock Windows install ships already present — writing them there is correct, not a defect. The real subset is the genuinely-absent policy values, chiefly the windows_update.yaml ones (AUOptions, NoAutoUpdate, SetDisableUXWUAccess, Defer*, TargetReleaseVersion*) plus privacy.yaml:70-82 and the ui.yaml Policies entries. (2) A workaround does exist: restore.rs:169-171 `if !op.existed { // Value didn't exist - delete it` shows Revert correctly deletes values that were absent at capture time, so a user who uses Revert instead of picking the "(Default)" option is not left policy-managed. Hence medium, not high.

<details><summary>Verifier reasoning</summary>

I re-derived the counts mechanically rather than trusting them, and they are exact: 98 "(Default)"-labelled options, 128 set vs 22 delete registry changes inside them, 33 set writes targeting a Policies key. The headline's "98 ... options write explicit Group Policy values" is a misstatement (98 is the total count of default-labelled options); the claim body's 33 is right. Both contradictory-idiom anchors verified verbatim: network.yaml:285-288 "All Enabled (Default)" uses `action: delete_value` for EnableMulticast while security.yaml:962-969 "LLMNR Enabled (Default)" writes `value: 1` for the identical value. privacy.yaml:58-82 is the sharpest internal contradiction — the same option uses delete_value for DisableAIDataAnalysis but writes TurnOffWindowsAI=0 and DisableSavingSnapshots=0 two entries later. I tried to refute on spec grounds ("each option declares the COMPLETE target state" arguably licenses explicit writes) and that argument does blunt the spec-conformance angle, but it does not rescue the windows_update case: the option labelled "(Default)" demonstrably leaves HKLM\\Software\\Policies\\Microsoft\\Windows\\WindowsUpdate populated on a machine that had no such key, which is not the default state the label promises.

</details>

---

### [MEDIUM · CONFIRMED] ipv6_transition_mode consists solely of pre_commands, so it is never snapshotted, never detectable and never revertible

`src-tauri/tweaks/network.yaml:65` — lens: code-defect — *severity adjusted by verifier: high → medium*

**What is wrong.** All four options of ipv6_transition_mode contain only pre_commands and zero registry/service/scheduler changes, so no state is captured, no option can ever match, and Revert cannot undo the netsh changes.

**Failing scenario.** User applies "All Disabled (Lowest Latency)", which runs `netsh interface teredo set state disabled`. TweakSnapshot has only registry_snapshots/service_snapshots/scheduler_snapshots fields (models/tweak_snapshot.rs:91-95) -- commands are not captured -- so nothing records the prior Teredo state. Detection then filters an empty change set and returns not_matched() for all four options, so the tweak permanently shows "System Default". Xbox Live party chat is now broken and Revert restores nothing, because the snapshot contains no data about Teredo. The user has no path back except knowing to select "All Enabled (Default)" by hand.

**Evidence.**

```
src-tauri/tweaks/network.yaml:93-108 (all four options):
      - label: "All Disabled (Lowest Latency)"
        pre_commands:
          - "netsh interface teredo set state disabled"
          - "netsh interface isatap set state disabled"
      - label: "Teredo Only (Xbox)"
        pre_commands:
          - "netsh interface teredo set state default"
          - "netsh interface isatap set state disabled"

src-tauri/src/models/tweak_snapshot.rs:91-95:
    /// Registry values captured before changes
    pub registry_snapshots: Vec<RegistrySnapshot>,
    /// Service states captured before changes
    pub service_snapshots: Vec<ServiceSnapshot>,
    /// Scheduled task states captured before changes
```

**Fix.** Back each option with the underlying registry state (Tcpip6\Parameters\DisabledComponents, or the Teredo/ISATAP policy values under Software\Policies\Microsoft\Windows\TCPIP\v6Transition) so the change is snapshottable and detectable, keeping netsh only as a supplementary post_command.

**Verifier correction.** The claim holds, but two details need adjusting: (1) a snapshot IS created and saved on first apply (src-tauri/src/commands/tweaks/apply.rs:95-105) - it is simply EMPTY, so Revert reports success while restoring nothing, and startup stale-detection deliberately keeps it forever (detection.rs:589-593 returns Ok(false) when all snapshot vectors are empty), so the tweak is stuck in a permanent "has_snapshot + System Default" state; (2) the netsh state is recoverable in-app by selecting "All Enabled (Default)", which is why this is medium rather than high - the wrong status and the no-op Revert are real, but the user's original state is restorable for any machine that was at the netsh default.

<details><summary>Verifier reasoning</summary>

Verified src-tauri/tweaks/network.yaml:92-108: all four options of ipv6_transition_mode contain only pre_commands, zero registry/service/scheduler/hosts/firewall changes. Verified detection.rs:124-131: `if validatable_registry.is_empty() && ... { return Ok(MatchResult::not_matched()); }` - every option is unmatchable, so detect_tweak_state falls through to current_option_index: None (detection.rs:161-168). Verified TweakCard.svelte:158-161 renders `{ value: -1, label: "System Default", disabled: true }` whenever currentOptionIndex === null, so this 4-option dropdown can never display the applied option. Verified TweakSnapshot has no command fields (models/tweak_snapshot.rs:91-104), so restore_from_snapshot has nothing to undo. Attempted refutations that failed: build.rs only rejects options with NO changes at all (build.rs:1044-1063 counts pre_commands as changes) so nothing blocks this at compile time; snapshot_option_index is loaded (detection.rs:59-63) but the frontend never uses it for the displayed value (grep for snapshotOptionIndex in src/ returns nothing; effectiveOptionIndex at TweakCard.svelte:147-152 uses currentOptionIndex only). This is the only command-bearing tweak in the corpus - the other pre_commands hit is commented out at ui.yaml:240-247.

</details>

---

### [MEDIUM · CONFIRMED] mouse_input_mode option 1 and option 3 declare byte-identical state, making option 3 permanently unreachable

`src-tauri/tweaks/gaming.yaml:477` — lens: code-defect

**What is wrong.** "No Acceleration" (gaming.yaml:409) and "Windows Default (EPP Off)" (gaming.yaml:477) set the same six registry values to the same six values, so first-match-wins detection can never report option 3.

**Failing scenario.** User selects "Windows Default (EPP Off)". All six writes succeed. On the next status query, detection walks options in order and "No Acceleration" (index 1) matches first because its validatable set is identical, so the UI switches the selection to "No Acceleration". The user's chosen option is silently replaced on screen and can never be shown as current; re-selecting it changes nothing. Confirmed by signature comparison: MouseSpeed="0", MouseThreshold1="0", MouseThreshold2="0", MouseSensitivity="10", and identical SmoothMouseXCurve/SmoothMouseYCurve binary strings in both options.

**Evidence.**

```
src-tauri/tweaks/gaming.yaml:409-441 ("No Acceleration") and gaming.yaml:477-509 ("Windows Default (EPP Off)") both end with:
            value_name: "SmoothMouseXCurve"
            value_type: "REG_BINARY"
            value: "00,00,00,00,00,00,00,00,15,6E,00,00,00,00,00,00,00,40,01,00,00,00,00,00,29,DC,03,00,00,00,00,00,00,00,28,00,00,00,00,00"
          - hive: HKCU
            key: "Control Panel\\Mouse"
            value_name: "SmoothMouseYCurve"
            value_type: "REG_BINARY"
            value: "00,00,00,00,00,00,00,00,FD,11,01,00,00,00,00,00,00,24,04,00,00,00,00,00,00,FC,12,00,00,00,00,00,00,00,C0,BB,01,00,00,00,00"
```

**Fix.** Delete the redundant "Windows Default (EPP Off)" option, or differentiate it (e.g. leave MouseSensitivity untouched / use delete_value on the curves so it truly restores stock rather than pinning MarkC-adjacent values).

<details><summary>Verifier reasoning</summary>

Read gaming.yaml:409-441 ("No Acceleration") and gaming.yaml:477-509 ("Windows Default (EPP Off)") in full: both write exactly MouseSpeed="0", MouseThreshold1="0", MouseThreshold2="0", MouseSensitivity="10", and the identical SmoothMouseXCurve/SmoothMouseYCurve REG_BINARY strings ending "...00,00,28,00,00,00,00,00" and "...00,C0,BB,01,00,00,00,00". Same hive (HKCU), same key, no skip_validation, no windows_versions gate, no missing_is_match difference - the validatable sets are byte-identical. detection.rs:150-160 iterates options in order and returns on the FIRST match, so index 1 always wins and index 3 is unreachable. Refutation attempts that failed: build.rs only checks duplicate option LABELS (build.rs:1116-1127), not duplicate content, so nothing rejects this; profile content-hashing (services/profile/mod.rs:25-33) hashes the serialized option including the label so profile import is not additionally broken. Frontend shows selectOptions by currentOptionIndex (TweakCard.svelte:147-152), confirming the user's pick silently snaps back to "No Acceleration". Medium is correct: cosmetic-to-confusing, no state loss.

</details>

---

### [MEDIUM · CONFIRMED] Seven registry values are owned by two tweaks each, and four of those pairs disagree on the stock default

`src-tauri/tweaks/privacy.yaml:151` — lens: code-defect

**What is wrong.** Multiple tweaks write the same registry value with contradictory "restore" values, so applying one tweak silently changes another tweak's reported state and at most one of the two restore values can be correct.

**Failing scenario.** dmwappushservice\Start is owned by both disable_telemetry and disable_dmwappushservice, and their restore options disagree: disable_telemetry -> "Enabled" writes Start=2 (privacy.yaml:151, automatic) while disable_dmwappushservice -> "WAP Push Enabled (Default)" writes Start=3 (privacy.yaml:1189, manual). Windows 10 1803+/11 ship this service as Manual (3), so restoring via disable_telemetry leaves it auto-starting -- a state the machine was never in. Concretely: user applies disable_dmwappushservice -> Enabled (Start=3), then applies disable_telemetry -> Enabled, and Start becomes 2 with no warning. Same class: EnableMulticast (delete_value at network.yaml:287 vs value 1 at security.yaml:964), NodeType (delete_value at network.yaml:287 vs value 1 at security.yaml:1002 -- NodeType=1 pins B-node broadcast-only, not stock), SMB1 (value 1 at network.yaml:243/313 vs value 0 at security.yaml:218). Also duplicated without conflict: SoftLandingEnabled (performance.yaml:128 / privacy.yaml:439) and EnableActivityFeed (privacy.yaml:210 / privacy.yaml:603).

**Evidence.**

```
src-tauri/tweaks/privacy.yaml:151 (disable_telemetry "Enabled"):
            key: "System\\CurrentControlSet\\Services\\dmwappushservice"
            value_name: "Start"
            value_type: "REG_DWORD"
            value: 2

src-tauri/tweaks/privacy.yaml:1189 (disable_dmwappushservice "WAP Push Enabled (Default)"):
            key: "System\\CurrentControlSet\\Services\\dmwappushservice"
            value_name: "Start"
            value_type: "REG_DWORD"
            value: 3

src-tauri/tweaks/security.yaml:1002 (disable_netbios "NetBIOS Default (B-Node)"):
            value_name: "NodeType"
            value_type: "REG_DWORD"
            value: 1
```

**Fix.** Give each registry value exactly one owning tweak (merge disable_llmnr/disable_netbios/disable_smbv1 into legacy_network_protocols, drop dmwappushservice from disable_telemetry, dedupe disable_timeline vs disable_activity_history and disable_windows_tips vs disable_suggested_content), and add a build.rs check that fails the build when two tweaks declare the same hive+key+value_name.

**Verifier correction.** Anchors drift ~4 lines (EnableMulticast delete_value is network.yaml:291 not :287; NodeType delete_value is network.yaml:309; disable_llmnr enabled-value is security.yaml:968; disable_netbios B-node is security.yaml:1006; SMB1 enabled is security.yaml:229). More importantly, the dmwappushservice example is weaker than presented: privacy.yaml:150-156 carries `windows_versions: [10]` AND `skip_validation: true`, and BOTH options of disable_dmwappushservice are also skip_validation (privacy.yaml:1181-1194), so that pair corrupts the WRITTEN value on Win10 only and neither tweak is status-detectable anyway. The load-bearing, fully-validatable conflicts are EnableMulticast (legacy_network_protocols delete_value vs disable_llmnr value 1), NodeType (delete_value vs value 1) and SMB1 (network value 1 vs security value 0) - use those as the primary anchors.

<details><summary>Verifier reasoning</summary>

Confirmed both owners are live tweaks: legacy_network_protocols (network.yaml:149) and disable_llmnr/disable_netbios/disable_smbv1 (security.yaml:938/972/194). Grep for EnableMulticast|NodeType|SMB1 across src-tauri/tweaks/*.yaml shows each value written by two distinct tweaks with disagreeing restore values. Searched for any mitigation and found none: grep 'conflict' across src-tauri/src, src-tauri/build.rs and docs/*.md returns zero hits - no build-time overlap check, no runtime conflict warning. Concrete state-corruption walk-through against the real code: apply legacy_network_protocols->"All Disabled" (snapshot records SMB1 absent), then apply disable_smbv1->"SMBv1 Disabled" (its capture now records SMB1=1 as the "original", a value the machine never had), then revert legacy_network_protocols, then revert disable_smbv1 -> the machine ends with SMB1=1 on LanmanServer, i.e. SMBv1 server re-enabled by a pair of reverts. Status corruption is equally reachable: applying disable_llmnr->"LLMNR Disabled" writes EnableMulticast=0, which is exactly what legacy_network_protocols' "All Disabled" option expects, flipping that tweak's reported state without the user touching it. Medium is right - requires two overlapping tweaks, but the outcome is a security regression plus wrong status.

</details>

---

### [LOW · PLAUSIBLE] disable_wdigest is risk_level low but offers an option that re-enables cleartext credential storage in LSASS

`src-tauri/tweaks/security.yaml:1013` — lens: spec-conformance — *severity adjusted by verifier: medium → low*

**What is wrong.** RiskLevel::Low is documented as "Safe to apply/revert without issues", yet this tweak's second option sets UseLogonCredential=1, which the tweak's own info block describes as the Mimikatz cleartext-password-extraction condition.

**Failing scenario.** User browsing low-risk tweaks selects "WDigest Enabled (Insecure)" without a risk warning. Windows then caches plaintext logon passwords in LSASS memory, so any subsequent local-admin compromise yields the user's cleartext credentials. The UI presents this as a low-risk, freely reversible toggle, which contradicts the RiskLevel::Low contract in the model.

**Evidence.**

```
src-tauri/src/models/tweak.rs:17-18 (spec anchor):
    /// Safe to apply/revert without issues
    Low,

src-tauri/tweaks/security.yaml:1013 (code anchor):
    risk_level: low

src-tauri/tweaks/security.yaml:1019 (the tweak's own description of the hazard):
      Disables WDigest authentication which stores plaintext passwords in memory. This is exploited by tools like Mimikatz to extract credentials.

src-tauri/tweaks/security.yaml:1035-1040:
      - label: "WDigest Enabled (Insecure)"
        registry_changes:
          - hive: HKLM
            key: "System\\CurrentControlSet\\Control\\SecurityProviders\\WDigest"
            value_name: "UseLogonCredential"
            value_type: "REG_DWORD"
            value: 1
```

**Fix.** Raise risk_level to high, and make the secure option use action: delete_value (stock Windows ships UseLogonCredential absent, which is already the secure state) so the tweak does not permanently materialise the value. Same treatment applies to disable_remote_registry (security.yaml:36 restores RemoteRegistry Start=2 where Windows 10/11 client ships it Disabled=4).

**Verifier correction.** The spec-conformance framing does not survive: the repo's own risk taxonomy is stability-oriented, not security-oriented - docs/TWEAK_AUTHORING.md:1519-1532 defines the ladder as "Low: Can be safely toggled anytime / Medium: Might need restart, minor side effects / High: Could break functionality / Critical: Could prevent boot", and toggling UseLogonCredential genuinely is safe to apply and revert with no functional breakage, so `risk_level: low` does not contradict the quoted line. The defensible version of this finding is behavioural, not doc-based: (a) risk_level low bypasses the confirmation dialog (src/lib/components/tweaks/TweakCard.svelte:58 `isHighRisk = risk_level === "high" || "critical"`, dialog at :430-437), so this option applies with no prompt while disable_smartscreen - a pure security-exposure tweak with no functional breakage - is rated high (security.yaml:273) and does prompt; and (b) the OFF position of a 2-option toggle writes UseLogonCredential=1 (security.yaml:1035-1040) where stock Win10/11 ships the value absent, so toggling "off" leaves the machine less secure than stock rather than at stock.

<details><summary>Verifier reasoning</summary>

Verified the code anchors exist verbatim: models/tweak.rs:16-18 (`/// Safe to apply/revert without issues` / `Low,`), security.yaml:1013 (`risk_level: low`), security.yaml:1018-1019 (Mimikatz description) and security.yaml:1035-1040 (`UseLogonCredential` `value: 1`). Refutation succeeded on the spec side: the authoring doc's own definitions scope risk to breakage/reboot, not to security posture, so no doc line is contradicted - the finding reinterprets "risk". Refutation only partly succeeded on the harm side: the confirm-dialog gate at TweakCard.svelte:58 is real and does mean this applies unprompted, and the internal inconsistency with disable_smartscreen (high) is real. Downgraded to low because the option is explicitly labelled "(Insecure)", the info block spells out the Mimikatz hazard, the user must deliberately select it, and Revert correctly restores the original absent value from the snapshot.

</details>

---


## Documentation (found by reading the two docs against each other)

These are not code defects — they are contradictions *within* and *between* the two spec documents.
An author following them will write a tweak that does not do what they expect.

### `value: null` is documented three mutually exclusive ways

- `TWEAK_AUTHORING.md` **Example 4** writes `value_type: "REG_SZ"` with `value: null`.
- The **Common Validation Errors** section says that exact combination is a *build error*:
  `REG_SZ requires string value, got null`.
- The **Appendix: Value Type Reference** says `Null | (delete) | value: null | Deletes the value`.

At most one can be true. If the validator is strict, the guide's own Example 4 does not compile.

### "System Default" — selectable or not?

- `TWEAK_SYSTEM.md` design table: *"Non-selectable indicator when no option matches."*
- `TWEAK_AUTHORING.md`: *"This ensures users can always see and **return to** the original system state."*

One of these is wrong about whether the user can click it. This matters: it is the only documented
path back to an unrecognised original state.

### `hosts_changes` and `firewall_changes` are missing from the Option Structure block

The canonical field listing under **Options Array → Option Structure** stops at `scheduler_changes` +
the three `*_missing_is_match` flags. Both newer change types are documented further down and appear in
the execution order, but an author reading the structure reference will not know they exist.

Related: there is no `hosts_missing_is_match` / `firewall_missing_is_match`, though both types
participate in detection.

### The State Detection algorithm only covers registry and services

`TWEAK_AUTHORING.md`'s step-by-step detection algorithm filters and compares `registry_changes` and
`service_changes` only. `TWEAK_SYSTEM.md` says all five change types are checked in parallel.
The authoring guide's algorithm is a strict subset of what the system does.

### Common Mistake #2 shows an identical wrong and correct example

"Wrong Option Count for Toggle" presents the same three-option Low/Medium/High block under both the
incorrect and correct headings. The example teaches nothing.

### Root cause

`TWEAK_SYSTEM.md` is dated 2026-05-29; `TWEAK_AUTHORING.md` says *"Last updated: December 2025"*.
The authoring guide predates the hosts/firewall work, which explains most of the omissions above.
