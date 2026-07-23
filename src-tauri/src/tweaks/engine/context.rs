//! Execution-context routing (spec §9, ADR-0005 as amended; invariant 24): effective-level
//! computation (max, escalate-only), the HKCU always-in-process-as-user exception, grouping
//! consecutive System/TI steps into one child, and the over-the-shoulder SID guard. Pure logic —
//! the one seam that touches the OS (the SID lookups) sits behind an injectable [`SidProbe`], so
//! everything else here runs with zero OS contact by default.

use crate::tweaks::kinds::ExecCx;
use crate::tweaks::model::{Effect, EffectDef, EffectId, Hive, Level, Setting, Tweak};

/// Escalate-only ranking for `effective_level` (spec §9): `User < Admin < System < Ti`.
fn rank(level: Level) -> u8 {
    match level {
        Level::User => 0,
        Level::Admin => 1,
        Level::System => 2,
        Level::Ti => 3,
    }
}

/// `effective = max(floor, step)`, escalate-only (spec §9, invariant 24): a step may raise the
/// level above the tweak's floor, never lower it. `step: None` means the effect declared no level
/// of its own, so the floor alone decides.
pub fn effective_level(floor: Level, step: Option<Level>) -> Level {
    match step {
        Some(step) if rank(step) > rank(floor) => step,
        _ => floor,
    }
}

/// Whether `s` is a user-hive (HKCU) registry/registry-key effect (spec §9's HKCU exception) --
/// the only two `Setting` variants that carry a `Hive` at all.
fn is_hkcu(s: &Setting) -> bool {
    match s {
        Setting::Registry(addr) => addr.hive == Hive::Hkcu,
        Setting::RegistryKey(addr) => addr.hive == Hive::Hkcu,
        Setting::Service(_) | Setting::Task(_) | Setting::Hosts(_) | Setting::Firewall(_) => false,
    }
}

/// Whether `effect` is itself a user-hive (HKCU) `Setting` -- shared by [`route`] (drives) and
/// [`read_route`] (reads), so both apply the exact same structural HKCU determination. A
/// `Shared`/`Action` effect is never HKCU by this check: the corpus's `shared:` block addresses its
/// own `Setting` directly, and Actions carry no hive at all.
fn effect_is_hkcu(effect: &EffectDef) -> bool {
    matches!(&effect.kind, Effect::Setting(setting) if is_hkcu(setting))
}

/// Routes one effect's DRIVE to its execution context (spec §9): effective level = `max(tweak's
/// floor, the effect's own declared level)`, EXCEPT a user-hive (HKCU) `Setting` always runs
/// in-process as the interactive user regardless of the floor -- an HKCU write/read-back must
/// never land in an elevated child's own account (ADR-0005). "Is this effect HKCU" is determined
/// structurally, by inspecting the `Setting`'s own `RegAddr`/`KeyAddr.hive` field -- not by any
/// separate flag.
pub fn route(effect: &EffectDef, tweak: &Tweak) -> ExecCx {
    if effect_is_hkcu(effect) {
        return ExecCx::new(Level::User);
    }
    ExecCx::new(effective_level(tweak.elevation, effect.elevation))
}

/// Routes one effect's READ to its execution context (spec §9, invariant 24: "reads run at
/// whatever level the app currently has" -- they NEVER escalate to a tweak's declared floor/step,
/// unlike [`route`]). `current_level` is `Deps::level` -- the elevation the app actually holds
/// right now, the ceiling every read runs at. The one exception mirrors `route`'s: a user-hive
/// (HKCU) `Setting` must still be read in-process as the interactive user regardless of
/// `current_level` -- reading "the current level"'s own hive would read the WRONG account's HKCU
/// the moment `current_level` is ever anything but `User` (e.g. a batch's ceiling reported as
/// `Admin`/`System`/`Ti` for gating purposes elsewhere).
pub fn read_route(effect: &EffectDef, current_level: Level) -> ExecCx {
    if effect_is_hkcu(effect) {
        return ExecCx::new(Level::User);
    }
    ExecCx::new(current_level)
}

