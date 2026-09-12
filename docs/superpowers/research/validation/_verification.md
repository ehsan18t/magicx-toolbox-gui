# Adversarial verification record

Findings from the category pass are researcher claims. This document records which of them were
independently attacked by a second agent working from primary sources and told to refute rather than
confirm. Anything listed here has a verification level; anything not listed remains a single-source
researcher claim.

Target platform throughout: Windows 11 24H2 (build 26100) and newer including 25H2, with Windows 10
IoT Enterprise LTSC 2021 as a low-priority secondary target.

## Verification levels

| Level | Meaning |
|---|---|
| `CONFIRMED` | A second agent, prompted to refute, failed to and produced a primary source supporting the claim. |
| `PARTLY CONFIRMED` | The core holds but the original phrasing overstated it. The corrected form is given. |
| `REFUTED` | The original claim is wrong. |
| `OPEN` | Attacked but not settled by available sources. |

## Privacy claims, adjudicated

### 1. Advertising ID hive: PARTLY CONFIRMED

The original claim was that the companion `Enabled` value belongs at HKLM rather than HKCU, implying
the YAML's HKCU write is wrong. The documented half holds and the "HKCU is wrong" half does not.

Microsoft's "Manage connections from Windows operating system components to Microsoft services" page
documents exactly two values for turning off the advertising ID, both in HKLM: `Enabled = 0` under
`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\AdvertisingInfo`, and `DisabledByGroupPolicy = 1`
under `HKLM\SOFTWARE\Policies\Microsoft\Windows\AdvertisingInfo`. The Policy CSP entry
`Privacy/DisableAdvertisingId` is Device-scoped only. HKCU appears nowhere for AdvertisingInfo.

However, no Microsoft source states the HKCU value is inert, and the advertising ID is genuinely a
per-user identity, so "the OS ignores HKCU" is not established.

**Correct action:** the tweak is *incomplete*, not incorrect. It must add the documented
`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\AdvertisingInfo\Enabled = 0`. Keeping the HKCU write
as a supplementary per-user write is defensible; relying on it alone is not.

### 2. Tailored experiences hive: CONFIRMED on scope, OPEN on inertness

`CloudContent.admx` defines `DisableTailoredExperiencesWithDiagnosticData` with `class="User"`, and
the Policy CSP entry `Experience/AllowTailoredExperiencesWithDiagnosticData` is User-scoped only with
a Group Policy mapping under User Configuration. The YAML writes it to HKLM.

Because the policy is `class="User"`, the Group Policy engine will never write or refresh an HKLM
copy. Whether the consuming component additionally ignores an HKLM value is not documented, so do not
assert the write is inert; assert that it is in the wrong hive.

**Correct action:** move the policy effect to
`HKCU\Software\Policies\Microsoft\Windows\CloudContent`.

**Do not conflate two different knobs.** The tweak's other effect,
`HKCU\...\CurrentVersion\Privacy\TailoredExperiencesWithDiagnosticDataEnabled`, is the Settings UI
value and is a separate, undocumented control from the policy value. Both can legitimately exist.

This finding generalized into a corpus-wide audit, recorded in `_policy-hive-audit.md`.

### 3. AllowInputPersonalization is a speech policy: CONFIRMED

The Policy CSP entry `Privacy/AllowInputPersonalization` maps to the Group Policy
"Allow users to enable online speech recognition services", located under Control Panel, Regional and
Language Options, defined in **Globalization.admx**, at
`HKLM\SOFTWARE\Policies\Microsoft\InputPersonalization\AllowInputPersonalization`. Its documented
text is about speech services, not inking.

The genuine inking-and-typing policy is a different one: `AllowLinguisticDataCollection`
("Improve inking and typing recognition") from **TextInput.admx**, at
`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\TextInput\AllowLinguisticDataCollection`.
The correct per-user companions are the `RestrictImplicitTextCollection` and
`RestrictImplicitInkCollection` values the tweak already writes.

**Correct action:** `disable_inking_typing_personalization` is currently mixing a speech policy into
an inking tweak. Either move that effect to the speech tweak, or replace it with
`AllowLinguisticDataCollection` if inking and typing was the intent. The two options are not
equivalent and the choice should be deliberate.

### 4. Two Edge telemetry values are dead: CONFIRMED

Microsoft's Edge browser policy documentation carries an explicit banner on both
`MetricsReportingEnabled` and `SendSiteInfoToImproveServices`: "OBSOLETE: This policy is obsolete and
doesn't work after Microsoft Edge version 88." Both pages still list MSEdge.admx, which is why they
still appear in policy tooling.

**Correct action:** remove both effects from `disable_edge_telemetry`. Half the tweak currently does
nothing. The live replacement for Edge diagnostic control is the `DiagnosticData` policy (Edge 111
and later) alongside the OS-level Windows diagnostic data policy.

### 5. HasAccepted has no default of 1: CONFIRMED

Microsoft documents only the disable direction for
`HKCU\Software\Microsoft\Speech_OneCore\Settings\OnlineSpeechPrivacy\HasAccepted`, namely writing 0.
No Microsoft source gives it a shipped default of 1, and the CSP text confirms the OS position is
that the control is deferred to the user. The value materializes when the user makes a consent
choice in OOBE or in Settings.

**Correct action:** the stock representation is `absent`, not 0 and not 1. The revert must **delete**
the value rather than write 1.

This one is not cosmetic. A revert that writes 1 manufactures a cloud-speech consent the user never
granted. If the engine's state model cannot express "restore to absent" for this effect, it needs a
delete-capable revert rather than a value write.

## Two patterns worth auditing corpus-wide

The verifier noted that three of five claims reduced to the same two root causes. Both are being
tracked as their own workstreams because they are mechanical and therefore findable:

1. **Policy values in the wrong hive relative to their ADMX class.** Audit in progress, results in
   `_policy-hive-audit.md`.
2. **Revert states that write a literal value where the stock state is value-absent.** Already
   identified as the corpus's most common defect class in `README.md`. Claim 5 shows the failure mode
   at its worst: the revert does not merely fail to restore, it fabricates a consent decision.
