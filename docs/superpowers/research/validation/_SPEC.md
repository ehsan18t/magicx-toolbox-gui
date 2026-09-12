# Tweak validation spec

The contract every category validation document follows. Read this before writing anything.

## Your job

You are validating one category YAML file from `src-tauri/tweaks/`. For every tweak in that file you
must independently confirm, against authoritative sources, that the mechanism the YAML writes is
real, correctly typed, correctly valued, and correctly scoped to the Windows versions claimed. Then
you write a research entry for it.

This is not a proofreading pass. The corpus reaches real machines. A wrong registry type, or an
option that drives the machine to a state you could not source, produces a broken revert, which is
worse than no tweak at all.

## Ground rules

1. **Default to doubt.** A tweak is `UNVERIFIED` until a source confirms it. Never mark something
   `VERIFIED` because it looks plausible or because you have seen it in a tweak script.
2. **Do not read `docs/superpowers/research/2026-07-23-windows-tweak-catalog.md`.** It is a prior
   research pass and using it would launder its errors into this one. Work from the YAML plus your
   own sources only.
3. **Never use an em dash (`—`) in the document you write.** Use a comma, colon, parentheses,
   semicolon, or a separate sentence. This is a hard project rule.
4. **Do not edit any YAML.** This pass is research only. Corrections are recorded in prose.
5. Every factual claim needs a source. If you cannot source it, say so in the entry rather than
   asserting it.

## Source tiers and the corroboration rule

Cite the highest tier you can reach, and label the tier on every source.

| Tier | What counts |
|---|---|
| `A` | Microsoft Learn, the shipped ADMX file content, Microsoft KB / support articles, MSRC advisories |
| `B` | Microsoft engineering blogs (Windows IT Pro blog, Windows Insider blog), CIS / DISA STIG benchmarks, NIST |
| `C` | Established security research (Eclypsium, SpecterOps, Trail of Bits), reputable vendor docs (NVIDIA, AMD, Intel), long-lived community references with a track record (elevenforum, tenforums, Sysinternals-adjacent writeups), mature open-source tweak projects whose behaviour is auditable (privacy.sexy, Chris Titus WinUtil, O&O ShutUp10 documentation) |
| `D` | Anonymous forum posts, Reddit threads, blog aggregators, tweak scripts with no provenance |

**Absence of Microsoft documentation is not evidence against a tweak.** Microsoft documents what it
wants administrators to configure, not everything the OS reads. Large parts of the shell, the
GameConfigStore values, hover timings, several CLSID behaviours, and many per-user preferences are
genuinely undocumented while being entirely real and stable. Treating "Microsoft has no page for it"
as "doubtful" would wrongly demote working tweaks, which is its own kind of error.

So apply this rule:

- A tier `A` or `B` source confirming key, name, type, and semantics gives `VERIFIED`.
- **Three or more independent tier `C` sources that agree on the exact key path, value name, value
  type, and the meaning of each value also give `VERIFIED`.** Independent means genuinely separate
  origins, not three sites that visibly copied one another. Say in the entry that the tweak is
  community-corroborated rather than Microsoft-documented, so the reader knows which kind of
  confidence it is.
- Two tier `C` sources, or sources that disagree on any detail, give `UNVERIFIED`.
- Only tier `D`, or a claim that no source states directly, gives `UNVERIFIED`.

When you mark something `UNVERIFIED`, say which of these cases it is: nobody documents it, sources
disagree, or the sources are all downstream of one another. Those mean different things to a reader.

Reserve `DISPUTED` for a genuine conflict of evidence or a contested real-world effect, not for
"Microsoft is silent."

## Inclusion: the only test

**The product exists to give the user control over their machine.** Whether Microsoft ships a setting
on or off is not an inclusion criterion.

> Does a real control exist that changes something on Windows 11 24H2 or newer?

If yes, include it. If no, exclude it, because there is nothing to control. That is the whole test.

Never demote or question a tweak on the grounds that it is "already the default", "just confirms the
current state", or "changes nothing today". That judges the default when the subject is the control.

The default state still has to be researched carefully, but for two narrow reasons, both about
correctness rather than inclusion:

1. **The revert value** must be the state the machine would be in without the tweak. Getting this
   wrong is the corpus's most damaging defect class. See `_harmful-revert.md`.
2. **Honest description.** If applying produces no visible change today, say so in Drawbacks so the
   user is not hunting for a difference that is not there.

Full reasoning in `_inclusion-principle.md`.

## What to check on every tweak

Go through this list for each one. These are the failure modes that actually occur.

