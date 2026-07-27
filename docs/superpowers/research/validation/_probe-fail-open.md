# Probe fail-open defect

A systematic defect affecting 18 of the corpus's 28 action probes, including every app-removal
tweak in `debloat.yaml`. It violates the project's own "did-it-work" contract in `CLAUDE.md`:

> a failed privileged or effect operation must surface as `Err`, never a benign-looking value.

This finding is derived from the YAML plus engine source plus documented PowerShell semantics, not
from any single machine's state.

## The mechanism

The engine treats a probe's exit code as the sole signal, and exit 0 means "the effect is present":

`src-tauri/src/tweaks/kinds/action.rs:152`

```
/// Exit 0 = present (`Ok(true)`); non-zero = absent (`Ok(false)`). A probe that cannot be run
```

`src-tauri/src/tweaks/kinds/action.rs:161`

```rust
} => Ok(run_script(*shell, &probe.0, ACTION_TIMEOUT)? == 0),
```

Every app-removal probe is shaped like this:

```powershell
if (Get-AppxPackage -AllUsers 'Microsoft.GetHelp') { exit 1 } else { exit 0 }
```

The intent is "if the package is still here, report not-removed; otherwise report removed." The
defect is that **"otherwise" also catches every way the query can fail.**

`Get-AppxPackage` failures are *non-terminating* errors. A non-terminating error does not stop the
script and does not skip the `else`. It writes to the error stream, the expression evaluates to
`$null`, `$null` is falsy, the `else` branch runs, and the probe exits 0. The engine reads exit 0
and reports the removal as successfully applied.

So the probe answers "is this app gone?" with "yes" whenever it fails to find out.

## Why this is not theoretical

The realistic trigger is ordinary and common: **`Get-AppxPackage -AllUsers` requires elevation.**
Run without it, the cmdlet fails with an access error. Under this probe shape that failure is
indistinguishable from success. The app is installed, fully present, and the tool reports it removed.

Other triggers that produce the same wrong answer:

- A package identity that no longer exists on the target build. `remove_teams_consumer_app` queries
  `MicrosoftTeams`, which does not ship on 24H2, so its probe reports "Removed" on a machine that
  never had it and where nothing was done.
- A parameter binding error. `remove_bing_news_weather` passes two names to a parameter typed
  `System.String`, which the binder rejects outright. Its probe therefore reports "Removed"
  unconditionally, whether or not either app is present.
- Any transient WinRT or Appx service failure.

The consequence is the exact inversion of the safety model: a state the tool cannot verify is
displayed to the user as a state it has confirmed.

## Affected probes

18 of 28. All 17 app and component removals in `debloat.yaml`, plus one in `network.yaml`.

| File | Tweak |
|---|---|
| debloat | `remove_copilot_app`, `remove_teams_consumer_app`, `remove_cortana`, `remove_clipchamp`, `remove_dev_home`, `remove_quick_assist`, `remove_bing_news_weather`, `remove_solitaire`, `remove_get_help`, `remove_getstarted_tips`, `remove_feedback_hub`, `remove_maps`, `remove_people`, `remove_phone_link`, `remove_outlook_new`, `remove_xbox_game_bar`, `remove_onedrive` |
| network | `disable_netbios_tcpip` |

`disable_netbios_tcpip` fails open differently: its loop falls through to a trailing `exit 0`, so
any enumeration failure inside the loop still reaches the success exit.

## The ten probes that are already correct

These use the opposite polarity, where the *success* case is the explicit branch and any failure
falls through to a non-zero exit. They are the model to copy.

`remove_recall_feature`, `disable_hibernation`, `disable_usb_selective_suspend`, `disable_wake_timers`,
`ultimate_performance_power_plan`, `disable_memory_compression`, `remove_smbv1`, `remove_powershell_v2`,
`audit_logon_events`, and `disable_nic_power_management` (which exits 1 as its fallthrough).

Note the asymmetry this creates today: an action tweak backed by DISM or powercfg fails safe, while
every Appx removal fails unsafe.

## Fix

The rule to apply: **the probe must prove the desired state, and every path that did not prove it
must exit non-zero.** Never let "the query failed" share a branch with "the thing is gone."

Concretely, make the query failure terminate rather than fall through:

```powershell
$ErrorActionPreference = 'Stop'
try {
    $pkg = Get-AppxPackage -AllUsers -Name 'Microsoft.GetHelp'
} catch {
    exit 1        # could not determine -> not applied
}
if ($pkg) { exit 1 } else { exit 0 }
```

Three points about this shape:

1. `$ErrorActionPreference = 'Stop'` promotes the non-terminating error so `catch` can see it.
   Without it, `catch` never runs and the original defect remains.
2. The `catch` exits 1, so "cannot determine" is reported as not-applied. That is the safe direction:
   it may show a completed removal as pending, which is recoverable, rather than showing an
   incomplete removal as done, which is not.
3. Use `-Name` explicitly, and for multiple packages iterate rather than passing an array, since the
   `Name` parameter is typed `System.String` and rejects arrays at the binder.

A stricter variant worth considering: have the probe distinguish "absent" from "cannot tell" and let
the engine surface the latter as Needs Attention rather than as a clean state, which matches ADR-0001's
treatment of unverifiable outcomes. That is an engine change, not a YAML change, so it is a larger
piece of work than correcting the 18 scripts.

## Recommended priority

High, and above most of the value-level corrections. A wrong registry value produces a tweak that
does not work, which the user can see. A fail-open probe produces a tweak that **reports** it worked
when it did not, which the user cannot see, and which will also mislead the snapshot and revert
machinery about what state the machine is actually in.
