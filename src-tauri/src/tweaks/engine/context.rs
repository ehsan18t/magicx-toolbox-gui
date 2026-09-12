//! Execution-context routing (spec §9, ADR-0005 as amended; invariant 24): effective-level
//! computation (max, escalate-only), the HKCU always-in-process-as-user exception, grouping
//! consecutive System/TI steps into one child, and the over-the-shoulder SID guard. Pure logic —
//! the one seam that touches the OS (the SID lookups) sits behind an injectable [`SidProbe`], so
//! everything else here runs with zero OS contact by default.

use crate::tweaks::kinds::ExecCx;
use crate::tweaks::model::{ActionDef, Corpus, Effect, EffectDef, Hive, Level, Setting, Tweak};

pub use crate::tweaks::model::effective_level;

/// Whether `s` is a user-hive (HKCU) registry/registry-key effect (spec §9's HKCU exception) --
/// the only two `Setting` variants that carry a `Hive` at all.
fn is_hkcu(s: &Setting) -> bool {
    match s {
        Setting::Registry(addr) => addr.hive == Hive::Hkcu,
        Setting::RegistryKey(addr) => addr.hive == Hive::Hkcu,
        Setting::Service(_) | Setting::Task(_) | Setting::Hosts(_) | Setting::Firewall(_) => false,
    }
}

/// Whether `effect` drives a user-hive (HKCU) `Setting` -- the single HKCU determination, shared by
/// [`route`] (drives), [`read_route`] (reads) and [`tweak_touches_hkcu`] (the availability guard),
/// so none of them can drift apart. That drift is not hypothetical: the guard used to key on the
/// tweak's `elevation:` floor while routing keyed on the hive, and the two disagreed about 85 of the
/// 203 tweaks in the corpus.
///
/// Three shapes can name a hive, and all three are inspected:
///
/// - `Effect::Setting` directly, via its `RegAddr`/`KeyAddr`.
/// - `Effect::Shared`, resolved through `corpus.shared` to the block's own `Setting`. This matters
///   even though today's corpus declares zero shared blocks: without it, a shared HKCU setting would
///   be routed by the tweak's floor and driven inside a System/TI child, against that child's own
///   hive. An unresolvable `SharedId` cannot occur in a compiled corpus (the build rejects it).
/// - `Effect::Action(ActionDef::DeleteTree)`, which carries a `KeyAddr` and therefore a hive. A
///   `Script` action carries no address and is never HKCU by this check. Missing the `DeleteTree`
///   case would leave an HKCU *subtree deletion* routed by the floor, i.e. the single most
///   destructive effect kind running against the wrong account's hive.
fn effect_is_hkcu(effect: &EffectDef, corpus: &Corpus) -> bool {
    match &effect.kind {
        Effect::Setting(setting) => is_hkcu(setting),
        Effect::Shared(id) => corpus
            .shared
            .iter()
            .find(|def| def.id == *id)
            .is_some_and(|def| is_hkcu(&def.setting)),
        Effect::Action(ActionDef::DeleteTree { key, .. }) => key.hive == Hive::Hkcu,
        Effect::Action(ActionDef::Script { .. }) => false,
    }
}

/// Whether any effect in `tweak`'s surface drives an HKCU setting -- the availability guard's input
/// (see [`hkcu_disabled_by_sid_mismatch`]). Built on the same [`effect_is_hkcu`] that [`route`]
/// uses, which is the point: one question, one answer, one place.
pub fn tweak_touches_hkcu(tweak: &Tweak, corpus: &Corpus) -> bool {
    tweak
        .surface
        .iter()
        .any(|effect| effect_is_hkcu(effect, corpus))
}

