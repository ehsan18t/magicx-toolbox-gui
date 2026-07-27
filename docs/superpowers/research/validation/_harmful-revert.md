# Reverts that leave the machine worse than stock

The corpus's most serious defect class. It is distinct from, and worse than, "revert does not fully
restore".

## The claim

For several tweaks, choosing the **System Default** option does not return the machine to its
original state. It writes a state the machine was never in, and in the confirmed cases that state is
**less private or less secure than if the user had never touched the tweak at all**.

This inverts the safety model. ADR-0003 makes System Default a selectable state that performs a
revert, and the whole snapshot design exists so a user can undo safely. A revert that degrades the
machine turns the safety feature into the hazard.

## Why it happens

Windows has three distinct "off" states and the corpus collapses them into one:

1. **Value absent.** Nothing written. Windows applies its internal default.
2. **Value present, set to 0.** An explicit configuration that may differ from absent, and which the
   Group Policy engine and some components treat differently.
3. **Feature shipped disabled.** Windows itself ships the thing off, so "restore the default" means
   leaving it off, not turning it on.

Authoring a revert as a literal `0` or `1` assumes case 2 is equivalent to case 1 or 3. It frequently
is not.

## Evidence provenance, read this before acting

The development machine used during this review is **heavily modified by its owner** and is not a
stock system. Any claim about a Windows *default* that rests on that machine's state is unreliable,
including:

- current registry values, whether present, absent, 0, or 1
- current service start types
- current scheduled-task enabled state, **including the `<Enabled>` element in the task XML under
  `C:\Windows\System32\Tasks`, because enabling or disabling a task rewrites that file**
- `C:\Users\Default\NTUSER.DAT`, which is modifiable and is an LTSC image here

What remains usable, because it describes the product rather than one machine's configuration:

- ADMX and ADML definitions under `C:\Windows\PolicyDefinitions`, for declared class, `valueName`,
  and enabled or disabled values
- the presence of a string inside a shipped binary, as evidence a value name exists and is read,
  never as evidence of its default
- Microsoft documentation, KB articles, and CIS or DISA STIG controls

Findings below are graded accordingly.

## Confirmed, documentation-backed

These do not depend on the local machine.

| Tweak | Revert writes | Real stock state | Consequence |
|---|---|---|---|
| `privacy:disable_online_speech_recognition` | `HasAccepted = 1` | Microsoft documents only the `0` direction, and the CSP states the control is deferred to the user, so the value is consent-derived and absent until the user chooses | **Fabricates a cloud-speech consent the user never gave** |
| `security:require_smb_signing` | `RequireSecuritySignature = 0` | Microsoft documents SMB signing as required by default on Windows 11 24H2 | **Explicitly disables a protection that 24H2 enables by default, leaving the machine less secure than never applying the tweak** |
| `privacy:remove_recall_component` | Value absent | Policy semantics: not-configured leaves Recall **disabled**, not available | The "Available" option does not make Recall available; that requires an explicit `1` |

The Recall case is the mirror image of the other two: it writes `absent` where a literal is required.
Both directions are the same underlying mistake, which is assuming the revert value is obvious rather
than establishing it.

## Suspected, but withdrawn pending a clean image

Each of these was originally reported as confirmed. The evidence was the modified machine's state, so
the finding does not stand as written. They remain plausible and worth checking, but must not be
acted on until confirmed on a clean 24H2 image.

| Tweak | Original claim | Why it is withdrawn |
|---|---|---|
| `services:task_disk_diagnostic_datacollector` | Ships `<Enabled>false</Enabled>`, so revert turns on a collector Windows ships off | Disabling a task rewrites that element. The owner may simply have disabled it. The supporting observation of an empty `<Triggers />` set is more interesting, since no enable or disable operation produces that, but it is not enough on its own |
| `services:task_maps_update` | `MapsUpdateTask` ships disabled | Same contamination. The "untouched image-build timestamp" argument is weakened because file timestamps do not reliably survive servicing |
| `interface:disable_startup_sound` | Ships `DisableStartupSound = 1` | The value read on the machine is very likely the owner's own application of **this exact tweak**, which makes it circular evidence |

## Why this class outranks the others

A wrong registry value produces a tweak that does nothing. The user sees no change and investigates.

A harmful revert produces damage at the moment the user is trying to be careful. The people most
exposed are those who applied a privacy or security tweak, had second thoughts, and backed it out.
They end up worse off than users who never touched it, and nothing in the interface tells them.

It also poisons the snapshot record: the stored "previous state" disagrees with what the machine
ever actually was.

## Priority

1. **`require_smb_signing`.** A security regression, documentation-confirmed, no clean image needed.
2. **`disable_online_speech_recognition`.** Fabricated consent, documentation-confirmed.
3. **`remove_recall_component`.** Its "Available" option is simply wrong about what it does.
4. **Everything else waits on a clean-image baseline.** That baseline is now the blocking dependency
   for this entire workstream.

## The systematic fix

Correcting three tweaks does not close this class. The corpus needs a rule and a check.

**Rule for authors.** A revert value must be established from evidence, never assumed, and the
evidence must describe Windows rather than one machine. Acceptable sources, in order:

1. Microsoft documentation, a KB, or a CIS or DISA STIG control stating the default.
2. The shipped ADMX definition, for policy-backed values.
3. A dump taken from a **known-clean** Windows image of the target build.

Community agreement is **not** sufficient for a revert value, even under the corroboration rule that
governs mechanism verification. A wrong mechanism makes a tweak inert; a wrong revert damages the
machine, so it carries a higher burden.

**Check to add.** A validation rule flagging any tweak whose stock-default option writes a literal
with no recorded provenance for that literal. This is mechanical and would have caught every instance
above.

**Guard against the inverse error.** Do not sweep every literal to `absent`. Some values genuinely
ship seeded, and rewriting those deletes a shipped value, which is the same bug pointing the other
way. Two are documentation-confirmed and must keep their literals: `NetworkThrottlingIndex` (default
10 per KB 948066) and `SystemResponsiveness` (default 20). The others previously listed as seeded
(`SystemPaneSuggestionsEnabled`, `RotatingLockScreenOverlayEnabled`, `SilentInstalledAppsEnabled`,
`PreInstalledAppsEnabled`, `OemPreInstalledAppsEnabled`, `ShowSyncProviderNotifications`) rested on
the modified machine's Default hive and now also need clean-image confirmation.

## The blocking dependency

A **clean-image baseline** is now the single highest-value missing input for this project. It would
resolve, in one pass: the six withdrawn or unconfirmed items above, the roughly one-third of the
interface category whose stock defaults are unknown, the ContentDeliveryManager absent-versus-literal
question affecting six privacy and debloat tweaks, and the SysMain and Delivery Optimization default
questions raised elsewhere.

Practical ways to obtain it without a spare machine:

1. A Windows 11 24H2 evaluation VM from Microsoft, sign in once, then dump the relevant keys, service
   start types, and task states.
2. Mount a clean 24H2 install ISO's `install.wim` and read the offline SOFTWARE, SYSTEM, and
   `Users\Default\NTUSER.DAT` hives, plus the task XML, without booting anything.

Option 2 is cheaper and gives the true shipped state rather than a post-first-logon state, though it
cannot show values Explorer materializes at first interactive logon. Doing both answers everything.