/// One step in a tweak's drive plan, already routed to its effective level (spec §9). `id` is
/// carried through opaquely -- grouping itself only ever looks at `level`; a caller maps a group
/// back to the effects it drives via this id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedStep {
    pub id: EffectId,
    pub level: Level,
}

/// The output of [`group_steps`] (spec §9's grouped execution, invariant 18): either one User/Admin
/// step running in-process -- never grouped with a neighbor, even an adjacent same-level one -- or
/// a run of consecutive same-level System/TI steps sharing ONE elevated child.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecGroup {
    InProcess(PlannedStep),
    Batch {
        level: Level,
        steps: Vec<PlannedStep>,
    },
}

/// Groups a routed step sequence for execution (spec §9, invariant 18): consecutive `System`/`Ti`
/// steps at the SAME level share one child; a level change (including `System` -> `Ti`) starts a
/// new group; `User`/`Admin` steps are always their own [`ExecGroup::InProcess`] group, breaking
/// any run around them. Order is preserved throughout -- this only ever batches adjacent equals,
/// never reorders.
pub fn group_steps(steps: &[PlannedStep]) -> Vec<ExecGroup> {
    let mut groups: Vec<ExecGroup> = Vec::new();
    for step in steps {
        match step.level {
            Level::User | Level::Admin => groups.push(ExecGroup::InProcess(step.clone())),
            level => match groups.last_mut() {
                Some(ExecGroup::Batch {
                    level: batch_level,
                    steps: batch_steps,
                }) if *batch_level == level => {
                    batch_steps.push(step.clone());
                }
                _ => groups.push(ExecGroup::Batch {
                    level,
                    steps: vec![step.clone()],
                }),
            },
        }
    }
    groups
}

/// The over-the-shoulder guard's data source (spec §9, ADR-0005 amended): the process token's user
/// SID vs the interactive console session's user SID, each as a SID's textual form (e.g. via
/// `ConvertSidToStringSidW`), so the comparison itself needs no Windows API and stays a pure
/// `String` compare.
pub trait SidProbe {
    fn process_token_sid(&self) -> Option<String>;
    fn console_session_sid(&self) -> Option<String>;
}

/// `true` when the two SIDs differ -- a different admin's credentials elevated the app (ADR-0005's
/// over-the-shoulder case). Either side unreadable (`None`) counts as a mismatch too: the guard's
/// job is to DISABLE User-level tweaks on any doubt, never to assume agreement (spec §9 -- the app
/// never silently escalates, and this extends to never silently assuming the accounts agree).
pub fn sid_mismatch(probe: &dyn SidProbe) -> bool {
    match (probe.process_token_sid(), probe.console_session_sid()) {
        (Some(process), Some(console)) => process != console,
        _ => true,
    }
}

/// The guard's flagging half (spec §9): `true` only when `level` is `Level::User` AND the SID
/// guard detected a mismatch -- the shape the command layer (a later task) checks before allowing
/// a User-level (HKCU-touching) tweak's apply. Never flags Admin/System/Ti: the over-the-shoulder
/// mismatch only ever matters for the in-process-as-the-real-user path (spec §9: "User-level
/// (HKCU-touching) tweaks are disabled").
pub fn user_level_disabled_by_sid_mismatch(level: Level, mismatch: bool) -> bool {
    mismatch && level == Level::User
}

/// The real Windows [`SidProbe`]: `GetTokenInformation`/`TokenUser` on the current process's token
/// for the process side; `WTSGetActiveConsoleSessionId` -> `WTSQueryUserToken` ->
/// `GetTokenInformation`/`TokenUser` for the interactive console session's side.
pub struct RealSidProbe;

impl SidProbe for RealSidProbe {
    fn process_token_sid(&self) -> Option<String> {
        windows_impl::process_token_sid()
    }
    fn console_session_sid(&self) -> Option<String> {
        windows_impl::console_session_sid()
    }
}