- **Value name spelling and case** exactly as Windows reads it.
- **Registry hive and full key path**, including whether the policy lives under `Policies` or the
  direct product key, and whether the effective one is HKLM or HKCU.
- **Value type.** `REG_SZ` written as `REG_DWORD` is silently ignored by Windows. This is the single
  most common real defect. Flag every type you cannot confirm.
- **Semantics of each numeric value.** Confirm what 0, 1, 2 actually mean. Watch for inverted
  polarity (a value named `Disable*` where 1 means enabled, and similar).
- **Every literal an option writes.** An option is a state the app drives the machine to, so each
  value in one has to be established, including whether the right value is `absent` (not present at
  all) rather than a written zero. Getting this wrong breaks revert. Treat it as high stakes. If a
  state cannot be established, say so: the corpus authors fewer options rather than inventing one.
  **Never label an option as the Windows or stock default.** What Microsoft ships today is a fact
  about this month's Windows, not about the control; label an option by the state it produces.
- **Applicability.** Which Windows 10 versions and Windows 11 builds does this exist on? Is it
  gated to a SKU (Home / Pro / Education / Enterprise / IoT / LTSC)? Is it silently ignored on some
  SKU? Does it need specific hardware (Copilot+ NPU, VRR display, discrete GPU, TPM)?
- **Reboot / sign-out / Explorer-restart requirement**, and whether the YAML's `requires_reboot`
  flag matches reality.
- **Whether the tweak is a no-op** on the target OS range, because the feature was removed or the
  key is no longer read.
- **Whether the effect claimed is real** or is a widely-repeated placebo.
- **Service tweaks:** confirm the service short name exists on Win10 22H2 and Win11, its real
  default start type, whether it is trigger-started, and what breaks when it is disabled.
- **Task tweaks:** confirm the full task path exists, and note builds where it is absent.
- **Action tweaks (PowerShell / powercfg / DISM):** confirm the command is correct, that the `undo`
  genuinely restores the prior state, and that the `probe` actually detects the applied state.

## Output

Write to `docs/superpowers/research/validation/<category>.md`. Use this structure exactly.

````markdown
# <Category name> tweak validation

Validated <date>. Source corpus: `src-tauri/tweaks/<file>.yaml` (<N> tweaks).
Scope: Windows 10 22H2 and Windows 11 22H2 / 23H2 / 24H2 / 25H2, x64.

## Verdict summary

| Tweak | Verdict | Correction needed |
|---|---|---|
| `tweak_id` | VERIFIED | none |
...

## Corrections required

Numbered list of every concrete defect found, each naming the tweak id, the exact wrong thing, and
the exact right thing. If there are none, say so.

## Merge candidates

Groups of tweaks in this file that write overlapping or directly related settings and could
reasonably become one tweak with multiple options. Only propose merges where the settings are part
of the same subsystem and a user would sensibly choose between them rather than combine them. For
each, give the group, the proposed shape, and what granularity would be lost.

## Tweak entries

### `tweak_id` <Display name>

**Verdict:** <one of the verdicts below>

**Mechanism as authored:** the exact effects the YAML writes, copied faithfully.

**What it actually does:** two to four sentences of technical explanation grounded in the sources.

**Applicability:** Windows versions and builds, SKU limits, hardware prerequisites, and whether a
reboot, sign-out, or Explorer restart is required.

**Why use it:** the genuine benefit, stated without marketing.

**When not to use it:** the concrete situations where this is the wrong choice.

**Downside:** what you give up. Write "none of consequence" only if that is genuinely true.

**Cautions:** anything that can bite, including lockout risk, breakage of other features, and
whether Windows updates revert it.

**Corrections needed:** `none`, or a precise description of what the YAML has wrong.

**Sources:**
1. Title, URL (tier A)
2. Title, URL (tier C)
````

## Verdicts

| Verdict | Meaning |
|---|---|
| `VERIFIED` | A tier A or B source confirms the key, value name, type, semantics, and applicability, and the YAML matches. |
| `VERIFIED-WITH-CORRECTION` | The mechanism is real and sourced, but the YAML has a concrete error (wrong type, wrong default, wrong applicability gate, missing companion value, wrong polarity). |
| `UNVERIFIED` | The mechanism is plausible and community-repeated but no tier A/B/C source confirms it. Real-world effect unproven. |
| `DISPUTED` | Sources conflict, or the claimed benefit is contested, or measurement shows the effect is negligible or negative. |
| `INCORRECT` | The key or value does not exist, is not read on the target OS range, has inverted polarity, or the tweak is a no-op. |

Be willing to return `INCORRECT` and `DISPUTED`. A validation pass that confirms everything has
validated nothing.
