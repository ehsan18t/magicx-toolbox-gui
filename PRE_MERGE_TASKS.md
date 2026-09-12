# Pre-Merge Tasks (branch: tweak-system-redesign)

Temporary tracker for work that must land on this branch **before it merges to `main`**.
Delete this file as part of the final pre-merge cleanup.

Status: `[ ]` todo, `[~]` researched / designed (needs a decision), `[x]` done.

---

## 1. Shared-address multi-value / sub-value support `[deferred]`

> **Decision (2026-07-23): deferred, not building it now.** The common case (independent named values
> under one key) already works; parking this rather than over-engineering a speculative feature.
> Revisit only if a real tweak added in task 3 actually needs bitmask or shared-value support. The
> analysis below is kept as ready reference so we don't re-derive it.
>
> **Confirmed by the collision audit
> ([2026-07-23-address-collision-audit.md](docs/superpowers/research/2026-07-23-address-collision-audit.md)):**
> across all 224 catalog tweaks there is exactly one same-value multi-writer case
> (`DirectXUserGlobalSettings`, a `kv_semicolon` packed REG_SZ where two tweaks own different fields),
> and the existing field-addressing mechanism already handles it. No bitmask or shared-value feature is
> needed for this corpus. Deferral stands on evidence, not just triage.

**The ask:** a shared registry area (e.g. the Windows Update policy region) can hold several values;
Tweak A needs one, Tweak B needs another, and today there is no clean way to model that without a
collision.

### What actually works today (verified against the code)

The one-owner uniqueness guard keys on **`(hive, key-path, value-name)`** (`validate.rs:295-317`,
`CoarseKey::Registry`). The packed `field` is a *second* tier under that key. Concretely:

| Two tweaks touch... | Result today |
|---|---|
| **Different value-names** under the same key (`AU\NoAutoUpdate` vs `AU\AUOptions`) | **Allowed.** Independent addresses, no collision. |
| Different **fields** of the same `kv_semicolon` packed value | **Allowed.** Each field owned once. |
| The **same whole value** | Build error `DuplicateAddress` (`validate.rs:385-395`). |
| The **same field** of one packed value | Build error `DuplicateAddress` (`validate.rs:404-409`). |

So the literal "four values under a key, one tweak per value" case is **already handled** as independent
addresses. That is the important framing correction.

### The real gaps

1. **A Shared Setting is strictly one address + one target value.** `SharedDef { id, setting, value }`
   (`model.rs:326-331`). Claim/release is refcounted by shared-id: first claim captures the live
   original, last release restores that single captured value (`shared_claims.rs:208-323`). Every
   claimant must therefore want the *same* value. There is no shared address where two tweaks want
   *different* values, and no shared address claimed at *field* granularity.
2. **No bitmask / sub-value addressing.** The only sub-value addressing is `kv_semicolon` string fields
   (REG_SZ/REG_EXPAND_SZ). Many Windows policies pack several independent behaviors into the bits of
   one DWORD; there is no way for two tweaks to each own a different bit-range of one DWORD.
3. **Latent coordination gap (existence vs value).** A `registry_key` (key-existence) claim and a
   `registry` value claim under that same key are *different* `CoarseKey` variants
   (`RegistryKey(hive,path)` vs `Registry(hive,path,name)`), so the guard never relates them. A tweak
   that manages a key's existence and a tweak that manages a value inside it are invisible to each
   other. Worth deciding whether that is an intended gap.

### If revisited: which scenario is it?

These lead to very different designs, so pin the scenario (a real key + value(s) + the two tweaks)
before building anything:

- **(A) Independent named values under one key** ("four values, one tweak each"): already works. The
  only real work is optional coordination for gap #3 (key existence vs the values inside it).
- **(B) One DWORD, different bits per tweak** (bitmask policy): needs a new `bitmask` addressing mode.
- **(C) The same single value, different whole target values, mutually exclusive** (Tweak A wants
  `AUOptions=1`, Tweak B wants `AUOptions=2`): this is a genuine conflict, not a sharing problem. An
  address holds one value at a time; the honest model is one Tweak with more Options, or an
  exclusivity group, not a shared setting.
- **(D) A shared address whose *original capture + restore* several tweaks must coordinate, each
  driving its own sub-part**: coherent only at sub-value (field/bit) granularity, i.e. it collapses
  into (B).

A concrete real example (the actual key, value(s), and the two tweaks) would settle it immediately.

### Candidate designs (once the scenario is confirmed)

- **B1 - Bitmask field addressing.** Add `format: bitmask` for DWORD/QWORD, analogous to
  `kv_semicolon`: a field addresses a bit or mask; `read = value & mask`, `drive = (value & !mask) |
  target`; the existing per-field ownership guard then lets tweaks coexist (overlapping masks reuse
  the "same field" collision error). Reversible and detectable by construction. Moderate cost: new
  addressing mode, mask math, coverage/`absent` semantics for a bit-range, validation for overlap.