mod windows_impl {
    use windows_sys::Win32::Foundation::{CloseHandle, LocalFree, FALSE, HANDLE};
    use windows_sys::Win32::Security::Authorization::ConvertSidToStringSidW;
    use windows_sys::Win32::Security::{GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER};
    use windows_sys::Win32::System::RemoteDesktop::{
        WTSGetActiveConsoleSessionId, WTSQueryUserToken,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    /// The textual SID (`ConvertSidToStringSidW`) belonging to an already-open token handle.
    /// `None` on any API failure -- [`super::sid_mismatch`] treats "couldn't determine" as a
    /// mismatch, never a silent pass, so failing closed here is the correct, honest answer.
    ///
    /// # Safety
    /// `token` must be a valid, currently-open token handle with at least `TOKEN_QUERY` access.
    unsafe fn token_sid_string(token: HANDLE) -> Option<String> {
        let mut needed: u32 = 0;
        // Two-call pattern: the first discovers the required buffer size.
        GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut needed);
        if needed == 0 {
            return None;
        }
        let mut buf = vec![0u8; needed as usize];
        if GetTokenInformation(
            token,
            TokenUser,
            buf.as_mut_ptr().cast(),
            needed,
            &mut needed,
        ) == FALSE
        {
            return None;
        }
        // SAFETY: `buf` was sized to exactly `needed` bytes by the successful call above, which
        // for `TokenUser` always writes a `TOKEN_USER` header followed by its variable-length SID.
        let token_user = &*(buf.as_ptr().cast::<TOKEN_USER>());
        let mut sid_wide: *mut u16 = std::ptr::null_mut();
        if ConvertSidToStringSidW(token_user.User.Sid, &mut sid_wide) == FALSE || sid_wide.is_null()
        {
            return None;
        }
        let len = (0..).take_while(|&i| *sid_wide.add(i) != 0).count();
        let text = String::from_utf16_lossy(std::slice::from_raw_parts(sid_wide, len));
        LocalFree(sid_wide.cast());
        Some(text)
    }

    pub(super) fn process_token_sid() -> Option<String> {
        // SAFETY: `GetCurrentProcess` returns a pseudo-handle that never needs closing; the token
        // handle `OpenProcessToken` produces is closed below on every path.
        unsafe {
            let mut token: HANDLE = std::ptr::null_mut();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == FALSE {
                return None;
            }
            let sid = token_sid_string(token);
            CloseHandle(token);
            sid
        }
    }

