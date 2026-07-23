//! The running Windows version (spec §6.6, invariant 22): `RtlGetVersion` for the build --
//! **never** `GetVersionEx`, whose compatibility shim under-reports the build on a process with no
//! matching manifest entry -- plus the `UBR` registry value for the revision, the finer axis
//! `windows: { revision: ... }` scopes against.
//!
//! ## Reconciling `WinVer` with `validate::Milestone`
//! `validate::Milestone { build: u32 }` is build-only by design: `build.rs`'s guards (spec §10)
//! quantify over the declared support matrix, an explicit list of *builds* -- revision/UBR is a
//! finer runtime axis those guards never see, and `validate.rs` stays untouched by this task. At
//! runtime, though, a live machine has a real revision, and `windows: { revision: ... }` must be
//! honored (invariant 22) -- so `WinVer::applies` implements the FULL grammar (products/build AND
//! revision), and callers that only need the build-only shape (`validate::applicable_surface` and
//! friends) get there via [`WinVer::to_milestone`], a cheap field-projection, never a second
//! implementation of those helpers.

use crate::models::RegistryHive;
use crate::services::registry_service;
use crate::tweaks::model::WindowsScope;
use crate::tweaks::parse::expand_product;
use crate::tweaks::validate::{build_expr_contains, Milestone};

use windows_sys::Wdk::System::SystemServices::RtlGetVersion;
use windows_sys::Win32::System::SystemInformation::OSVERSIONINFOW;

/// The running Windows version (spec §6.6). Major/product membership is derivable from `build`
/// (the same range logic `parse::expand_product` already encodes), so `build` + `revision` are the
/// only two fields actually carried.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WinVer {
    pub build: u32,
    pub revision: u32,
}

impl WinVer {
    /// The cheap build-only adapter (see the module docs): every runtime call site that still
    /// drives `validate.rs`'s Milestone-shaped helpers goes through this rather than a duplicate
    /// implementation.
    pub fn to_milestone(self) -> Milestone {
        Milestone { build: self.build }
    }
}

/// The real running Windows version: `RtlGetVersion` for the build, `UBR` for the revision.
pub fn running_winver() -> WinVer {
    WinVer {
        build: running_build(),
        revision: running_ubr(),
    }
}

/// `RtlGetVersion` (ntdll) rather than `GetVersionEx`/`GetVersionExW` (spec §6.6): the latter is
/// subject to the application-compatibility shim, which under-reports the OS version to a process
/// whose manifest does not declare `supportedOS` entries up to the running release -- exactly the
/// unmanifested case this app ships as. `RtlGetVersion` reports the true build unconditionally.
fn running_build() -> u32 {
    let mut info: OSVERSIONINFOW = unsafe { std::mem::zeroed() };
    info.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;
    // SAFETY: `info` is a correctly-sized, zeroed stack value of exactly the type `RtlGetVersion`
    // expects; the call only ever writes through the pointer, never reads uninitialized fields.
    let status = unsafe { RtlGetVersion(&mut info) };
    if status != 0 {
        // NTSTATUS success is 0; RtlGetVersion is documented to always succeed for this input
        // shape, so a nonzero status here means something is deeply wrong with the process --
        // report 0 rather than a fabricated build (invariant 3's "never guess" spirit applies here
        // too, even outside the detect/apply Value domain).
        log::warn!("RtlGetVersion failed with NTSTATUS {status:#x}; reporting build 0");
        return 0;
    }
    info.dwBuildNumber
}

const CURRENT_VERSION_KEY: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion";

/// The Update Build Revision -- the part after the dot in e.g. `26100.2314` (spec §6.6's
/// `revision` axis). Absent on very old builds that predate UBR; `0` is the honest default there,
/// never a fabricated value.
fn running_ubr() -> u32 {
    match registry_service::read_dword(&RegistryHive::Hklm, CURRENT_VERSION_KEY, "UBR") {
        Ok(Some(n)) => n,
        Ok(None) => 0,
        Err(e) => {
            log::warn!("could not read UBR: {e}; reporting revision 0");
            0
        }
    }
}