- **B2 - Field-scoped shared claims.** Extend `SharedDef`/`shared_claims` so a shared claim can target
  a field (string field or bitmask), capturing/restoring only that sub-part while refcounting the
  whole value's original once. (A field-addressed `Setting` may already flow through the registry
  kind's read/drive; verify whether field-scoped sharing partially works before building.)
- **C1 - Exclusivity, not sharing.** For true same-value conflicts: model as one Tweak with multiple
  Options, or an authored mutually-exclusive group. Document that sharing cannot resolve a genuine
  single-address conflict.

**Recommendation pending your answer:** if the real corpus needs bitmask policies (likely), do **B1**,
and only add **B2** if a real shared bitmask address appears. Avoid inventing machinery for scenario
(C) unless a real pair demands it.

---

## 2. Corpus serialization: is the YAML -> JSON step wasteful? Should it be binary? `[moved to separate PR]`

> **Decision (2026-07-23): adopt rkyv, in a SEPARATE PR (not this branch).** The premise below
> ("tens-to-low-hundreds of small records") was corrected by the maintainer: the finished corpus will
> be **multi-MB**. That is exactly the regime where rkyv's zero-copy access (no deserialize pass, read
> straight from the embedded buffer) pays off, and the earlier "no change" recommendation no longer
> holds. Out of scope for this branch; tracked here only so the rationale and the known blockers below
> are not lost. Key blockers to solve in that PR: (a) the `Option<serde_json::Value>` schema field has
> no rkyv `Archive` impl (needs an archivable representation), (b) `include_bytes!` gives no alignment
> guarantee (needs an aligned wrapper), (c) rkyv is 0.8.x and version-fragile against the byte-repro CI
> guard (pin the version). `serde_json` still stays for the broker / snapshots / registry `Value`.

**The ask:** we compile YAML to a JSON dump and embed it; JSON is parsed at runtime just like YAML
would be, so why transform at all? And could we embed a binary blob and skip parsing entirely?

### Findings

- **The transform is not wasteful; it splits authoring from loading.** `build.rs` loads the YAML, runs
  full structural + semantic validation, then emits compact JSON (`serde_json::to_string`) which is
  embedded via `include_str!` and parsed once at runtime into a `LazyLock<Corpus>`
  (`build.rs:72-89`, `lib.rs:13`). YAML is the human authoring format; JSON is the pre-validated
  runtime form.
- **Why not keep YAML at runtime?** Both need a parse, but the YAML crate is **build-only** today.
  Keeping YAML at runtime would promote a YAML parser to a *runtime* dependency, for a parser that is
  heavier and slower than `serde_json`, which is *already* a runtime dependency (broker IPC, snapshot
  store, shared-claims store, the in-memory registry `Value` type). JSON is the cheaper, already-present
  runtime form.
- **Is parse cost even a bottleneck? No.** The corpus is parsed **exactly once, lazily, at first
  access** (not per command). For tens-to-low-hundreds of records that is roughly **0.5-2 ms**, and it
  is dwarfed (well under 1%) by Tauri/WebView2 cold-start (~100-500+ ms). Optimizing it optimizes the
  wrong thing.
- **Binary / zero-copy (rkyv) does not pay here.** rkyv is the only option that truly removes the parse,
  but: it is still `0.8.x` (never hit 1.0), pulls the heaviest dependency tree (against our lean,
  Windows-only, avoid-bloat policy), forces `Archived*` types through the schema, is version-fragile
  against our byte-reproducibility CI assertion, and has a **concrete blocker**: the schema's
  `Option<serde_json::Value>` field has no rkyv `Archive` impl. Swapping the corpus format also would
  **not** remove `serde_json` (it stays for the subsystems above).

### Original recommendation (superseded by the decision above)

The research recommended "no change, keep `serde_json`" *on the assumption* of a small corpus, where the
parse is sub-millisecond and rkyv's cost is not justified. That assumption was wrong for the finished
product (multi-MB), so the decision above supersedes this. Retained only to show the reasoning path.
For the record, the format swap itself is a **2-line change** at the corpus load site (`build.rs` emit +
the generated `include_str!`/parse template); the real work in the separate PR is the three blockers
listed in the decision, not the wiring.

---

## 3. Add a real, working tweak corpus `[~]` research done, authoring pending, LAST task before merge

The old collection was discarded (uncertain how real those tweaks were). Before merge, research and
add a curated set of tweaks that genuinely work and are verified on the support matrix (builds 19045,
22621, 22631, 26100). This is deliberately the **last** task on the branch.