/// Routes one effect's DRIVE to its execution context (spec §9): effective level = `max(tweak's
/// floor, the effect's own declared level)`, EXCEPT a user-hive (HKCU) `Setting` always runs
/// in-process as the interactive user regardless of the floor -- an HKCU write/read-back must
/// never land in an elevated child's own account (ADR-0005). "Is this effect HKCU" is determined
/// structurally, by inspecting the `Setting`'s own `RegAddr`/`KeyAddr.hive` field -- not by any
/// separate flag. `corpus` is needed only to resolve an `Effect::Shared` to the block it names.
pub fn route(effect: &EffectDef, tweak: &Tweak, corpus: &Corpus) -> ExecCx {
    if effect_is_hkcu(effect, corpus) {
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
pub fn read_route(effect: &EffectDef, current_level: Level, corpus: &Corpus) -> ExecCx {
    if effect_is_hkcu(effect, corpus) {
        return ExecCx::new(Level::User);
    }
    ExecCx::new(current_level)
}

/// The over-the-shoulder guard's data source (spec §9, ADR-0005 amended): the process token's user
/// SID vs the user who owns **this process's own session**, each as a SID's textual form (e.g. via
/// `ConvertSidToStringSidW`), so the comparison itself needs no Windows API and stays a pure
/// `String` compare.
///
/// Deliberately the process's own session, not the *console* session: under RDP the app can
/// legitimately run in session 2 while a different user holds the console, and comparing against
/// the console would report a mismatch between two accounts that were never in conflict. The
/// over-the-shoulder threat is about the session this process actually writes into.
pub trait SidProbe {
    fn process_token_sid(&self) -> Option<String>;
    fn session_user_sid(&self) -> Option<String>;

    /// `DOMAIN\User` for each side, used ONLY when the SID comparison could not be made.
    ///
    /// Resolving the session owner's SID goes through `LookupAccountNameW`, which for a domain
    /// account is a directory lookup: on a domain-joined machine with an unreachable DC, or an
    /// Entra-joined machine whose name provider does not answer, it fails. Without a fallback that
    /// yields `Undetermined`, which blocks every HKCU tweak -- the very failure this guard was
    /// rewritten to stop causing. These two names need no directory: one comes from the process's
    /// own token, the other from the session WTS already tracks.
    ///
    /// Defaulted so test fakes need not implement them.
    fn process_account_name(&self) -> Option<String> {
        None
    }
    fn session_account_name(&self) -> Option<String> {
        None
    }
}

/// The guard's outcome. Three states rather than a `bool` so the command layer can tell the user
/// the truth: an unreadable SID is not evidence that two accounts differ, and reporting it as
/// "another administrator elevated this app" would be a fabricated accusation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidCheck {
    /// Both SIDs read, and equal: this process writes the session owner's own hive.
    SameUser,
    /// Both SIDs read, and different: ADR-0005's over-the-shoulder case.
    DifferentUser,
    /// At least one side could not be read, so the question is genuinely unanswered.
    Undetermined,
}

impl SidCheck {
    /// Whether an HKCU write must be refused. `Undetermined` refuses too -- see [`sid_check`].
    pub fn blocks_hkcu(self) -> bool {
        !matches!(self, SidCheck::SameUser)
    }
}

/// Compares the process token's user against the session owner's (spec §9, ADR-0005 amended).
///
/// An unreadable side yields [`SidCheck::Undetermined`], which still blocks. Failing closed is
/// deliberate, and the reason is the *asymmetry of the two errors*, not caution for its own sake:
/// refusing mutates nothing and tells the user what to do, whereas proceeding on an unanswered
/// question can write another account's hive -- and that write is not recoverable, because the
/// snapshot store is keyed to the machine (`snapshot.rs`'s `Entry` carries `machine_guid`, no user)
/// while the hive is keyed to the account, and apply's read-back verification reads the same hive it
/// just wrote, so it confirms itself. See `docs/plans/fix-hkcu-user-level-gate.md`.
pub fn sid_check(probe: &dyn SidProbe) -> SidCheck {
    if let (Some(process), Some(session)) = (probe.process_token_sid(), probe.session_user_sid()) {
        return if process == session {
            SidCheck::SameUser
        } else {
            SidCheck::DifferentUser
        };
    }

    // Fall back to comparing account names. Symmetric by construction: names are only ever compared
    // with names, so a half-resolved state can never masquerade as a difference between accounts.
    match (probe.process_account_name(), probe.session_account_name()) {
        (Some(process), Some(session)) => {
            log::warn!(
                "SID guard could not resolve a SID; comparing account names instead \
                 (process='{process}', session='{session}')"
            );
            if process.eq_ignore_ascii_case(&session) {
                SidCheck::SameUser
            } else {
                SidCheck::DifferentUser
            }
        }
        (process, session) => {
            log::warn!(
                "SID guard could not determine the session owner by SID or by name \
                 (process name readable: {}, session name readable: {}); \
                 treating per-user tweaks as blocked",
                process.is_some(),
                session.is_some()
            );
            SidCheck::Undetermined
        }
    }
}

/// The guard's flagging half (spec §9): `true` when this tweak drives any HKCU setting AND the SID
/// guard did not confirm the accounts agree.
///
/// Keyed on whether the tweak actually *touches HKCU* ([`tweak_touches_hkcu`]), not on its declared
/// `elevation:` floor. The floor and the hive are independent: 31 `admin`-floor tweaks in today's
/// corpus contain HKCU effects, and keying on the floor missed every one of them while
/// simultaneously blocking all 54 `user`-floor tweaks. The hive decides which account's state a
/// write lands in, so the hive is what the guard must ask about.
pub fn hkcu_disabled_by_sid_mismatch(touches_hkcu: bool, check: SidCheck) -> bool {
    touches_hkcu && check.blocks_hkcu()
}

/// The real Windows [`SidProbe`]: `GetTokenInformation`/`TokenUser` on the current process's token
/// for the process side; `ProcessIdToSessionId` -> `WTSQuerySessionInformationW` ->
/// `LookupAccountNameW` for the session-owner side.
pub struct RealSidProbe;

impl SidProbe for RealSidProbe {
    fn process_token_sid(&self) -> Option<String> {
        windows_impl::process_token_sid()
    }
    fn session_user_sid(&self) -> Option<String> {
        windows_impl::session_user_sid()
    }
    fn process_account_name(&self) -> Option<String> {
        windows_impl::process_account_name()
    }
    fn session_account_name(&self) -> Option<String> {
        windows_impl::session_account_name()
    }
}

mod windows_impl {
    use windows_sys::core::PWSTR;
    use windows_sys::Win32::Foundation::{CloseHandle, LocalFree, FALSE, HANDLE};
    use windows_sys::Win32::Security::Authentication::Identity::{
        GetUserNameExW, NameSamCompatible,
    };
    use windows_sys::Win32::Security::Authorization::ConvertSidToStringSidW;
    use windows_sys::Win32::Security::{
        GetTokenInformation, LookupAccountNameW, TokenUser, PSID, SID_NAME_USE, TOKEN_QUERY,
        TOKEN_USER,
    };
    use windows_sys::Win32::System::RemoteDesktop::{
        ProcessIdToSessionId, WTSDomainName, WTSFreeMemory, WTSQuerySessionInformationW,
        WTSUserName, WTS_CURRENT_SERVER_HANDLE, WTS_INFO_CLASS,
    };
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcess, GetCurrentProcessId, OpenProcessToken,
    };

    /// The textual SID (`ConvertSidToStringSidW`) belonging to an already-open token handle.
    /// `None` on any API failure -- [`super::sid_check`] treats "couldn't determine" as blocking,
    /// never a silent pass, so failing closed here is the correct, honest answer.
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
        sid_to_string(token_user.User.Sid)
    }

    /// A SID's textual form (`S-1-5-21-...`). `None` on any API failure.
    ///
    /// # Safety
    /// `sid` must point at a valid SID structure that outlives the call.
    unsafe fn sid_to_string(sid: PSID) -> Option<String> {
        let mut sid_wide: PWSTR = std::ptr::null_mut();
        if ConvertSidToStringSidW(sid, &mut sid_wide) == FALSE || sid_wide.is_null() {
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

    /// The SID of the user who owns the session this process runs in.
    ///
    /// Deliberately NOT `WTSQueryUserToken`: that requires `SE_TCB_NAME`, which only LocalSystem
    /// holds, so it returned `ERROR_PRIVILEGE_NOT_HELD` (1314) for every real run of the app --
    /// elevated or not -- and the guard's fail-closed arm then disabled every HKCU tweak. Nor
    /// `WTSGetActiveConsoleSessionId`: under RDP the console session belongs to a different user
    /// than the one this process serves, which would be a false mismatch. `ProcessIdToSessionId`
    /// plus a session query needs no privilege and asks the question that actually matters.
    pub(super) fn session_user_sid() -> Option<String> {
        account_name_to_sid(&session_account_name()?)
    }

    /// `DOMAIN\User` of whoever owns this process's session, straight from WTS. No directory
    /// lookup, so this still answers on a machine where `LookupAccountNameW` cannot.
    pub(super) fn session_account_name() -> Option<String> {
        let session = own_session_id()?;
        let user = session_info_string(session, WTSUserName)?;
        if user.is_empty() {
            return None; // the session exists but nobody is logged into it
        }
        let domain = session_info_string(session, WTSDomainName)?;
        Some(if domain.is_empty() {
            user
        } else {
            format!("{domain}\\{user}")
        })
    }

    /// `DOMAIN\User` of the process token's own user, via `GetUserNameExW(NameSamCompatible)`.
    /// Reads the token's logon session, so like [`session_account_name`] it needs no directory.
    pub(super) fn process_account_name() -> Option<String> {
        let mut len: u32 = 0;
        // SAFETY: the first call is the documented size probe and fails with `ERROR_MORE_DATA`,
        // writing the required length (including the NUL) into `len`; its return is ignored on
        // purpose. The second call's buffer is sized from that.
        unsafe {
            GetUserNameExW(NameSamCompatible, std::ptr::null_mut(), &mut len);
            if len == 0 {
                return None;
            }
            let mut buf = vec![0u16; len as usize];
            if !GetUserNameExW(NameSamCompatible, buf.as_mut_ptr(), &mut len) {
                return None;
            }
            // On success `len` is the length WITHOUT the terminator.
            buf.truncate(len as usize);
            Some(String::from_utf16_lossy(&buf))
        }
    }

    fn own_session_id() -> Option<u32> {
        let mut session: u32 = 0;
        // SAFETY: `GetCurrentProcessId` takes no arguments and always succeeds; `session` is a
        // live local the callee only writes.
        unsafe {
            if ProcessIdToSessionId(GetCurrentProcessId(), &mut session) == FALSE {
                return None;
            }
        }
        Some(session)
    }

    /// One `WTS_INFO_CLASS` string for `session`. The buffer belongs to WTS, so it is released with
    /// `WTSFreeMemory` rather than dropped.
    fn session_info_string(session: u32, info_class: WTS_INFO_CLASS) -> Option<String> {
        let mut buf: PWSTR = std::ptr::null_mut();
        let mut bytes: u32 = 0;
        // SAFETY: `buf` is only read once the call reports success and is non-null, and is freed
        // on that same path before returning.
        unsafe {
            if WTSQuerySessionInformationW(
                WTS_CURRENT_SERVER_HANDLE,
                session,
                info_class,
                &mut buf,
                &mut bytes,
            ) == FALSE
                || buf.is_null()
            {
                return None;
            }
            // Cap the NUL scan by the byte count WTS reported. These two classes are contractually
            // NUL-terminated, so the cap never truncates in practice; it just means a buffer that
            // somehow is not terminated stops at its own end instead of reading past it.
            let cap = bytes as usize / std::mem::size_of::<u16>();
            let len = (0..cap).take_while(|&i| *buf.add(i) != 0).count();
            let text = String::from_utf16_lossy(std::slice::from_raw_parts(buf, len));
            WTSFreeMemory(buf.cast());
            Some(text)
        }
    }

    /// `DOMAIN\User` -> textual SID, via the standard two-call `LookupAccountNameW` pattern.
    fn account_name_to_sid(account: &str) -> Option<String> {
        let name: Vec<u16> = account.encode_utf16().chain(std::iter::once(0)).collect();
        let mut sid_len: u32 = 0;
        let mut domain_len: u32 = 0;
        let mut name_use: SID_NAME_USE = 0;
        // SAFETY: the first call is the documented size probe and is EXPECTED to fail with
        // `ERROR_INSUFFICIENT_BUFFER`; its return value is deliberately ignored because the sizes
        // it writes are the result we want. The second call's buffers are sized from those.
        unsafe {
            LookupAccountNameW(
                std::ptr::null(),
                name.as_ptr(),
                std::ptr::null_mut(),
                &mut sid_len,
                std::ptr::null_mut(),
                &mut domain_len,
                &mut name_use,
            );
            if sid_len == 0 {
                return None; // the name did not resolve at all
            }
            let mut sid = vec![0u8; sid_len as usize];
            let mut domain = vec![0u16; domain_len.max(1) as usize];
            if LookupAccountNameW(
                std::ptr::null(),
                name.as_ptr(),
                sid.as_mut_ptr().cast(),
                &mut sid_len,
                domain.as_mut_ptr(),
                &mut domain_len,
                &mut name_use,
            ) == FALSE
            {
                return None;
            }
            sid_to_string(sid.as_mut_ptr().cast())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::model::{
        EffectId, KeyAddr, RegAddr, RegType, RiskLevel, SharedDef, SharedId, SvcAddr,
        TypedRegValue, Value,
    };

    // --- effective_level ------------------------------------------------------------------------

    #[test]
    fn effective_level_is_max_escalate_only() {
        // Ascending, so a higher index is a higher level: `expected` needs no ranking helper.
        let levels = [Level::User, Level::Admin, Level::Ti];
        for (floor_rank, &floor) in levels.iter().enumerate() {
            assert_eq!(
                effective_level(floor, None),
                floor,
                "no step-level override must keep the floor"
            );
            for (step_rank, &step) in levels.iter().enumerate() {
                let expected = if step_rank > floor_rank { step } else { floor };
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

    fn shared_effect(id: &str) -> EffectDef {
        EffectDef {
            id: EffectId("shared".to_string()),
            kind: Effect::Shared(SharedId(id.to_string())),
            elevation: None,
            optional: false,
            if_missing: None,
            windows: None,
        }
    }

    fn empty_corpus() -> Corpus {
        Corpus {
            categories: Vec::new(),
            tweaks: Vec::new(),
            shared: Vec::new(),
        }
    }

    /// A corpus whose one `shared:` block (`s1`) addresses `hive`. Today's real corpus declares no
    /// shared blocks at all, so this fixture is the only way to exercise that resolution path.
    fn corpus_with_shared(hive: Hive) -> Corpus {
        Corpus {
            shared: vec![SharedDef {
                id: SharedId("s1".to_string()),
                setting: Setting::Registry(RegAddr {
                    hive,
                    path: "Software\\Shared".to_string(),
                    name: "V".to_string(),
                    ty: RegType::Dword,
                    field: None,
                }),
                value: Value::Reg(TypedRegValue::Dword(1)),
            }],
            ..empty_corpus()
        }
    }

    #[test]
    fn hkcu_ignores_floor() {
        let tweak = tweak_with_floor(Level::Ti);

        let hkcu = registry_effect(Hive::Hkcu);
        assert_eq!(
            route(&hkcu, &tweak, &empty_corpus()).level(),
            Level::User,
            "an HKCU registry effect must run as the interactive user regardless of a Ti floor"
        );

        let hkcu_key = key_effect(Hive::Hkcu);
        assert_eq!(
            route(&hkcu_key, &tweak, &empty_corpus()).level(),
            Level::User,
            "the HKCU exception applies to RegistryKey too, not just Registry"
        );

        let hklm = registry_effect(Hive::Hklm);
        assert_eq!(
            route(&hklm, &tweak, &empty_corpus()).level(),
            Level::Ti,
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
        assert_eq!(
            route(&svc_effect, &tweak, &empty_corpus()).level(),
            Level::Ti
        );

        svc_effect.elevation = None;
        assert_eq!(
            route(&svc_effect, &tweak, &empty_corpus()).level(),
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
        for &current in &[Level::User, Level::Admin, Level::Ti] {
            assert_eq!(
                read_route(&svc_effect, current, &empty_corpus()).level(),
                current,
                "a read must stay at current_level={current:?} regardless of the effect's own declared Ti"
            );
        }
        svc_effect.elevation = None;
        assert_eq!(
            read_route(&svc_effect, Level::Admin, &empty_corpus()).level(),
            Level::Admin
        );
    }

    #[test]
    fn read_route_still_forces_hkcu_to_user_regardless_of_current_level() {
        // A read at "the current level" must still land on the interactive user's own hive, never
        // whatever account `current_level` nominally denotes -- the same correctness reason
        // `route` forces HKCU drives to User regardless of the floor.
        let hkcu = registry_effect(Hive::Hkcu);
        for &current in &[Level::User, Level::Admin, Level::Ti] {
            assert_eq!(
                read_route(&hkcu, current, &empty_corpus()).level(),
                Level::User,
                "an HKCU read must stay User even when current_level={current:?}"
            );
        }

        let hklm = registry_effect(Hive::Hklm);
        assert_eq!(
            read_route(&hklm, Level::Ti, &empty_corpus()).level(),
            Level::Ti,
            "a non-HKCU read follows current_level exactly"
        );
    }

    // --- SID guard ---------------------------------------------------------------------------------

    #[derive(Default)]
    struct FixedSidProbe {
        process: Option<&'static str>,
        session: Option<&'static str>,
        process_name: Option<&'static str>,
        session_name: Option<&'static str>,
    }
    impl SidProbe for FixedSidProbe {
        fn process_token_sid(&self) -> Option<String> {
            self.process.map(str::to_string)
        }
        fn session_user_sid(&self) -> Option<String> {
            self.session.map(str::to_string)
        }
        fn process_account_name(&self) -> Option<String> {
            self.process_name.map(str::to_string)
        }
        fn session_account_name(&self) -> Option<String> {
            self.session_name.map(str::to_string)
        }
    }

    #[test]
    fn name_fallback_answers_when_sid_resolution_fails() {
        // A domain-joined machine with an unreachable DC cannot resolve the session owner's SID.
        // Falling back to names keeps HKCU tweaks working instead of blocking all 85 of them --
        // which is the exact failure this guard was rewritten to stop causing.
        assert_eq!(
            sid_check(&FixedSidProbe {
                process: Some("S-1-5-21-AAA"),
                session: None,
                process_name: Some("CORP\\alice"),
                session_name: Some("CORP\\alice"),
            }),
            SidCheck::SameUser
        );
        assert_eq!(
            sid_check(&FixedSidProbe {
                process: Some("S-1-5-21-AAA"),
                session: None,
                process_name: Some("CORP\\bob"),
                session_name: Some("CORP\\alice"),
            }),
            SidCheck::DifferentUser
        );
        // Windows account names are case-insensitive.
        assert_eq!(
            sid_check(&FixedSidProbe {
                process: None,
                session: None,
                process_name: Some("CORP\\Alice"),
                session_name: Some("corp\\alice"),
            }),
            SidCheck::SameUser
        );
    }

    #[test]
    fn a_half_resolved_state_never_becomes_an_accusation() {
        // Names are only ever compared with names. If a SID is missing on one side and no name
        // pair is available, the answer is Undetermined -- never DifferentUser, which the UI
        // presents as "another account elevated this app".
        for probe in [
            FixedSidProbe {
                process: Some("S-1-5-21-AAA"),
                session: None,
                process_name: Some("CORP\\alice"),
                session_name: None,
            },
            FixedSidProbe {
                process: None,
                session: Some("S-1-5-21-AAA"),
                process_name: None,
                session_name: Some("CORP\\alice"),
            },
        ] {
            assert_eq!(sid_check(&probe), SidCheck::Undetermined);
        }
    }

    #[test]
    fn sid_check_classifies_all_three_outcomes() {
        assert_eq!(
            sid_check(&FixedSidProbe {
                process: Some("S-1-5-21-AAA"),
                session: Some("S-1-5-21-AAA"),
                ..Default::default()
            }),
            SidCheck::SameUser
        );
        assert_eq!(
            sid_check(&FixedSidProbe {
                process: Some("S-1-5-21-AAA"),
                session: Some("S-1-5-21-BBB"),
                ..Default::default()
            }),
            SidCheck::DifferentUser
        );
        for probe in [
            FixedSidProbe {
                process: Some("S-1-5-21-AAA"),
                session: None,
                ..Default::default()
            },
            FixedSidProbe {
                process: None,
                session: Some("S-1-5-21-AAA"),
                ..Default::default()
            },
            FixedSidProbe {
                process: None,
                session: None,
                ..Default::default()
            },
        ] {
            assert_eq!(
                sid_check(&probe),
                SidCheck::Undetermined,
                "an unreadable side is 'I do not know', never 'they agree'"
            );
        }
    }

    #[test]
    fn undetermined_fails_closed() {
        // Pinned deliberately, not incidentally. Proceeding on an unanswered question can write
        // another account's hive, and that write is unrecoverable: the snapshot store is keyed to
        // the machine while the hive is keyed to the account, and apply's read-back verifies the
        // same hive it just wrote. Refusing mutates nothing. See ADR-0005's 2026-07 amendment.
        assert!(SidCheck::Undetermined.blocks_hkcu());
        assert!(SidCheck::DifferentUser.blocks_hkcu());
        assert!(!SidCheck::SameUser.blocks_hkcu());
    }

    #[test]
    fn guard_keys_on_the_hive_not_the_floor() {
        for check in [SidCheck::DifferentUser, SidCheck::Undetermined] {
            assert!(
                hkcu_disabled_by_sid_mismatch(true, check),
                "an HKCU-touching tweak must be blocked under {check:?}"
            );
            assert!(
                !hkcu_disabled_by_sid_mismatch(false, check),
                "a tweak that touches no HKCU state is unaffected by whose session this is"
            );
        }
        assert!(
            !hkcu_disabled_by_sid_mismatch(true, SidCheck::SameUser),
            "confirmed same user -> HKCU is this user's own hive -> never blocked"
        );
    }

    #[test]
    fn tweak_touches_hkcu_sees_every_shape() {
        let corpus = empty_corpus();

        let mut hkcu_tweak = tweak_with_floor(Level::User);
        hkcu_tweak.surface = vec![registry_effect(Hive::Hkcu)];
        assert!(tweak_touches_hkcu(&hkcu_tweak, &corpus));

        // The case the old floor-keyed guard missed entirely: an Admin-floor tweak whose surface
        // still reaches into the user's own hive. 31 of these exist in the shipped corpus.
        let mut admin_floor_hkcu = tweak_with_floor(Level::Admin);
        admin_floor_hkcu.surface = vec![registry_effect(Hive::Hklm), registry_effect(Hive::Hkcu)];
        assert!(
            tweak_touches_hkcu(&admin_floor_hkcu, &corpus),
            "the floor says which privilege is needed, not whose state changes"
        );

        let mut hklm_only = tweak_with_floor(Level::Admin);
        hklm_only.surface = vec![registry_effect(Hive::Hklm)];
        assert!(!tweak_touches_hkcu(&hklm_only, &corpus));

        // A Shared effect resolves through the corpus, so its hive counts as if inlined.
        let shared_corpus = corpus_with_shared(Hive::Hkcu);
        let mut shared_tweak = tweak_with_floor(Level::Admin);
        shared_tweak.surface = vec![shared_effect("s1")];
        assert!(
            tweak_touches_hkcu(&shared_tweak, &shared_corpus),
            "a shared HKCU setting is still an HKCU write"
        );
        assert!(
            !tweak_touches_hkcu(&shared_tweak, &corpus_with_shared(Hive::Hklm)),
            "a shared HKLM setting is not"
        );
    }

    #[test]
    fn route_and_the_guard_agree_on_shared_effects() {
        // The original defect was two answers to "is this HKCU" drifting apart. `route` and
        // `tweak_touches_hkcu` must resolve a Shared effect the same way, or a shared HKCU setting
        // would be flagged by the guard yet still driven inside a System/TI child.
        let shared_corpus = corpus_with_shared(Hive::Hkcu);
        let mut tweak = tweak_with_floor(Level::Ti);
        tweak.surface = vec![shared_effect("s1")];

        assert_eq!(
            route(&tweak.surface[0], &tweak, &shared_corpus).level(),
            Level::User,
            "a shared HKCU setting must drive in-process as the user, not in a TI child"
        );
        assert!(tweak_touches_hkcu(&tweak, &shared_corpus));
    }
}