impl WindowsScope {
    /// The full §6.6 grammar evaluated against a real running [`WinVer`]: `products` (set
    /// membership over build ranges) AND `build` AND `revision` -- ANDed together, exactly as
    /// authored. Reuses `validate::build_expr_contains` and `parse::expand_product` -- the same
    /// grammar `validate::scope_admits` (build-time, Milestone-only) already implements -- rather
    /// than reimplementing it; the only addition here is the `revision` axis `scope_admits`
    /// intentionally omits (see that function's own docs).
    pub fn applies(&self, v: &WinVer) -> bool {
        let build_ok = self
            .build
            .is_none_or(|expr| build_expr_contains(expr, v.build));
        let product_ok = self.products.as_ref().is_none_or(|products| {
            products
                .iter()
                .any(|&p| expand_product(p).is_ok_and(|expr| build_expr_contains(expr, v.build)))
        });
        let revision_ok = self
            .revision
            .is_none_or(|expr| build_expr_contains(expr, v.revision));
        build_ok && product_ok && revision_ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::model::BuildExpr;

    fn winver(build: u32, revision: u32) -> WinVer {
        WinVer { build, revision }
    }

    fn scope(
        products: Option<Vec<u8>>,
        build: Option<BuildExpr>,
        revision: Option<BuildExpr>,
    ) -> WindowsScope {
        WindowsScope {
            products,
            build,
            revision,
        }
    }

    #[test]
    fn unconstrained_scope_always_applies() {
        assert!(scope(None, None, None).applies(&winver(19045, 9999)));
    }

    #[test]
    fn products_gate_by_build_range() {
        let win10_only = scope(Some(vec![10]), None, None);
        assert!(
            win10_only.applies(&winver(19045, 0)),
            "19045 is in Win10's range"
        );
        assert!(
            !win10_only.applies(&winver(22621, 0)),
            "22621 is Win11, out of Win10's range"
        );

        let win11_only = scope(Some(vec![11]), None, None);
        assert!(win11_only.applies(&winver(22631, 0)));
        assert!(!win11_only.applies(&winver(19045, 0)));
    }

    #[test]
    fn build_expr_axes_min_max_range_exact() {
        assert!(scope(None, Some(BuildExpr::Min(26100)), None).applies(&winver(26100, 0)));
        assert!(!scope(None, Some(BuildExpr::Min(26100)), None).applies(&winver(22631, 0)));
        assert!(scope(None, Some(BuildExpr::Max(22631)), None).applies(&winver(19045, 0)));
        assert!(!scope(None, Some(BuildExpr::Max(22631)), None).applies(&winver(26100, 0)));
        assert!(scope(None, Some(BuildExpr::Range(22000, 22631)), None).applies(&winver(22621, 0)));
        assert!(scope(None, Some(BuildExpr::Exact(26100)), None).applies(&winver(26100, 0)));
        assert!(!scope(None, Some(BuildExpr::Exact(26100)), None).applies(&winver(26120, 0)));
    }

    /// The scenario the whole `WinVer`/`Milestone` split exists for: `revision` only ever narrows
    /// within one exact build, and it must actually be honored at runtime (invariant 22) -- unlike
    /// `validate::scope_admits`, which deliberately ignores it (build-time guards are build-only).
    #[test]
    fn revision_on_exact_build_narrows_within_that_build() {
        let scoped = scope(
            None,
            Some(BuildExpr::Exact(26100)),
            Some(BuildExpr::Min(2314)),
        );
        assert!(
            scoped.applies(&winver(26100, 2314)),
            "at the revision floor, on the pinned build -- must apply"
        );
        assert!(
            scoped.applies(&winver(26100, 5000)),
            "above the revision floor -- must apply"
        );
        assert!(
            !scoped.applies(&winver(26100, 100)),
            "below the revision floor, same build -- must not apply"
        );
        assert!(
            !scoped.applies(&winver(22631, 5000)),
            "revision alone cannot rescue a build that doesn't match at all"
        );
    }

    #[test]
    fn all_three_axes_and_together() {
        let scoped = scope(
            Some(vec![11]),
            Some(BuildExpr::Exact(26100)),
            Some(BuildExpr::Max(1999)),
        );
        assert!(scoped.applies(&winver(26100, 1999)));
        assert!(
            !scoped.applies(&winver(26100, 2000)),
            "revision axis rejects"
        );
        assert!(
            !scoped.applies(&winver(19045, 1999)),
            "build axis rejects (also fails products)"
        );
    }

    /// Read-only, no elevation -- the one live check this file needs (winver's own gate, spec
    /// brief): the real running build on the dev machine must be a plausible modern Windows 10/11
    /// build, proving `RtlGetVersion` (not the `GetVersionEx` compat-shim value) is really wired.
    #[test]
    #[ignore = "reads the real running Windows version -- machine-dependent, run explicitly"]
    fn real_running_winver_is_plausible() {
        let v = running_winver();
        assert!(
            v.build >= 19045,
            "expected a Windows 10 22H2+ build on the dev machine, got {}",
            v.build
        );
    }
}