**Research delivered and validated (2026-07-23):**
[docs/superpowers/research/2026-07-23-windows-tweak-catalog.md](docs/superpowers/research/2026-07-23-windows-tweak-catalog.md)
is a curated catalog of **207 genuinely-useful tweaks** across 7 domains (Privacy 29, Debloat/AI 31,
Performance/Gaming 24, Services/Tasks 29, UI/UX 34, Network/Update/Power 20, Security 40), each with a
benefit-forward one-liner, the concrete mechanism (registry value / service / task / appx), and a
risk / applicability / reboot tag. It also lists **86 excluded myths** (placebo and cargo-cult tweaks:
IRPStackSize, large system cache, QoS 20%, disable-paging-executive, timer-resolution hacks, etc.) so
we never re-add them. It matches the old corpus description style (verb-first one-liner) and makes the
user's gain explicit.

**Two independent adversarial validation passes** cross-checked every entry against authoritative
sources (Microsoft Learn, admx.help, KB):

- **Pass 1 (mechanism, classification, applicability):** 20 corrections, incl. mechanism fixes
  (`ExtendedUIHoverTime`, the `PowerThrottling` subkey, `NoAutoUpdate=0 + AUOptions=2`, the `WinHttp\DisableWpad`
  value instead of disabling the service), SKU-floor and Enterprise/Education-only applicability fixes,
  six community-only tweaks flagged `Unverified:`, and Wi-Fi Sense demoted to myths.
- **Pass 2 (exact value, type, polarity):** 4 more value/type defects that pass 1 missed, mostly
  REG_SZ-versus-DWORD traps that would have shipped as silent no-ops (`CachedLogonsCount`, the
  screensaver-lock trio `ScreenSaveActive`/`ScreenSaverIsSecure`/`ScreenSaveTimeOut`, and `MinAnimate`
  which also lives under `...\WindowMetrics`), plus a wrong enum (`SmartActiveHoursState` disable is 2, not 0).

- **Pass 3 (whole-catalog holistic):** found the 7-domain fan-out had authored the *same mechanism*
  in 2 to 3 sections with drifted metadata, which inflated the count. Canonicalized to one entry per
  mechanism (224 to 207, ~19 duplicate entries removed), reconciled the risk/reboot/applies_to drift, dropped a self-contradicting
  entry (`dmwappushservice`, which its own myths list rejects), raised an under-labeled lockout risk
  (smart-card service, low to medium), and added two missing high-value tweaks (mouse acceleration,
  accessibility key prompts).

Plus a deterministic integrity check (14 assertions: no dupes, no tweak/myth overlap, honest counts,
every correction present, zero em dashes) that now passes clean. The catalog carries a `Verifier fix:` /
`Round-2 value fix:` / `Consistency fix:` / `Safety fix:` / `Unverified:` audit trail inline. Net: ~28
corrections and ~19 duplicate removals across three passes plus a collision audit. Value-checking and
final consolidation of any remaining partial overlaps remain advisable per-tweak at authoring time.

An address-collision audit
([2026-07-23-address-collision-audit.md](docs/superpowers/research/2026-07-23-address-collision-audit.md))
checked all 272 registry writes and 59 service/task rows for same-address clashes. Result: one packed
REG_SZ field case (handled by existing field addressing), plus ~5 duplicate reg pairs and ~3 duplicate
service/task groups (Recall, Cortana, Copilot button, Explorer Gallery, Game DVR, DiagTrack, CEIP and
AppCompat tasks) that are the same action authored twice under different names and **must be
consolidated before authoring** or they build-error on `DuplicateAddress`. The `WindowsUpdate\AU`
cluster must be authored as one tweak's Surface with multiple Options, not competing tweaks.

**Remaining work:** consolidate the duplicate pairs above, curate the catalog down to an initial
authoring batch, write those as YAML in the new effect-centric schema (per `docs/TWEAK_AUTHORING.md`),
and verify each on the support matrix. The catalog is a research draft, not a commitment to author all
224.

---

## 4. Bugs found during the tweak-system redesign `[moved]`

Two confirmed defects, diagnosed but not yet fixed, now tracked in `docs/KNOWN_ISSUES.md`. Neither
blocks this merge:

1. Snapshots carry no user identity (a shared portable install lets one account's baseline be
   restored into another's hive).
2. Toggling a System Default tweak off stages an apply instead of cancelling the staged change.

---

## Related fix found during research `[x]` done (NOT one of the 3 tasks; this one stays after merge)

The `reproducible-build` CI job grepped for the old artifact name:
`find target/debug/build -name tweaks.json` (`.github/workflows/ci.yml`), but the redesign now emits
**`corpus.json`** (`build.rs:73`). So that job was either hashing a stale cached `tweaks.json`
(guarding nothing) or finding no file. The earlier `touch examples.yaml` fix corrected the rebuild
trigger but not this stale name. **Fixed:** `find_artifact` and the error message now point at
`corpus.json`; verified locally that the corrected job finds the artifact and it is byte-reproducible
across two builds. Unlike this tracker, this is a permanent fix that stays after merge (not yet
committed).