    pub(super) fn console_session_sid() -> Option<String> {
        const NO_SESSION: u32 = 0xFFFF_FFFF;
        // SAFETY: `WTSGetActiveConsoleSessionId` takes no arguments; `WTSQueryUserToken`'s output
        // handle is only read once the call reports success, and is closed below on every path.
        unsafe {
            let session = WTSGetActiveConsoleSessionId();
            if session == NO_SESSION {
                return None; // no interactive session attached (e.g. a service session)
            }
            let mut token: HANDLE = std::ptr::null_mut();
            if WTSQueryUserToken(session, &mut token) == FALSE {
                return None;
            }
            let sid = token_sid_string(token);
            CloseHandle(token);
            sid
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::model::{KeyAddr, RegAddr, RegType, RiskLevel, SvcAddr};

    // --- effective_level ------------------------------------------------------------------------

    #[test]
    fn effective_level_is_max_escalate_only() {
        let levels = [Level::User, Level::Admin, Level::System, Level::Ti];
        for &floor in &levels {
            assert_eq!(
                effective_level(floor, None),
                floor,
                "no step-level override must keep the floor"
            );
            for &step in &levels {
                let expected = if rank(step) > rank(floor) {
                    step
                } else {
                    floor
                };
                assert_eq!(
                    effective_level(floor, Some(step)),
                    expected,
                    "floor={floor:?} step={step:?}"
                );
            }
        }
    }

    // --- route / HKCU exception -------------------------------------------------------------------

    fn tweak_with_floor(floor: Level) -> Tweak {
        Tweak {
            id: "demo".to_string(),
            name: "demo".to_string(),
            description: String::new(),
            category: "misc".to_string(),
            info: None,
            warning: None,
            requires_reboot: false,
            risk_level: RiskLevel::Low,
            elevation: floor,
            reversible: true,
            surface: Vec::new(),
            options: Vec::new(),
            windows: None,
        }
    }

    fn registry_effect(hive: Hive) -> EffectDef {
        EffectDef {
            id: EffectId("reg".to_string()),
            kind: Effect::Setting(Setting::Registry(RegAddr {
                hive,
                path: "Software\\Test".to_string(),
                name: "V".to_string(),
                ty: RegType::Dword,
                field: None,
            })),
            elevation: None,
            optional: false,
            if_missing: None,
            windows: None,
        }
    }

    fn key_effect(hive: Hive) -> EffectDef {
        EffectDef {
            id: EffectId("key".to_string()),
            kind: Effect::Setting(Setting::RegistryKey(KeyAddr {
                hive,
                path: "Software\\Test".to_string(),
            })),
            elevation: None,
            optional: false,
            if_missing: None,
            windows: None,
        }
    }

    #[test]
    fn hkcu_ignores_floor() {
        let tweak = tweak_with_floor(Level::System);

        let hkcu = registry_effect(Hive::Hkcu);
        assert_eq!(
            route(&hkcu, &tweak).level(),
            Level::User,
            "an HKCU registry effect must run as the interactive user regardless of a System floor"
        );

        let hkcu_key = key_effect(Hive::Hkcu);
        assert_eq!(
            route(&hkcu_key, &tweak).level(),
            Level::User,
            "the HKCU exception applies to RegistryKey too, not just Registry"
        );

        let hklm = registry_effect(Hive::Hklm);
        assert_eq!(
            route(&hklm, &tweak).level(),
            Level::System,
            "an HKLM effect must still get the tweak's declared floor"
        );
    }

    #[test]
    fn route_escalates_a_step_above_the_floor() {
        let tweak = tweak_with_floor(Level::Admin);
        let mut svc_effect = EffectDef {
            id: EffectId("svc".to_string()),
            kind: Effect::Setting(Setting::Service(SvcAddr {
                name: "Spooler".to_string(),
            })),
            elevation: Some(Level::Ti),
            optional: false,
            if_missing: None,
            windows: None,
        };
        assert_eq!(route(&svc_effect, &tweak).level(), Level::Ti);

        svc_effect.elevation = None;
        assert_eq!(
            route(&svc_effect, &tweak).level(),
            Level::Admin,
            "no step-level override keeps the floor"
        );
    }

    // --- read_route (invariant 24: reads never escalate) -------------------------------------------

    #[test]
    fn read_route_never_escalates_to_the_effects_declared_level() {
        // Unlike `route`, an effect's own declared elevation (or a tweak's floor) must never be
        // consulted for a READ -- only `current_level` (Deps::level, the ceiling the app actually
        // holds right now) decides, for every non-HKCU Setting.
        let mut svc_effect = EffectDef {
            id: EffectId("svc".to_string()),
            kind: Effect::Setting(Setting::Service(SvcAddr {
                name: "Spooler".to_string(),
            })),
            elevation: Some(Level::Ti),
            optional: false,
            if_missing: None,
            windows: None,
        };
        for &current in &[Level::User, Level::Admin, Level::System, Level::Ti] {
            assert_eq!(
                read_route(&svc_effect, current).level(),
                current,
                "a read must stay at current_level={current:?} regardless of the effect's own declared Ti"
            );
        }
        svc_effect.elevation = None;
        assert_eq!(read_route(&svc_effect, Level::Admin).level(), Level::Admin);
    }

    #[test]
    fn read_route_still_forces_hkcu_to_user_regardless_of_current_level() {
        // A read at "the current level" must still land on the interactive user's own hive, never
        // whatever account `current_level` nominally denotes -- the same correctness reason
        // `route` forces HKCU drives to User regardless of the floor.
        let hkcu = registry_effect(Hive::Hkcu);
        for &current in &[Level::User, Level::Admin, Level::System, Level::Ti] {
            assert_eq!(
                read_route(&hkcu, current).level(),
                Level::User,
                "an HKCU read must stay User even when current_level={current:?}"
            );
        }

        let hklm = registry_effect(Hive::Hklm);
        assert_eq!(
            read_route(&hklm, Level::System).level(),
            Level::System,
            "a non-HKCU read follows current_level exactly"
        );
    }

    // --- grouping --------------------------------------------------------------------------------

    fn step(id: &str, level: Level) -> PlannedStep {
        PlannedStep {
            id: EffectId(id.to_string()),
            level,
        }
    }

    #[test]
    fn grouping_preserves_order_and_boundaries() {
        // U, S, S, T, T, S -> [U] [S,S] [T,T] [S]
        let steps = vec![
            step("1", Level::User),
            step("2", Level::System),
            step("3", Level::System),
            step("4", Level::Ti),
            step("5", Level::Ti),
            step("6", Level::System),
        ];
        let groups = group_steps(&steps);
        assert_eq!(groups.len(), 4, "got {groups:?}");

        assert!(matches!(&groups[0], ExecGroup::InProcess(s) if s.id == EffectId("1".into())));

        match &groups[1] {
            ExecGroup::Batch { level, steps } => {
                assert_eq!(*level, Level::System);
                assert_eq!(
                    steps.iter().map(|s| &s.id.0).collect::<Vec<_>>(),
                    ["2", "3"]
                );
            }
            other => panic!("expected Batch, got {other:?}"),
        }
        match &groups[2] {
            ExecGroup::Batch { level, steps } => {
                assert_eq!(*level, Level::Ti);
                assert_eq!(
                    steps.iter().map(|s| &s.id.0).collect::<Vec<_>>(),
                    ["4", "5"]
                );
            }
            other => panic!("expected Batch, got {other:?}"),
        }
        match &groups[3] {
            ExecGroup::Batch { level, steps } => {
                assert_eq!(*level, Level::System);
                assert_eq!(steps.iter().map(|s| &s.id.0).collect::<Vec<_>>(), ["6"]);
            }
            other => panic!("expected Batch, got {other:?}"),
        }
    }

    #[test]
    fn admin_never_grouped_into_child() {
        let steps = vec![
            step("a", Level::Admin),
            step("b", Level::Admin),
            step("c", Level::Admin),
        ];
        let groups = group_steps(&steps);
        assert_eq!(
            groups.len(),
            3,
            "Admin steps must never share a child, even when adjacent and same-level"
        );
        assert!(
            groups.iter().all(|g| matches!(g, ExecGroup::InProcess(_))),
            "got {groups:?}"
        );
    }

    #[test]
    fn user_admin_never_grouped_together_either() {
        let steps = vec![step("a", Level::User), step("b", Level::Admin)];
        let groups = group_steps(&steps);
        assert_eq!(groups.len(), 2);
        assert!(groups.iter().all(|g| matches!(g, ExecGroup::InProcess(_))));
    }

    // --- SID guard ---------------------------------------------------------------------------------

    struct FixedSidProbe {
        process: Option<&'static str>,
        console: Option<&'static str>,
    }
    impl SidProbe for FixedSidProbe {
        fn process_token_sid(&self) -> Option<String> {
            self.process.map(str::to_string)
        }
        fn console_session_sid(&self) -> Option<String> {
            self.console.map(str::to_string)
        }
    }

    #[test]
    fn sid_mismatch_disables_user_level() {
        let mismatched = FixedSidProbe {
            process: Some("S-1-5-21-AAA"),
            console: Some("S-1-5-21-BBB"),
        };
        assert!(sid_mismatch(&mismatched));
        assert!(user_level_disabled_by_sid_mismatch(Level::User, true));
        assert!(
            !user_level_disabled_by_sid_mismatch(Level::Admin, true),
            "the guard only ever disables User-level tweaks"
        );
        assert!(!user_level_disabled_by_sid_mismatch(Level::System, true));
        assert!(!user_level_disabled_by_sid_mismatch(Level::Ti, true));
        assert!(
            !user_level_disabled_by_sid_mismatch(Level::User, false),
            "no mismatch -> never disabled"
        );

        let matched = FixedSidProbe {
            process: Some("S-1-5-21-AAA"),
            console: Some("S-1-5-21-AAA"),
        };
        assert!(!sid_mismatch(&matched));

        let unknown_side = FixedSidProbe {
            process: Some("S-1-5-21-AAA"),
            console: None,
        };
        assert!(
            sid_mismatch(&unknown_side),
            "an undeterminable side must fail closed as a mismatch, never assume agreement"
        );
    }
}
