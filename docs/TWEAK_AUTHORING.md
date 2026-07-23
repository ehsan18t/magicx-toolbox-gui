# Tweak Authoring Guide

This is the **single source of truth** for writing a tweak for the redesigned engine. The syntax here
is invented for this project. It is not a convention you know from `.reg` files, Chris Titus, O&O
ShutUp10, or the old MagicX schema. Every rule, keyword, and edge case is documented here; nothing is
left implicit. A newcomer who has never seen this system should be able to author any tweak correctly
from this document alone.

If you only read one thing: a tweak declares the **surface of state it manages once** (`effects:`), and
each **option** is a flat value-map over that surface. That is the whole mental model. Everything else
is detail.

For the engine architecture see [TWEAK_SYSTEM.md](./TWEAK_SYSTEM.md); for the design rationale see the
spec at
[`docs/superpowers/specs/2026-07-21-tweak-system-redesign-design.md`](./superpowers/specs/2026-07-21-tweak-system-redesign-design.md)
and the decision records under [`docs/adr/`](./adr/). This guide is **self-contained** (you never need
to open the spec to author a tweak) but it cites `spec §N` / `ADR-000N` throughout so you can go deeper.

### Where tweaks live and how they compile

- Tweaks are YAML files in **`src-tauri/tweaks/`** (`*.yaml` or `*.yml`).
- **`build.rs` loads, validates, and compiles them into the binary at compile time.** A mistake in your
  YAML is a **build error**, not a runtime surprise: the app will not compile until every tweak is
  valid on every supported Windows build.
- The validator, the parsers, and the compiled model are the **same Rust code the runtime uses**
  (`src-tauri/src/tweaks/{model,parse,schema,validate}.rs`), included into `build.rs` verbatim. What
  builds is exactly what runs; build and runtime can never disagree.
- The shipped reference corpus is
  [`src-tauri/tweaks/examples.yaml`](../src-tauri/tweaks/examples.yaml): eight tweaks exercising every
  feature. Every fragment in this guide is drawn from it or checked against the shipped validator.

### Table of contents

1. [Introduction & mental model](#1-introduction--mental-model)
2. [Anatomy of a corpus file](#2-anatomy-of-a-corpus-file)
3. [Tweak-level fields](#3-tweak-level-fields)
4. [Every effect kind](#4-every-effect-kind)
5. [Value literals: the complete grammar](#5-value-literals-the-complete-grammar)
6. [The `absent` keyword: deep dive](#6-the-absent-keyword-deep-dive)
7. [Options & the coverage rule](#7-options--the-coverage-rule)
8. [Presence: `optional` / `if_missing`](#8-presence-optional--if_missing)
9. [Shared settings: declared, refcounted](#9-shared-settings-declared-refcounted)
10. [Windows version scoping](#10-windows-version-scoping)
11. [Packed / field-addressed values](#11-packed--field-addressed-values)
12. [The Action contract](#12-the-action-contract)
13. [Elevation](#13-elevation)
14. [Reversibility](#14-reversibility)
15. [Detectability & the status model](#15-detectability--the-status-model)
16. [Build errors reference (all 21)](#16-build-errors-reference-all-21)
17. [Complete worked examples](#17-complete-worked-examples)
18. [What is gone from the old schema](#18-what-is-gone-from-the-old-schema)
19. [Authoring checklist / do's & don'ts](#19-authoring-checklist--dos--donts)

---

## 1. Introduction & mental model

### 1.1 Effect-centric, not option-centric

Almost every other tweak tool (and the **old** MagicX schema) is **option-centric**: each option (or
"profile", or "state") carries its own list of changes. "Disabled" has a `registry_changes` list, a
`service_changes` list, some `pre_commands`; "Enabled" has _different_ lists. Nothing forces the two
options to touch the _same_ set of addresses.

That is the root of most tweak bugs. If "Disabled" writes three registry values and "Enabled" only
writes two of them, the third is silently stranded when you flip back; the machine is now in a state no
option describes, and revert cannot fix what it does not know it changed.

This engine is **effect-centric**. You declare the **managed surface once** (the complete set of
addresses this tweak touches) and then each option is just **one row in a table** whose columns are
those addresses:

```yaml
effects: # the managed surface: the set of addresses, declared ONCE
  - id: no_auto_update
    registry: { key: 'HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU', name: NoAutoUpdate, type: REG_DWORD }
  - id: update_service
    service: { name: wuauserv }

options: # each option supplies ONE value per effect: a table row
  - label: "Disabled"
    values:
      no_auto_update: 1
      update_service: disabled
  - label: "Notify Only"
    values:
      no_auto_update: 1
      update_service: manual
```

| option-centric (old / other tools)           | effect-centric (this engine)                            |
| -------------------------------------------- | ------------------------------------------------------- |
| each option owns a change-list               | the tweak owns one address surface                      |
| options can touch different address sets     | **every option values every effect** (build-enforced)   |
| stranded state on flip-back is common        | stranded state is structurally impossible               |
| "the default" is often authored as an option | "System Default" is **computed**, never authored        |
| detection is bespoke per tweak               | detection is `read the surface, compare to each option` |

Because every option must supply a value for every effect (the **coverage rule**, §7), two options can
never touch inconsistent address sets. Revert always knows the complete surface. This is the single most
important idea in the system.

### 1.2 A tweak = a surface + value-maps over it

Precisely:

- A **tweak** declares a `surface`: a list of **effects**. Each effect is one address (a registry
  value, a service, a scheduled task, …) or one imperative action.
- A tweak declares **options**. Each option is a `label` plus a `values` map: for every effect on the
  surface, what value that option wants there.
- **A `Value` is the one currency** shared by read, apply, detect, and revert. There is exactly one
  comparison per kind, so those four operations can never disagree about what a state "is".

### 1.3 "System Default" is a computed status, never authored

There is **no "System Default" option and no "revert to default" button** (ADR-0003). "System Default"
is a **status the app computes** when the live surface matches _none_ of your authored options, because
the author never defined that exact state, or the machine drifted out of every defined one.

You author **only the real states you offer.** The app supplies the rest of the story:

- **1 authored option → a toggle** (Default ↔ On).
- **≥2 authored options → a dropdown** (Default / A / B / …).

The "Default" position in the UI is always this computed status, never a target you write. Returning
toward a previous state happens only through **Restore Snapshot**, which walks the tweak's captured
history (§15, ADR-0003/0007).

### 1.4 Why this design

- **Drift-proof.** One declared surface means revert always has the full picture; no option can strand
  state another option owns.
- **One representation.** `read`, `apply`, `detect`, `revert` all speak the same `Value` per kind, so
  "did it work?" and "what is it now?" are the same comparison: never a fabricated success.
- **Honest by construction.** Undetectable options, colliding owners, and lying `reversible` flags are
  **build errors**, caught before the app ships (§16). The corpus cannot contain the defects the old one
  did.

### 1.5 Your first tweak: a complete, minimal, copyable file

This is a full, valid corpus file. Save it as `src-tauri/tweaks/my_first.yaml`, and the app compiles it.
It manages one registry value with two options.

```yaml
category:
  id: my_category
  name: "My Category"
  icon: "mdi:flask-outline"
  description: "A one-line description of this category."

tweaks:
  - id: example_registry_tristate
    name: "Example: Registry Tri-State"
    description: "A single registry value with an on state and a removed state."
    risk_level: low # low | medium | high | critical
    elevation: user # user | admin | system | ti: the privilege FLOOR
    reversible: true # declared AND build-checked against the computed value
    effects:
      - id: demo_flag
        registry: { key: 'HKCU\Software\MagicXToolboxExample\Registry', name: DemoFlag, type: REG_DWORD }
    options:
      - label: "Enabled"
        values:
          demo_flag: 1 # write DemoFlag = 1
      - label: "Disabled"
        values:
          demo_flag: absent # delete DemoFlag entirely (the reserved `absent` keyword)
```

Every line above is required except where §3 marks a field optional. Read the rest of this guide to
learn every other kind, keyword, and rule, but that file is a working tweak.

---

## 2. Anatomy of a corpus file

Each YAML file in `src-tauri/tweaks/` is one **corpus file** with exactly three top-level keys:

```yaml
category: # REQUIRED: exactly one, applies to every tweak in this file
  id: performance
  name: "Performance"
  icon: "mdi:speedometer"
  description: "Optimize system responsiveness."

shared: # OPTIONAL: corpus-level shared settings (§9). Omit if you have none.
  - id: ...

tweaks: # REQUIRED: the list of tweaks in this file
  - id: ...
```

- **Unknown keys anywhere are a build error** (`deny_unknown_fields`). A typo'd field name is never
  silently ignored: it fails the build naming the file and the bad key.
- **Multiple files are merged into one corpus.** `build.rs` loads every `*.yaml`/`*.yml` file in
  `tweaks/`, in sorted filename order, and merges them: all categories, all tweaks (each stamped with its
  own file's category), and all `shared:` blocks into one flat corpus-wide list.
- Because the merge is corpus-wide, **duplicate `shared:` ids and duplicate addresses are checked across
  _all_ files, not per file** (§9, §16).

### 2.1 The `category:` block

Category is declared **once per file** and applies to every tweak in that file. There is **no per-tweak
`category` field**: a tweak inherits its file's category.

| field         | required | meaning                                                     |
| ------------- | -------- | ----------------------------------------------------------- |
| `id`          | yes      | stable category id (kebab/snake case)                       |
| `name`        | yes      | display name                                                |
| `icon`        | yes      | icon name (e.g. `"mdi:speedometer"`), the frontend icon set |
| `description` | yes      | one-line category description                               |

All four are required strings. Put related tweaks in the same file to share a category; put a different
category's tweaks in a different file.

### 2.2 The `shared:` block

Optional. A list of corpus-level shared settings: the **only** legitimate way for two tweaks to touch
one address (§9). Omit the key entirely if the file declares none (it defaults to empty).

### 2.3 The `tweaks:` block

Required. A list of tweaks; each tweak's fields are §3.

### 2.4 One file's full shape (annotated)

```yaml
category:
  id: examples
  name: "Examples"
  icon: "mdi:flask-outline"
  description: "Demonstration corpus."

shared:
  - id: example_shared_dword
    registry: { key: 'HKCU\Software\MagicXToolboxExample\Shared', name: SharedFlag, type: REG_DWORD }
    value: 1 # THE shared value (§9)

tweaks:
  - id: example_registry_tristate
    name: "Example: Registry Tri-State"
    description: "…"
    risk_level: low
    elevation: user
    reversible: true
    effects:
      - id: demo_flag
        registry: { key: 'HKCU\Software\MagicXToolboxExample\Registry', name: DemoFlag, type: REG_DWORD }
    options:
      - label: "Enabled"
        values: { demo_flag: 1 }
      - label: "Disabled"
        values: { demo_flag: absent }
```

---

## 3. Tweak-level fields

Every entry in `tweaks:` is one tweak. Here is the **complete** field reference: every field the schema
accepts, whether it is required, its default, its legal values, and its gotchas.

| field             | required | type / legal values                       | default           | notes                                                           |
| ----------------- | -------- | ----------------------------------------- | ----------------- | --------------------------------------------------------------- |
| `id`              | **yes**  | string                                    | –                 | unique tweak id; used in snapshot keys and error messages       |
| `name`            | **yes**  | string                                    | –                 | display name                                                    |
| `description`     | **yes**  | string                                    | –                 | one-line description shown in the UI                            |
| `info`            | no       | string                                    | _(none)_          | optional longer explanation                                     |
| `warning`         | no       | string                                    | _(none)_          | optional caution banner shown in the UI                         |
| `risk_level`      | **yes**  | `low` \| `medium` \| `high` \| `critical` | –                 | advisory only; never changes behavior                           |
| `elevation`       | **yes**  | `user` \| `admin` \| `system` \| `ti`     | –                 | the privilege **floor** for the whole tweak (§13)               |
| `reversible`      | **yes**  | `true` \| `false`                         | –                 | declared **and** build-checked against the computed value (§14) |
| `requires_reboot` | no       | `true` \| `false`                         | `false`           | advisory: a reboot/logoff is needed for full effect             |
| `windows`         | no       | a `windows:` block (§10)                  | _(unconstrained)_ | scopes the whole tweak by Windows build                         |
| `effects`         | **yes**  | list of effects (§4)                      | –                 | the managed surface: at least one                               |
| `options`         | **yes**  | list of options (§7)                      | –                 | the authored states: at least one                               |

**Do / Don't callouts:**

- ✅ `risk_level`, `elevation`, `reversible` are **required**: you must always state them explicitly.
- ❌ There is **no `category` field on a tweak**: category is per file (§2.1).
- ❌ There is **no `is_default`, `is_toggle`, or option index**: the UI shape is computed from the
  option count, and "System Default" is a computed status (§1.3, §18).
- ⚠️ `reversible` is **not a free choice.** You declare it, but the build **computes** the true value and
  rejects a mismatch (`ReversibilityMismatch`, §14/§16). Declare what is actually true.
- ⚠️ `risk_level` is purely advisory. The build does **not** infer risk or reject a "too-low" rating:
  that rule is review guidance, not a build guard (spec §10).

### 3.1 Legal enum values (exhaustive)

These come straight from the compiled model; these are the _only_ accepted spellings:

- **`risk_level`** (advisory impact): `low`, `medium`, `high`, `critical`.
- **`elevation`** (privilege floor, §13): `user`, `admin`, `system`, `ti`.

Any other spelling is a build error (`unknown variant`). Case matters; these are lowercase.

---

## 4. Every effect kind

Each entry in a tweak's `effects:` list is one effect: an **`id`** plus **exactly one kind key**.

```yaml
effects:
  - id: some_id # your identifier for this effect, referenced from options
    registry:
      { … } # exactly ONE kind key: registry | registry_key | service |
      # task | hosts | firewall | shared | action
```

- The **`id`** is how options refer to this effect (in their `values:` map): it must be unique within
  the tweak.
- **Exactly one kind key** is required. Zero kind keys or two kind keys is a build error _by
  construction_: the schema models an effect as an untagged choice, so "no valid kind" is a plain
  deserialize failure. See the gotcha below.

**Common optional fields on any effect:**

| field        | applies to                   | meaning                                                                  |
| ------------ | ---------------------------- | ------------------------------------------------------------------------ |
| `elevation`  | all 8 kinds                  | per-effect escalation; effective level is `max(tweak floor, this)` (§13) |
| `windows`    | all 8 kinds                  | version scoping for this one effect (§10)                                |
| `optional`   | the **6 Setting kinds only** | this resource may legitimately not exist (§8)                            |
| `if_missing` | the **6 Setting kinds only** | value detection reads when the resource is Missing (§8)                  |

> ⚠️ **Gotcha: `optional`/`if_missing` on `shared` or `action` is a build error.** They are wired only
> on the six Setting kinds (`registry`, `registry_key`, `service`, `task`, `hosts`, `firewall`). Writing
> `optional:` or `if_missing:` on a `shared` or `action` effect is rejected as an unknown field.

> ⚠️ **Gotcha: a typo in the kind key gives an opaque error.** Because an effect is modelled as an untagged choice of kinds, misspelling the kind key (`regisrty:` instead of `registry:`) or supplying two kind keys makes _no_ kind match. You will see a YAML-level error like `data did not match any variant of untagged enum EffectRaw` (surfaced as the `Yaml` build error, §16). If you get that message, check that each effect has exactly one, correctly-spelled kind key.

The eight kinds divide into three families:

- **Settings** (declarative, reversible by construction, always detectable): `registry`, `registry_key`,
  `service`, `task`, `hosts`, `firewall`.
- **Shared** (a reference to a corpus-level shared setting): `shared`.
- **Action** (the imperative escape hatch): `action`.

---

### 4.1 `registry`: a registry value

Manages one **named registry value**.

```yaml
- id: demo_flag
  registry: { key: 'HKCU\Software\MagicXToolboxExample\Registry', name: DemoFlag, type: REG_DWORD }
```

| field    | required | meaning                                                                                  |
| -------- | -------- | ---------------------------------------------------------------------------------------- |
| `key`    | **yes**  | full merged path `HIVE\Sub\Key\...` (see below)                                          |
| `name`   | **yes**  | the value name                                                                           |
| `type`   | **yes**  | one of `REG_DWORD`, `REG_QWORD`, `REG_SZ`, `REG_EXPAND_SZ`, `REG_MULTI_SZ`, `REG_BINARY` |
| `field`  | no       | one field of a packed value (§11)                                                        |
| `format` | no       | packing format for `field`; v1: `kv_semicolon` (the default when `field` is set)         |

**The `key` path (spec §5.1):**

- Written as one merged string, `HIVE\Sub\Key\...`, exactly as regedit shows it. Use **plain or
  single-quoted YAML scalars** so backslashes are written **singly** (`'HKCU\Software\...'`). Do **not**
  double them.
- **v1 hives are HKLM and HKCU only**, short or long spelling, both normalized at build:
  `HKLM` = `HKEY_LOCAL_MACHINE`, `HKCU` = `HKEY_CURRENT_USER`. Any other hive (`HKCR`, `HKU`, `HKCC`) is
  a build error.
- These are build errors (the exact messages are in §16 under `InvalidAddress`):
  - a **leading** backslash (`\HKLM\...`),
  - a **trailing** backslash (`HKLM\...\`),
  - a **doubled** backslash / empty segment (`HKLM\A\\B`),
  - a **forward slash** (`HKLM/A/B`),
  - **no key path** (`HKLM` alone),
  - an **unsupported hive**.

**Value domain:** a typed literal in `type`'s terms, or the reserved keyword `absent` to delete the
value (§5, §6). `present` is **not** valid on a registry value (only presence kinds have it).

**Runtime behavior:**

- **read/detect:** returns the typed value; a missing key _or_ missing value both read as `Absent`; a
  value stored as a _different_ type than declared is a typed error (the tweak reads **Unknown**, never a
  fake absence); a malformed packed value is a typed error (**Unknown**).
- **apply/restore:** writes the value, **auto-creating any missing parent keys** (standard
  `RegCreateKeyEx`). Driving to `absent` deletes the value; deleting an already-absent value is an
  idempotent success, not a failure. Capture reads the pre-apply value (possibly `Absent`) so revert can
  put it back.

---

### 4.2 `registry_key`: registry key presence

Manages **whether a key exists**, with no value semantics.

```yaml
- id: feature_key
  registry_key: { key: 'HKCU\Software\ExampleCo\FeatureFlag' }
```

| field | required | meaning                                                  |
| ----- | -------- | -------------------------------------------------------- |
| `key` | **yes**  | full merged path `HIVE\Sub\Key\...` (same rules as §4.1) |

**Value domain:** `present` or `absent`.

**Runtime behavior:** pre-existence is **captured** like any value. `present` creates the key;
`absent` removes it, but on **revert the engine deletes the key only if it created it** (if the key
pre-existed, revert leaves it). This is why key presence is a Setting, not a `create_key`/`delete_key`
action (§18).

---

### 4.3 `service`: a Windows service's start type

Manages the **startup type** of a named service.

```yaml
- id: update_service
  service: { name: wuauserv }
```

| field  | required | meaning                                   |
| ------ | -------- | ----------------------------------------- |
| `name` | **yes**  | exact service name (not the display name) |

**Value domain: the six start types (spec §5.1):**

`boot`, `system`, `automatic`, `automatic_delayed`, `manual`, `disabled`.

**Runtime behavior:**

- **read/detect:** returns the current start type, or **`Missing`** if the service is not installed
  (§8). `automatic_delayed` is detected via the companion `DelayedAutostart` registry value beside the
  service's `Start` value: a merely-absent companion never fabricates delayed-start.
- **apply/restore:** sets the start type through the SCM and writes the `DelayedAutostart` companion to
  match (never left stale). Driving a service to `Missing` is a **defined no-op**. Driving a _real_ start
  type at a service that is **not installed** is a typed `ResourceMissing` error: **the engine never
  installs or uninstalls services** (spec §5.4). Declare `optional: true` for services that may be
  absent (§8).

> ❌ **Do not reach a service through its raw registry storage.** Writing
> `HKLM\SYSTEM\...\Services\<name>\Start` as a `registry` effect is a build error (`NonCanonicalKind`,
> §16). Use the `service` kind: that is how ownership stays single (§9).

> ❌ **You cannot disable `TrustedInstaller`** via a typed Service effect: it would strand the app's own
> TI elevation path (`TrustedInstallerDisabled`, §16).

---

### 4.4 `task`: a scheduled task's enabled state

Manages **whether a scheduled task is enabled**.

```yaml
- id: medic_task
  task: { path: '\Microsoft\Windows\WaaSMedic\PerformRemediation' }
```

| field  | required | meaning                                                        |
| ------ | -------- | -------------------------------------------------------------- |
| `path` | **yes**  | the **exact** task path, no patterns, no wildcards (spec §5.1) |

**Value domain:** `enabled` or `disabled`. (v1 task richness is enabled/disabled only, triggers and
actions are out of scope.)

**Runtime behavior:**

- **read/detect:** returns `enabled`/`disabled`, or **`Missing`** if the task does not exist (§8).
- **apply/restore:** enables/disables via the Task Scheduler COM service. Driving to `Missing` is a
  no-op; a real value at a missing task is a typed `ResourceMissing` error (never creates/deletes tasks).

> ❌ **No patterns.** The old `task_name_pattern` is gone (§18). Address the exact path; if a task may not
> exist on some machines, use `optional: true` (§8). Reaching a task through its registry storage
> (`…\Schedule\TaskCache\…`) is a build error (`NonCanonicalKind`, §16): use the `task` kind.

---

### 4.5 `hosts`: a hosts-file entry

Manages **whether an `ip domain` line exists in the hosts file**.

```yaml
- id: block_host
  hosts: { ip: "0.0.0.0", domain: "tracker.example.invalid" }
```

| field    | required | meaning                                               |
| -------- | -------- | ----------------------------------------------------- |
| `ip`     | **yes**  | the IP the line maps the domain to (e.g. `"0.0.0.0"`) |
| `domain` | **yes**  | the domain name                                       |

**Value domain:** `present` or `absent`.

**Runtime behavior:** `read` reports whether that exact `ip domain` line exists; `present` adds it,
`absent` removes it.

> 💡 Use RFC-2606 reserved domains (`.invalid`, `.example`) for demonstration/test tweaks so they can
> never affect real name resolution.

---

### 4.6 `firewall`: a firewall rule

Manages **whether a named firewall rule exists**, carrying the full definition needed to (re)create it.

```yaml
- id: block_rule
  firewall:
    name: "MagicX Toolbox Example Rule 5F3F1D2E"
    direction: outbound # inbound | outbound          (required)
    action: block # block | allow               (required)
    protocol: tcp # any | tcp | udp | icmpv4 | icmpv6   (optional)
    remote_addresses: ["203.0.113.0/24"] # optional list of strings
    description: "Example firewall rule." # optional
```

| field              | required | legal values / type                             | notes                                                |
| ------------------ | -------- | ----------------------------------------------- | ---------------------------------------------------- |
| `name`             | **yes**  | string                                          | the rule's display name, also its address (identity) |
| `direction`        | **yes**  | `inbound` \| `outbound`                         |                                                      |
| `action`           | **yes**  | `block` \| `allow`                              |                                                      |
| `protocol`         | no       | `any` \| `tcp` \| `udp` \| `icmpv4` \| `icmpv6` | `any` = unrestricted by protocol                     |
| `program`          | no       | string                                          | program path filter                                  |
| `service`          | no       | string                                          | service-name filter                                  |
| `remote_addresses` | no       | list of strings                                 | e.g. `["203.0.113.0/24"]`                            |
| `remote_ports`     | no       | string                                          | e.g. `"443"` or `"80,443"`                           |
| `local_ports`      | no       | string                                          |                                                      |
| `description`      | no       | string                                          |                                                      |

**Value domain:** `present` or `absent`.

**Runtime behavior:** the create/delete decision rides **entirely on the value**: `present` creates the
rule from the full definition; `absent` deletes the rule by name. Because the value alone carries the
decision, **`direction` and `action` are required**: whenever a `present` option drives the rule, the
address must already describe a complete, creatable rule. Restore recreates from the **authored
definition**, not from some prior captured rule state: that fidelity limit is by design.

> ⚠️ `firewall` cannot be a **shared** setting (§9): the `shared:` block has no firewall variant. If two
> tweaks need the same rule, that is a signal they are one tweak.

---

### 4.7 `shared`: a reference to a corpus-level shared setting

References a corpus-level shared setting by id. This is the **only** way two tweaks may touch one
address (§9).

```yaml
- id: shared_ref
  shared: example_shared_dword # the id of a `shared:` block entry
```

| field    | required | meaning                                       |
| -------- | -------- | --------------------------------------------- |
| `shared` | **yes**  | the id of a corpus-level `shared:` entry (§9) |

**Value domain:** `claim` or `unclaimed` (always explicit in every option; omission is a build error,
`SharedNotExplicit`, §16).

**Runtime behavior:** its lifecycle is the corpus-wide **claims record**, not the per-tweak snapshot
(§9). It may **not** carry `optional`/`if_missing`. It **may** carry `elevation`/`windows`.

---

### 4.8 `action`: the imperative escape hatch

Runs a `cmd`/`powershell` script for changes that cannot be expressed as a declarative Setting. Full
contract in §12.

```yaml
- id: flush_dns
  action:
    apply: "ipconfig /flushdns"
    ephemeral: true
    shell: cmd
```

| field       | required | meaning                                                                     |
| ----------- | -------- | --------------------------------------------------------------------------- |
| `apply`     | **yes**  | the script body to run on apply                                             |
| `undo`      | no       | the script body to reverse it (absent ⇒ one-way)                            |
| `probe`     | no       | the script body that reports present/absent (absent ⇒ not detectable)       |
| `ephemeral` | no       | `true` = a transient side-effect; takes no `undo`/`probe` (default `false`) |
| `shell`     | **yes**  | `cmd` \| `powershell`                                                       |

**Value domain:** `run`, or **omit the entry entirely** (omitted = "this option does not run it").

> ⚠️ **Scripts are inline strings only in v1.** `apply`, `undo`, and `probe` are plain string bodies
> (usually a YAML block scalar with `|`). There is **no `apply: { file: … }` filed-script form** in the
> shipped schema: writing one is a build error. See §12.6.

---

## 5. Value literals: the complete grammar

An option's `values:` map keys an **effect id** to a **value literal**. What is legal depends entirely
on the effect's kind. This section documents **every** literal form (spec §6.2).

### 5.1 Registry value literals

| type            | how to write it                                                    | examples                           |
| --------------- | ------------------------------------------------------------------ | ---------------------------------- |
| `REG_DWORD`     | decimal, or `0x`-prefixed hex (bare or quoted)                     | `1`, `0`, `0x2`, `"0xFF"`          |
| `REG_QWORD`     | decimal, or `0x` hex                                               | `5000000000`, `0x100000000`        |
| `REG_SZ`        | a plain string                                                     | `"Some text"`, `""` (empty string) |
| `REG_EXPAND_SZ` | a plain string (with `%VARS%`)                                     | `"%SystemRoot%\\system32"`         |
| `REG_BINARY`    | `.reg` hex-pair form: byte pairs separated by commas **or** spaces | `"de,ad,be,ef"`, `"de ad be ef"`   |
| `REG_MULTI_SZ`  | a real YAML string **list**; `[]` clears it                        | `["a", "b"]`, `[]`                 |

**Details and edge cases:**

- **`REG_DWORD`/`REG_QWORD`** accept decimal or `0x` hex. You may write hex bare (`0x2`, YAML parses it
  as an integer) or quoted (`"0x2"`, parsed as a string then converted); both compile to the same
  value. A `REG_DWORD` value that does not fit in 32 bits (`u32`) is a build error; use `REG_QWORD` for
  larger.
  - ⚠️ **Very large `REG_QWORD`s** (above `9223372036854775807`, i.e. `i64::MAX`) must be **quoted as a
    string** (`"0xFFFFFFFFFFFFFFFF"` or `"18446744073709551615"`), because a bare YAML integer that big
    cannot be represented. Quoted, it goes through the string→u64 path and works.
- **`REG_SZ`/`REG_EXPAND_SZ`** are verbatim strings. `""` is a **legitimate empty string** and keeps
  working; it is _not_ a delete (§6). A backslash inside a double-quoted YAML string must be escaped
  (`"C:\\Windows"`); prefer single quotes (`'C:\Windows'`) to avoid escaping.
- **`REG_BINARY`** is the `.reg` hex-pair form. Each token is **exactly two hex digits**; separate tokens
  by commas or spaces (not both styles at once). An odd-length token, a non-hex token, or a lone digit is
  a build error (it surfaces as `InvalidOptionValue`, §16 #4). Example: `"90,12,03,80"` = 4 bytes.
- **`REG_MULTI_SZ`** is written as a **YAML list of strings**, never a comma-joined string. `[]` clears
  it (deliberately better than `.reg`'s unwritable `hex(7):`). A non-list value on a `REG_MULTI_SZ`
  address is a build error (`WrongLiteralShape`: "a string is not valid for MultiSz"); a list value on a
  non-`REG_MULTI_SZ` address is likewise an error ("a list is not valid for Dword", etc.).

```yaml
values:
  a_dword: 1
  a_dword_hex: 0x2
  a_qword: 5000000000
  a_string: "Hello"
  an_empty_string: "" # legitimate empty REG_SZ, NOT a delete
  an_expand: '%SystemRoot%\explorer.exe'
  a_binary: "de,ad,be,ef"
  a_multi_sz: ["alpha", "beta"]
  cleared_multi_sz: [] # clears a REG_MULTI_SZ
  a_removed_value: absent # deletes the value (see §6)
```

### 5.2 Kind-specific keywords (the complete table)

| kind               | legal option values                                                                |
| ------------------ | ---------------------------------------------------------------------------------- |
| `registry` (value) | a typed literal (§5.1) **or** `absent`                                             |
| `registry_key`     | `present` \| `absent`                                                              |
| `service`          | `boot` \| `system` \| `automatic` \| `automatic_delayed` \| `manual` \| `disabled` |
| `task`             | `enabled` \| `disabled`                                                            |
| `hosts`            | `present` \| `absent`                                                              |
| `firewall`         | `present` \| `absent`                                                              |
| `shared`           | `claim` \| `unclaimed`                                                             |
| `action`           | `run` (or omit the entry)                                                          |

Using the wrong keyword for a kind is a build error (`InvalidOptionValue`, §16), e.g. a service value
that is not one of the six start types, a task value that is not `enabled`/`disabled`, a shared value
that is not `claim`/`unclaimed`, or an action value that is not `run`.

### 5.3 The `.reg` alignment (and where it stops)

Registry **type names** and the binary/`REG_MULTI_SZ` shapes are deliberately `.reg`-aligned so authors
who know `.reg` files feel at home. But **there is no `.reg` import and no `.reg` delete spellings**
(`-"Value"=-`). The YAML schema is the single source of truth. Deletion is the `absent` keyword (§6);
key presence is the `registry_key` Setting (§4.2).

### 5.4 The `{ literal: ... }` escape

The only value form that is a **map** (other than a per-option-value windows scope, §10) is the escape
for a string whose content is literally a reserved word:

```yaml
values:
  some_string: { literal: absent } # a REG_SZ whose CONTENT is the four letters "absent"
```

Without the escape, a bare `absent` is always the keyword. The escape exists purely to reach the
"ordinary string" path (§6.3). It only makes sense on `REG_SZ`/`REG_EXPAND_SZ` targets; on a
presence kind it is a build error (there is no string content there to render).

---

## 6. The `absent` keyword: deep dive

`absent` is the **one reserved word for "does not exist."** Because it is the _only_ way to spell
deletion, it must work everywhere: this section covers exactly how (spec §5.1/§6.2, ADR-0004).

### 6.1 `absent` works at value, key, and field depth, for every type

A **bare** `absent` is _always_ the keyword, never string content, at all three depths and for
**every** value type, including `REG_SZ`/`REG_EXPAND_SZ`:

```yaml
values:
  au_options: absent # registry VALUE: deletes the value
  feature_key: absent # registry_key: removes the key (if we created it)
  block_host: absent # hosts entry: removes the line
  block_rule: absent # firewall rule: deletes the rule
  packed_flag: absent # packed FIELD: removes just that field (§11)
```

It compiles to two typed outcomes depending on the kind (one spelling, one comparison per kind):

- On a **registry value or packed field** → `Value::Absent` (delete the value / remove the field).
- On a **presence kind** (`registry_key`, `hosts`, `firewall`) → `Value::Present(false)` (remove the
  resource).

> **Why "for every type, including `REG_SZ`"?** A type class with no deletion spelling would be a _hole_,
> not a safeguard (spec invariant 13). If `absent` were treated as string content on `REG_SZ`, you could
> never delete a `REG_SZ` value declaratively. So a bare `absent` is the keyword even there.

### 6.2 `present`: the positive mirror (presence kinds only)

`present` is `absent`'s positive twin, legal **only** on the three presence kinds (`registry_key`,
`hosts`, `firewall`). On a **registry value** it is a build error, because a registry value has no
"present" state, only a value or its absence:

```
`present` is not valid for a registry value; only registry keys, hosts entries, and firewall rules have a present/absent state
```

### 6.3 The `{ literal: absent }` escape

To store a **string whose content is the word `absent`**, use the escape (§5.4):

```yaml
values:
  some_string: { literal: absent } # REG_SZ content = "absent", NOT a delete
```

This is the escape's _entire_ purpose: it routes the text through the ordinary-string path instead of
the keyword path. The same applies to any reserved word you want as literal string content
(`{ literal: present }`, `{ literal: run }`, …).

### 6.4 `null`, omitted, or bare is a BUILD ERROR: never a silent delete

A `null`, an omitted value, or a bare `key:` with nothing after it is a **build error naming `absent`**:

```
value is null or empty: write `absent` to delete it, or supply a literal
```

**Why (ADR-0004):** YAML maps `value: null`, an omitted value, and a bare `some_effect:` all to the same
"absent" node. A _forgotten_ value must never silently become a delete. So the build rejects the empty
case and tells you the alternative: if you genuinely mean "delete", type `absent` deliberately.

```yaml
# ❌ WRONG: a forgotten value; build error naming `absent`
values:
  demo_flag: # nothing after the colon → BUILD ERROR

# ❌ WRONG: explicit null; same build error
values:
  demo_flag: null

# ✅ RIGHT: deliberate deletion
values:
  demo_flag: absent

# ✅ RIGHT: a real value
values:
  demo_flag: 1

# ✅ RIGHT: a legitimate empty string (NOT a delete)
values:
  some_string: ""
```

> Note: coverage (§7) _also_ independently rejects an omitted effect: every option must value every
> Setting effect. So a forgotten value is caught twice: as a null/empty literal and (if the whole key is
> missing) as missing coverage.

---

## 7. Options & the coverage rule

An option is a **`label`** plus a **`values`** map keyed by effect id:

```yaml
options:
  - label: "Fully Disabled"
    values:
      no_auto_update: 1
      au_options: absent
      update_service: disabled
      medic_task: disabled
```

| field    | required | meaning                                                    |
| -------- | -------- | ---------------------------------------------------------- |
| `label`  | **yes**  | the option's display name (also its identity in snapshots) |
| `values` | **yes**  | a map of `effect_id: <value literal>`                      |

### 7.1 The coverage rule: every option values every effect

**Every option must supply a value for every _Setting_ effect on the surface.** A hole is a build error
(`MissingCoverage`, §16):

```
tweak `T` option `O` does not cover effect `E`: every option must supply a value for every Setting effect on the surface
```

The three families cover differently:

| effect family                                                                    | coverage requirement                                                                               |
| -------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| **Setting** (`registry`, `registry_key`, `service`, `task`, `hosts`, `firewall`) | **must** be valued in every option                                                                 |
| **`shared`**                                                                     | **must** be explicit `claim` or `unclaimed` in every option (omission is `SharedNotExplicit`, §16) |
| **`action`**                                                                     | `run` **or omitted** (omitted = "this option does not run it")                                     |

**Why:** coverage is what makes stranded state impossible (§1.1). If an option could omit a Setting,
flipping to it would leave that address at whatever the previous option set: a state no option
describes.

### 7.2 The shape rule: toggle vs dropdown

The UI shape follows the option count (spec §6.1, ADR-0003):

- **1 authored option → a toggle** (Default ↔ On).
- **2 or more → a dropdown** (Default / A / B / …).

You never author "System Default": it is the computed status when the live surface matches no option
(§1.3, §15). So a **1-option** tweak is a switch between "the one state you defined" and "whatever the
machine was" (Default).

### 7.3 Per-option-value Windows scoping

An individual value may itself be scoped to certain Windows builds using the two-key map form
`{ value: <literal>, windows: {...} }`. This is the third scoping level; see §10.4. It works for
Setting, `shared`, and `action` values alike.

### 7.4 Worked multi-option example

```yaml
effects:
  - id: no_auto_update
    registry: { key: 'HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU', name: NoAutoUpdate, type: REG_DWORD }
  - id: au_options
    registry: { key: 'HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU', name: AUOptions, type: REG_DWORD }
  - id: update_service
    service: { name: wuauserv }

options:
  - label: "Fully Disabled"
    values:
      no_auto_update: 1
      au_options: absent # tri-state policy: remove the value
      update_service: disabled
  - label: "Notify Only"
    values:
      no_auto_update: 1
      au_options: 0x2 # decimal or 0x hex, as regedit shows it
      update_service: manual
```

Both options value all three effects; they differ on detectable Settings; the build accepts this.

---

## 8. Presence: `optional` / `if_missing`

A service or scheduled task (or any Setting) may legitimately **not exist** on a given machine. That is
the typed **`Missing`** state, distinct from `absent` (a state you can drive to) and from a read error
(spec §5.4).

### 8.1 The three related states: don't confuse them

| state          | meaning                                                                               | who produces it                                            |
| -------------- | ------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| **`absent`**   | a drivable "does not exist"; you can apply it and revert from it                      | authored value / capture of a nonexistent value            |
| **`Missing`**  | the _resource_ does not exist on this machine (service not installed, task not found) | **capture-only**: never authored, driving to it is a no-op |
| **read error** | the state could not be read (access denied, malformed)                                | typed `Err` → status **Unknown**                           |

Key rules:

- **`Missing` is capture-only.** No option can author it. **Driving to `Missing` is a defined no-op**:
  the engine **never installs or uninstalls** services/tasks/resources.
- A **non-optional** effect that reads `Missing` is a **typed error** → the tweak reads **Unknown**. That
  is deliberate: if you did not say a resource is allowed to be absent, its absence is a surprise worth
  surfacing.

### 8.2 `optional: true`

Declare `optional: true` on an effect that may legitimately be absent on some machines:

```yaml
- id: demo_service
  service: { name: RemoteRegistry }
  optional: true
```

Now a `Missing` capture is legal for this effect instead of an error.

### 8.3 `if_missing: <value>`

Optionally add `if_missing:` meaning "on a machine where this resource is absent, **detection treats this
effect as reading `<value>`**":

```yaml
- id: demo_service
  service: { name: RemoteRegistry }
  optional: true
  if_missing: disabled # a machine without this service counts as `disabled` for detection
```

- The `if_missing` value is parsed against the **effect's own domain** (a service → a start type; a
  presence kind → `present`/`absent`; a registry value → a literal or `absent`). An `if_missing` that
  does not parse in that domain is a build error (`InvalidIfMissing`, §16).
- **`if_missing` requires `optional: true`.** Declaring it on a non-optional effect is a build error
  (`IfMissingWithoutOptional`, §16): a non-optional effect never reads `Missing` (it errors instead), so
  `if_missing` there is dead authoring.

### 8.4 Unavailable-on-this-machine

If an option's desired value for a `Missing` resource **differs** from that effect's `if_missing`
meaning (e.g. an option that _enables_ a service that is not installed), that option is shown
**unavailable on this machine** at detect time, and apply is never offered for it. (If a resource
vanishes between detect and apply, the apply fails typed and rolls back: never a silent skip.)

### 8.5 Keep every option detectable _without_ optional effects

Because optional effects may be `Missing` on a real machine, **every option must stay distinguishable
without them.** The build enforces this: each option needs **at least one non-optional detectable
effect** on every supported build (`NotDetectable`, §16). Pair optional effects with a non-optional one:

```yaml
effects:
  - id: demo_marker # non-optional: keeps both options detectable everywhere
    registry: { key: 'HKCU\Software\MagicXToolboxExample\ServiceTask', name: Marker, type: REG_DWORD }
  - id: demo_service
    service: { name: RemoteRegistry }
    optional: true
    if_missing: disabled
  - id: demo_task
    task: { path: '\Microsoft\Windows\DiskCleanup\SilentCleanup' }
    optional: true
    if_missing: disabled
options:
  - label: "Enabled"
    values: { demo_marker: 1, demo_service: manual, demo_task: enabled }
  - label: "Disabled"
    values: { demo_marker: 0, demo_service: disabled, demo_task: disabled }
```

The `demo_marker` registry value is non-optional, so even on a machine lacking the service or task, each
option is still told apart by the marker.

### 8.6 When to use `optional`

Use it for resources that **genuinely vary by Windows edition/SKU or servicing state**: a service or
task present on some builds and absent on others. Do **not** use it to paper over a wrong service name or
a task that should always exist. And never rely on an optional effect as an option's _only_
distinguisher (§8.5).

---

## 9. Shared settings: declared, refcounted

Two tweaks must never both own one address. If tweak A and tweak B both write one registry value, then
A's revert changes the value out from under B: B's option is genuinely no longer in effect, detection
honestly shows B at System Default, and the user watches a tweak flip itself off (ADR-0006). This
happened in practice with the old corpus.

So the **ownership guard** (§16) enforces **one address, one owner, corpus-wide.** The _only_ sanctioned
way for two tweaks to touch one address is a corpus-level `shared:` block.

### 9.1 Declaring a shared setting

A `shared:` entry declares the **address and the single target value** both:

```yaml
shared:
  - id: telemetry_off
    registry: { key: 'HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection', name: AllowTelemetry, type: REG_DWORD }
    value: 0 # THE shared value: declared HERE, never per-option
```

| field          | required | meaning                                                                                         |
| -------------- | -------- | ----------------------------------------------------------------------------------------------- |
| `id`           | **yes**  | unique shared id, corpus-wide (duplicate ⇒ `DuplicateSharedId`, §16)                            |
| _one kind key_ | **yes**  | `registry`, `registry_key`, `service`, `task`, or `hosts`, **not** `firewall`, **not** `action` |
| `value`        | **yes**  | the single target value all claiming tweaks agree on, in the kind's domain                      |

Because the shared block declares the **single value**, two claiming tweaks **cannot disagree by
construction.** Two tweaks wanting _different_ values on one address is impossible to express: you would
have to declare two shared entries on one address, which is a duplicate-address build error naming both.

### 9.2 Referencing it from tweaks

A tweak references a shared setting as a `shared` effect, and every option says `claim` or `unclaimed`:

```yaml
effects:
  - id: telemetry
    shared: telemetry_off
options:
  - label: "Disabled"
    values: { telemetry: claim }
  - label: "Off"
    values: { telemetry: unclaimed }
```

`claim`/`unclaimed` must be **explicit** in every option: omission is a build error
(`SharedNotExplicit`, §16), so sharing is always a visible decision.

### 9.3 The claim/release lifecycle (runtime)

There is one engine-level, machine-stamped, atomically-written **claims record** in the snapshots
directory: `{ shared_id → { original: Value, claimants: [tweak_id] } }` (spec §8.6, ADR-0006):

- **First claim** (corpus-wide): capture the live original value **once**, drive to the declared shared
  value, record the claimant.
- **Further claims:** verified no-op; the claimant is added.
- **Release** (any transition whose target does not claim): the claimant is removed; while other
  claimants remain, the value is left alone and the releasing tweak reports _"held by \<tweaks\>"_ as
  info, not failure.
- **Last release:** drive the value back to the captured original, verify, and release the record: a
  verified restore (ADR-0002).

Shared-referenced effects appear in **no per-tweak snapshot**: their return path is exclusively the
claims record, so two tweaks' snapshots can never fight over one address.

### 9.4 The distinctness rule: a shared claim can't be the sole distinguisher

Because a claimed shared value can be held by **another** tweak too, it can never be the **only** thing
that distinguishes two of a tweak's own options. Two options that differ _only_ by a shared claim are a
build error (`SharedOnlyDistinguisher`, §16). **Always pair a shared effect with a non-shared one:**

```yaml
effects:
  - id: demo_marker # non-shared distinguisher
    registry: { key: 'HKCU\Software\MagicXToolboxExample\SharedA', name: Marker, type: REG_DWORD }
  - id: shared_ref
    shared: example_shared_dword
options:
  - label: "On"
    values: { demo_marker: 1, shared_ref: claim }
  - label: "Off"
    values: { demo_marker: 0, shared_ref: unclaimed }
```

Also note: an option whose _only_ effect is an `unclaimed` shared reference has **zero** detectable
signal (unclaimed asserts nothing), so it fails detectability (`NotDetectable`, §16): another reason to
pair shared with a real Setting.

### 9.5 Full two-tweak shared example

Two independent tweaks claiming one corpus-level shared setting, each with its own non-shared marker
(from `examples.yaml`):

```yaml
shared:
  - id: example_shared_dword
    registry: { key: 'HKCU\Software\MagicXToolboxExample\Shared', name: SharedFlag, type: REG_DWORD }
    value: 1

tweaks:
  - id: example_shared_claim_a
    name: "Example: Shared Claim A"
    description: "Claims example_shared_dword alongside B."
    risk_level: low
    elevation: user
    reversible: true
    effects:
      - id: demo_marker
        registry: { key: 'HKCU\Software\MagicXToolboxExample\SharedA', name: Marker, type: REG_DWORD }
      - id: shared_ref
        shared: example_shared_dword
    options:
      - label: "On"
        values: { demo_marker: 1, shared_ref: claim }
      - label: "Off"
        values: { demo_marker: 0, shared_ref: unclaimed }

  - id: example_shared_claim_b
    name: "Example: Shared Claim B"
    description: "Claims the same setting; reverting one leaves the other's claim until the last release."
    risk_level: low
    elevation: user
    reversible: true
    effects:
      - id: demo_marker
        registry: { key: 'HKCU\Software\MagicXToolboxExample\SharedB', name: Marker, type: REG_DWORD }
      - id: shared_ref
        shared: example_shared_dword
    options:
      - label: "On"
        values: { demo_marker: 1, shared_ref: claim }
      - label: "Off"
        values: { demo_marker: 0, shared_ref: unclaimed }
```

---

## 10. Windows version scoping

`windows:` scopes applicability by Windows build. Any field is optional; omitted = unconstrained; the
axes **AND** together (spec §6.6).

```yaml
windows:
  products: [10, 11] # set membership; 10 and 11 are the ONLY products
  build: ">=26100" # N | >=N | <=N | A..B  (inclusive), against the major build
  revision: ">=2314" # same grammar, against the UBR, only legal with a single pinned build
```

### 10.1 The `products` axis

`products` is a **set** of product ids; the machine matches if it is in **any** listed product's build
range (spec §6.6):

- `10` = builds `10240..19045` (Windows 10).
- `11` = builds `>=22000` (Windows 11).

Only `10` and `11` are valid product ids; any other is a build error (`{n} is not a supported windows
product: use 10 or 11`). `products: [10, 11]` means "Windows 10 or 11": effectively all supported
builds.

### 10.2 The `build` axis: the expression grammar

`build` is one expression against the **major build number** (spec §6.6, exact grammar from the parser):

| form   | meaning                           | example                 |
| ------ | --------------------------------- | ----------------------- |
| `N`    | exactly build N                   | `build: 26100`          |
| `>=N`  | build N or newer                  | `build: ">=26100"`      |
| `<=N`  | build N or older                  | `build: "<=19045"`      |
| `A..B` | builds A through B, **inclusive** | `build: "22621..26100"` |

Any other string is a build error (`{raw} is not a valid windows build expression: use N, >=N, <=N, or
A..B`). Quote expressions containing `>`/`<` so YAML does not misparse them.

### 10.3 The `revision` axis: only with a single pinned build

`revision` uses the **same grammar** as `build`, matched against the **UBR** (the part after the dot in
`10.0.26100.2314`). But **`revision` is only legal when `build` pins a single exact build**, because the
revision/UBR counter resets per build line, so a revision only means something within one build:

```yaml
windows: { build: 26100, revision: ">=2314" } # ✅ build is a single exact value
```

```yaml
windows: { build: ">=26100", revision: ">=2314" } # ❌ build is a range → RevisionWithoutExactBuild
windows: { revision: ">=2314" } # ❌ no build at all → RevisionWithoutExactBuild
```

The error:

```
`revision` requires `build` to pin a single exact build: add an exact `build` or drop `revision`
```

### 10.4 The three scoping levels

`windows:` is legal at three levels: **tweak**, **effect**, and **per-option-value**:

**Tweak level**: scopes the whole tweak. If the tweak's scope excludes the running build, the tweak
shows **unavailable, with the reason**:

```yaml
- id: example_windows_scoped
  windows: { build: ">=26100" } # whole tweak is 24H2+
  effects: […]
  options: […]
```

**Effect level**: excludes one effect on out-of-scope builds; the rest of the tweak still applies:

```yaml
effects:
  - id: modern_only_setting
    registry: { … }
    windows: { build: ">=22621" } # this effect only exists on 22621+
```

**Per-option-value level**: a single value inside an option's `values:` map, written as the two-key map
`{ value: <literal>, windows: {...} }`. This is the **only** value form that is a map with a `value`
key: it cannot collide with the `{ literal: … }` escape (whose only key is `literal`) or with bare
keywords:

```yaml
options:
  - label: "On"
    values:
      some_effect: { value: 1, windows: { build: ">=26100" } }
```

Per-option-value scoping applies to **Setting, Shared, and Action** values alike. The `value:` key holds
whatever that effect's option value normally is (a literal for a Setting, or the bare keyword `run` /
`claim` / `unclaimed` for an Action / Shared effect) and `windows:` scopes it:

```yaml
values:
  some_effect: { value: 1, windows: { build: ">=26100" } } # ✅ scoped Setting value
  notify_action: { value: run, windows: { build: ">=26100" } } # ✅ run this action only on 26100+
  shared_ref: { value: claim, windows: { build: ">=26100" } } # ✅ scoped claim
```

A value that this per-option-value scope excludes on a given build simply has **no answer** for that
effect there: the option behaves as if it did not cover the effect on that build (which is why an
always-in-scope companion effect matters for detectability: see §10.6 and §17.5).

### 10.5 What "excluded on this build" means

A scoped-out effect is **excluded entirely** (not applied, not read, not counted toward detection) on
builds its scope excludes. A tweak whose **entire applicable surface is empty** on the running build is
shown **unavailable, with the reason**; it is _not_ an error, just genuinely inapplicable there.

The runtime reads the build via `RtlGetVersion` (never `GetVersionEx`) and the revision via the `UBR`
registry value.

### 10.6 The support matrix: how the build guards quantify

The build-time guards do **not** just check "does this parse". They **quantify over a fixed support
matrix** of Windows builds (spec §10/§14):

```
19045   (Windows 10 22H2)
22621   (Windows 11 21H2/22H2)
22631   (Windows 11 22H2/23H2)
26100   (Windows 11 24H2)
```

For **every** milestone, the validator computes each tweak's **applicable projection** (the effects and
values in scope on that build) and runs the detectability and distinctness guards over it. This is why an
option can pass on one build and fail on another:

- If a per-option-value scope removes an option's only detectable value on build 22621, the option is
  `NotDetectable` **on build 22621** even though it is fine on 26100.
- The error message names the **first milestone** the failure was observed on. Fix the option, not each
  build: each violation is reported once, deduped across the matrix.

> ⚠️ **Design implication:** whenever you scope a value or effect, mentally walk all four milestones and
> confirm each option still has a non-optional, detectable, non-shared value there. §17.5 works a scoped
> example through the matrix.

The guards are build-only (they ignore `revision`); `revision` already requires a pinned build, so it
adds nothing at milestone granularity.

---

## 11. Packed / field-addressed values

Some registry values pack several independent knobs into one string, e.g.
`DirectXUserGlobalSettings = "SwapEffectUpgradeEnable=1;VRROptimizeEnable=0;"`. A `registry` effect may
address **one field** of such a value with `field` + `format` (spec §5.2):

```yaml
- id: packed_flag
  registry:
    {
      key: 'HKCU\Software\Microsoft\DirectX\UserGpuPreferences',
      name: DirectXUserGlobalSettings,
      type: REG_SZ,
      field: SwapEffectUpgradeEnable,
      format: kv_semicolon,
    }
```

- `field`: the name of the sub-field this effect manages.
- `format`: how the value is packed. **v1 ships exactly one format: `kv_semicolon`** (`Name=Value;`
  pairs). Omitting `format` when `field` is set **defaults to `kv_semicolon`**.

### 11.1 The `kv_semicolon` format

The live string is a series of `Name=Value;` segments: one per `;`, a single trailing `;` is the normal
terminator. A segment that is not exactly one non-empty name + one value (a stray `;;`, a missing `=`, a
doubled `=`) makes the **whole** parse fail: never a partial or guessed reading.

### 11.2 Runtime behavior: upsert, order preserved

- **apply/restore:** read the live string, parse it into fields, **upsert only the addressed field**, and
  re-serialize: **preserving every other field and their original order**. A field write is a
  read-modify-write cycle, serialized process-wide behind the registry kind's mutex so two tweaks writing
  different fields of the _same_ value never race.
- **`absent` on a field** removes just that field, leaving the others untouched.
- A live string the parser **cannot understand is a typed read error** → the tweak reads **Unknown**,
  never a guess and never a destructive rewrite.

### 11.3 Ownership: whole-value XOR field-addressed

A packed value is **whole-owned XOR field-addressed**: never both, and each field is owned once
(ADR-0006). The ownership guard groups by `(hive, path, name)` **ignoring** the field:

- ✅ Two effects addressing **different fields** of one value → fine (`SwapEffectUpgradeEnable` and
  `VRROptimizeEnable`).
- ❌ One effect addressing the **whole value** and another addressing a **field** of it → build error
  (`DuplicateAddress`, mentioning the mix).
- ❌ Two effects addressing the **same field** → build error.

### 11.4 Option values for a field

Option values for a packed field are plain literals in the **field's own terms**, or `absent` to remove
the field:

```yaml
options:
  - label: "Enabled"
    values: { packed_flag: "1" }
  - label: "Disabled"
    values: { packed_flag: "0" }
  - label: "Default"
    values: { packed_flag: absent } # remove just this field
```

### 11.5 Full packed example (from `examples.yaml`)

```yaml
- id: example_packed_field
  name: "Example: Packed Registry Field"
  description: "Field-addressed writes into one packed REG_SZ value, preserving other fields."
  risk_level: low
  elevation: user
  reversible: true
  effects:
    - id: packed_flag
      registry:
        {
          key: 'HKCU\Software\MagicXToolboxExample\Packed',
          name: DemoPacked,
          type: REG_SZ,
          field: DemoFlag,
          format: kv_semicolon,
        }
  options:
    - label: "Enabled"
      values: { packed_flag: "1" }
    - label: "Disabled"
      values: { packed_flag: "0" }
```

### 11.6 Gotcha: field addressing requires `REG_SZ` or `REG_EXPAND_SZ`

> ⚠️ **Warning:** a `field`-addressed effect's `type` **must** be `REG_SZ` or `REG_EXPAND_SZ`. The runtime (`kinds/registry.rs`) rejects every other type on **both** the read path (detect) and the write path (apply/restore) with `Error::Invalid("a packed field address must declare REG_SZ or REG_EXPAND_SZ")`. **This is a runtime check, not a build guard:** a tweak declaring `type: REG_DWORD, field: X, format: kv_semicolon` (or any non-string type paired with `field`) **builds clean** and only fails the first time it is detected or applied. Always pair `field`/`format` with `type: REG_SZ` or `type: REG_EXPAND_SZ`.

---

## 12. The Action contract

Actions are the imperative escape hatch for free-form scripts that cannot be expressed as a declarative
Setting (spec §7). Reach for a Setting first; an Action is a last resort.

```yaml
- id: marker_action
  action:
    apply: |
      New-Item -Path 'HKCU:\Software\MagicXToolboxExample\Action' -Force | Out-Null
      New-ItemProperty -Path 'HKCU:\Software\MagicXToolboxExample\Action' -Name ScriptMarker -Value 1 -PropertyType DWord -Force | Out-Null
    undo: |
      Remove-ItemProperty -Path 'HKCU:\Software\MagicXToolboxExample\Action' -Name ScriptMarker -Force -ErrorAction SilentlyContinue
    probe: |
      if (Get-ItemProperty -Path 'HKCU:\Software\MagicXToolboxExample\Action' -Name ScriptMarker -ErrorAction SilentlyContinue) { exit 0 } else { exit 1 }
    shell: powershell
```

### 12.1 The four fields: required and optional, all independent

| field       | required | meaning                                                                                           |
| ----------- | -------- | ------------------------------------------------------------------------------------------------- |
| `apply`     | **yes**  | the script that performs the change                                                               |
| `undo`      | no       | the script that reverses it; **absent ⇒ this action is one-way**                                  |
| `probe`     | no       | the script that reports present/absent; **absent ⇒ this action does not contribute to detection** |
| `ephemeral` | no       | `true` = a transient side-effect (§12.3); default `false`                                         |
| `shell`     | **yes**  | `cmd` or `powershell`                                                                             |

`undo`, `probe`, and `ephemeral` are **independent**: an action can carry any combination (subject to
the ephemeral rule, §12.3).

### 12.2 The exit-code result contract

Results are the **exit code**, locale-independent, never parsed text (spec §7/§14):

- `apply` / `undo`: **`0` = success**, non-zero = failure (a typed `ActionFailed(code)` error).
- `probe`: **`0` = present**, non-zero = absent. A probe that **cannot be run** (spawn failure, timeout)
  is `Err`: "we could not tell" must **never** read as "absent".

Write your scripts to `exit 0` / `exit 1` explicitly for the state they report.

### 12.3 `ephemeral: true`: transient side-effects

`ephemeral: true` marks an action whose effect is **transient** and changes no persistent state: flush
DNS, restart Explorer, `gpupdate /force`:

```yaml
- id: flush_dns
  action:
    apply: "ipconfig /flushdns"
    ephemeral: true
    shell: cmd
```

- It runs on apply and takes **no `undo` and no `probe`.** Declaring either alongside `ephemeral: true`
  is a build error (`EphemeralWithUndoOrProbe`, §16).
- It is **exempt** from the reversibility and detectability computations: a transient side-effect leaves
  no persistent state to detect and cannot be "reverted". An ephemeral action **never makes a tweak
  one-way** (§14).

### 12.4 The deciding question: does it need `undo`?

> **Can this action persistently alter or break the system?**
>
> - **Yes** → it needs `undo` (or the tweak is honestly `reversible: false`, §14).
> - **No** (it is transient) → mark it `ephemeral: true`.

Never leave a persistent, breaking change with no `undo` while claiming `reversible: true`: the build
computes the truth and rejects the lie (`ReversibilityMismatch`, §14/§16).

### 12.5 `probe`: state-based, cached, present/absent

`probe` answers _"is the state this action produces currently present?"_: **state-based, never
history-based.** The **same** probe is the apply-time did-it-work check and the detect-time
present/absent contribution. Probe results are **cached per session** and refreshed after an
apply/restore of that tweak (detection never re-spawns a shell per status poll). An action **without**
`probe` does not contribute to detection at all.

### 12.6 Inline scripts only (no filed form in v1)

`apply`, `undo`, and `probe` are **plain string bodies**: typically a YAML block scalar (`|`) for
multi-line scripts, or a quoted one-liner. There is **no `apply: { file: scripts/x.ps1 }` filed-script
form in the shipped schema**: writing a map there is a build error (`data did not match … EffectRaw`).
Put the script body inline.

> 📝 _Note for maintainers:_ spec §7 describes a filed-script form (`apply: { file: … }`, embedded by
> `build.rs`). The **shipped `ActionRaw` schema accepts only a string**, so filed scripts are not
> available today. This guide documents the shipped behavior. See the task report for this discrepancy.

**Execution mechanics:** `powershell` runs via `powershell.exe -EncodedCommand` (base64 of UTF-16LE), so
size, loops, quotes, and special characters carry **no escaping risk**; `cmd` runs the body from a temp
script file via `cmd.exe /c`. Every script has a **bounded timeout**: a hang is killed and surfaced as
a typed error, never a silent success.

### 12.7 Actions and distinctness (the subtle part)

Whether an action can distinguish two options depends on `probe` **and** `undo` (spec §8.4/§10):

- An action with **`probe` and `undo`** is a _reliable_ distinguisher: an option that **runs** it expects
  the probe _present_; an option that **omits** it expects _absent_; apply drives the state, so the
  expectation is **strict** and exactly one option matches. Two options may differ **solely** by such an
  action: this is legal (`differ_only_by_undo_action_ok` is a passing fixture).
- An action with **`probe` but no `undo`** is **not** a reliable sole distinguisher. The permanent
  product of a one-way run is **Residue**: once it has run, the _omitting_ option also matches (the
  Residue is tolerated). Two options that differ **only** by a no-undo action are a build error
  (`ResidueOnlyDistinguisher`, §16). The active option discloses the lingering Residue as an info marker.
- An action **without `probe`** contributes nothing to detection. Two options that differ only by
  probe-less actions are a build error (`OptionsNotDetectablyDistinct`, §16).

**Practical rule:** if two options must be told apart by an action, that action needs **both `probe` and
`undo`**: otherwise give the options a non-shared, detectable **Setting** difference.

### 12.8 Delete-tree: reserved, but not authorable in v1

The engine reserves one structural action, **delete-tree** (one-way unless the author supplies `undo`).
It **has no YAML mapping in v1**: you cannot author it. Reach for value-driven deletion instead:
`absent` removes a registry value or field, and a `registry_key` effect driven to `absent` removes a key
the engine created (§4.2, §6). There is **no `create_key` and no `delete_value` action**: both are
subsumed by Settings (§18).

### 12.9 Full action example with a Setting for detectability (from `examples.yaml`)

```yaml
- id: example_action
  name: "Example: Action (Probe + Undo, Plus Ephemeral)"
  description: "An undo-carrying probeable Action plus a separate ephemeral one-shot."
  risk_level: low
  elevation: user
  reversible: true
  effects:
    - id: demo_marker # non-shared Setting keeps both options distinct/detectable
      registry: { key: 'HKCU\Software\MagicXToolboxExample\Action', name: RunMarker, type: REG_DWORD }
    - id: marker_action
      action:
        apply: |
          New-Item -Path 'HKCU:\Software\MagicXToolboxExample\Action' -Force | Out-Null
          New-ItemProperty -Path 'HKCU:\Software\MagicXToolboxExample\Action' -Name ScriptMarker -Value 1 -PropertyType DWord -Force | Out-Null
        undo: |
          Remove-ItemProperty -Path 'HKCU:\Software\MagicXToolboxExample\Action' -Name ScriptMarker -Force -ErrorAction SilentlyContinue
        probe: |
          if (Get-ItemProperty -Path 'HKCU:\Software\MagicXToolboxExample\Action' -Name ScriptMarker -ErrorAction SilentlyContinue) { exit 0 } else { exit 1 }
        shell: powershell
    - id: flush_dns
      action:
        apply: "ipconfig /flushdns"
        ephemeral: true
        shell: cmd
  options:
    - label: "Run"
      values:
        demo_marker: 1
        marker_action: run
        flush_dns: run
    - label: "Skip"
      values:
        demo_marker: 0
```

Note "Skip" **omits** both actions (legal, omitted actions are not run) and relies on `demo_marker: 0`
to stay detectable/distinct from "Run".

---

## 13. Elevation

The app ships **unelevated** (`asInvoker`); Admin is **user-provided** (launch as admin, or the in-app
**Elevate** relaunch), never silently acquired (ADR-0005). You declare a privilege level; the app never
infers or escalates it.

### 13.1 The four levels

| level    | what it means at runtime                                                                 |
| -------- | ---------------------------------------------------------------------------------------- |
| `user`   | in-process, as the interactive user (per-user / HKCU state must land in the user's hive) |
| `admin`  | in-process, in the elevated app (a persistent property of the process once granted)      |
| `system` | a fresh, short-lived child from winlogon's duplicated token                              |
| `ti`     | a fresh, short-lived child via starting the TrustedInstaller service + parent spoofing   |

### 13.2 The floor + per-effect escalation

- A tweak declares an `elevation:` **floor** (required, §3).
- An effect may declare its **own** `elevation:`; the effective level for that effect is
  **`max(floor, step)`**, escalate-only, never lowered.

```yaml
elevation: admin # floor for the whole tweak
effects:
  - id: medic_task
    task: { path: '\Microsoft\Windows\WaaSMedic\PerformRemediation' }
    elevation: ti # this ONE effect escalates to TrustedInstaller
```

### 13.3 The HKCU exception (and why)

**A user-hive (HKCU) effect always runs in-process as the interactive user, regardless of the floor**,
even inside a `system`/`ti` tweak. **Why:** if it ran in a System/TI child, every HKCU write, read-back,
and detection would target the _wrong_ account's hive (SYSTEM's, or an elevated admin's), reporting green
against a hive the user never sees (ADR-0005). The exception keeps per-user state landing in the real
user's hive.

Relatedly, the **over-the-shoulder guard**: if a _different_ admin's credentials elevated the app (its
HKCU is that admin's hive), User-level (HKCU-touching) tweaks are **disabled** with a clear message,
rather than silently writing to the wrong hive.

### 13.4 Choosing a level

- Per-user settings (HKCU) → `user`.
- Machine settings requiring admin (most HKLM policy values, service start types) → `admin`.
- Resources readable/writable only as SYSTEM → `system`.
- TrustedInstaller-protected resources (WaaSMedic-class keys/tasks) → `ti`.

Pick the **lowest** level that actually works, but the level is **trusted, not build-validated** (the
privilege a resource needs is a property of the _machine_, not the tweak). A too-low declaration surfaces
at apply time as a **named insufficient-elevation error** (abort + rollback), never a silent escalation.
Two distinct failures are surfaced: _couldn't acquire the level_ (environmental, TI service unstartable,
`SeDebugPrivilege` denied) vs. _acquired but access-denied_ (the declaration is genuinely too low; fix
it).

### 13.5 When the app is not elevated

- **Reads run at whatever level the app currently has.** Most state is world-readable, so detection works
  unelevated; TI-protected resources legitimately deny reads and read as **Unknown** with a
  needs-elevation hint until the user elevates.
- Tweaks whose floor exceeds the current level are **disabled** in the UI (status still shown), enabled
  only after the user chooses to elevate. Elevation triggers an automatic full re-scan.

### 13.6 The one elevation build guard

There is exactly one elevation-related build guard: **you cannot disable the `TrustedInstaller` service**
via a typed Service effect (`TrustedInstallerDisabled`, §16): it would strand the app's own TI path.
(Script contents are statically opaque, so this guard is honestly scoped to _typed_ effects.)

### 13.7 Current limitation: which kinds actually route through `system`/`ti` today

> ⚠️ **Current limitation:** declaring `elevation: system` or `elevation: ti` (as the tweak's floor, or
> as a per-effect escalation, §13.2) only actually reaches the elevation broker for four kinds today.
> Every other kind **builds clean** at `system`/`ti` but fails every real apply with an
> unsupported-elevation-level error, because `engine::AllKinds::drive` has no broker translation for
> them yet:
>
> - **Routed through the broker at `system`/`ti`:** whole-value `registry` effects (no `field`),
>   `registry_key`, `service`, `task`.
> - **Not routed yet; fails every apply at `system`/`ti`:** `hosts`, `firewall`, `action` (including
>   `DeleteTree`), and a `field`-addressed `registry` effect (§11).
>
> Since per-effect elevation only ever escalates (§13.2, never lowers below the tweak's floor), a
> `hosts`/`firewall`/`action`/field-addressed effect anywhere inside a `system`/`ti`-floor tweak
> inherits that unsupported level too. For now, keep those kinds (and any field-addressed `registry`
> effect) inside tweaks whose effective level never rises above `admin`.

---

## 14. Reversibility

`reversible` is a **computed** property that the build checks against your declared flag (spec §6.4). You
must declare it, but you cannot lie about it.

### 14.1 How it is computed

A tweak is **reversible** if and only if **every** effect on its surface is one of:

- a **Setting** (registry, registry_key, service, task, hosts, firewall), reversible by construction, or
- a **`shared`** reference, reversible via the claim/release lifecycle (§9), or
- an **Action with `undo`**, reverts cleanly, or
- an **`ephemeral` Action**, exempt (transient, nothing to revert).

A tweak is **one-way** (`reversible: false`) if it has **at least one non-ephemeral Action without
`undo`**.

### 14.2 The build check

The build computes the true value and rejects a mismatch (`ReversibilityMismatch`, §16):

```
tweak `T` declares reversible: true but the computed value is false: reversible requires every effect to be a Setting, an undo-carrying Action, or an ephemeral Action
```

Fix it by either **correcting the flag** (`reversible: false`) or **making the tweak reversible** (add
`undo` to the offending action, or mark it `ephemeral` if it truly is transient).

```yaml
# ❌ WRONG: a no-undo, non-ephemeral action but reversible: true
reversible: true
effects:
  - id: one_way
    action: { apply: "some-irreversible-thing", shell: cmd } # no undo, not ephemeral
    # → computed reversible = false → ReversibilityMismatch

# ✅ RIGHT (option A): declare the truth
reversible: false

# ✅ RIGHT (option B): give it an undo
reversible: true
effects:
  - id: one_way
    action:
      apply: "some-thing"
      undo: "the-reverse-thing"
      shell: cmd
```

### 14.3 What "Needs Attention" means at runtime

A `reversible: false` tweak is **labelled one-way up front**, before apply. On restore, **everything else
still reverts**: only the genuinely one-way action cannot, and it surfaces as **Needs Attention**
(ADR-0001). "Partial" never means "nothing reverts." Likewise, a rollback that cannot fully complete
(a locked service, access denied) surfaces as **Needs Attention** with its snapshot kept for retry:
never hidden.

---

## 15. Detectability & the status model

Detection reads the live surface and compares it to your options. Understanding the statuses tells you
what an author must guarantee.

### 15.1 Detectability is typed

- **Settings are always detectable** (read the address, compare).
- **Actions are detectable iff they declare `probe`** (§12.5); a probe-less action contributes nothing to
  detection.
- **Shared** claims count as matching for every claiming option while any claim is held; an `unclaimed`
  entry asserts nothing and is excluded from that option's detectable projection.

There is **no `skip_validation` flag** (§18): detectability is a structural property, not something you
opt out of.

### 15.2 How detection matches an option

Detect reads each applicable, detectable, non-shared Setting once; maps a `Missing` optional through its
`if_missing`; folds in probeable actions' cached present/absent; and counts claimed shared settings as
matching. Then:

- **A matching option wins.** At most one option can match: guaranteed by the distinctness guard (§16).
- **No match ⇒ System Default.**
- **A read that fails ⇒ Unknown** (never System Default, never a guess).

### 15.3 The statuses an author causes

From the author's point of view, here is what makes a tweak show each status:

| status                            | what makes it show                                                                                       | authoring lever                                                                            |
| --------------------------------- | -------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| **Active** (an option is applied) | the live surface matches exactly one option                                                              | your options are distinct + detectable                                                     |
| **System Default**                | the live surface matches **no** option                                                                   | the machine drifted / was never in a defined state                                         |
| **Unknown**                       | a read failed: access denied, malformed packed value, a **non-optional** Missing resource                | use `optional`/`if_missing` for resources that may be absent; declare adequate `elevation` |
| **Unavailable on this machine**   | an option needs a value a Missing resource can't satisfy, or the tweak/effect is version-scoped out here | expected for `optional` effects and `windows`-scoped tweaks (§8, §10)                      |
| **Needs Attention**               | an apply/rollback/restore could **not fully complete**                                                   | a runtime outcome, not a detection verdict (§14)                                           |

### 15.4 What the build guarantees for you

The distinctness and detectability guards (§16) guarantee, on **every** supported build, that:

- every option has **≥1 non-optional detectable effect** (so it is always tellable-apart), and
- **at most one option can match** (no two options are byte-identical, identical on their detectable
  projection, distinguished only by a shared claim, or distinguished only by a no-undo action).

If your corpus builds, these hold. §16 is where you turn when it does not build.

---

## 16. Build errors reference (all 21)

`build.rs` runs three phases in order and stops at the first phase that fails, printing a framed report
listing **every** error in that phase (you fix them all in one pass):

1. **Load** (`schema.rs`): parse YAML, parse paths/literals/scopes. → `YAML LOAD FAILED`
2. **Structural** (`validate_structural`): ownership, coverage, reversibility, etc. → `STRUCTURAL VALIDATION FAILED`
3. **Semantic** (`validate_semantic`): detectability & distinctness, **per support-matrix milestone**. → `SEMANTIC VALIDATION FAILED`

Below is **every** build-error variant, the message you will see (paraphrased from the validator), what
triggers it, **why** the rule exists, and a wrong→right fix. The 21 `ValidationError` variants are
grouped by phase.

### Load-phase errors (6)

#### 1. `Yaml`: malformed YAML or an unknown/mistyped key

> **Message:** `{file}: {message}`

**Trigger:** the file is not valid YAML, uses an **unknown field** (`deny_unknown_fields`), or an effect
has **zero or two kind keys** / a misspelled kind key (the untagged-enum message `data did not match any
variant of untagged enum EffectRaw`). **Why:** a typo must never be silently ignored: it would compile
to something you did not write.

```yaml
# ❌ unknown field
- id: t
  reversibl: true # typo → Yaml error naming the file

# ❌ two kind keys on one effect
- id: e
  registry: { … }
  service: { … } # → "data did not match any variant of untagged enum EffectRaw"

# ✅ exactly one, correctly-spelled kind key
- id: e
  registry: { … }
```

#### 2. `InvalidAddress`: a registry path (or other address) failed to parse

> **Message:** ``tweak `T` effect `E`: {parse error}``

The wrapped parse error is one of (spec §5.1):

- `registry path "…" uses a forward slash: registry paths use backslashes only`
- `registry path "…" has no key path: expected HIVE\Key\..., e.g. HKLM\Software\...`
- `registry path "…" starts with a backslash: remove the leading backslash`
- `registry path "…" ends with a backslash: remove the trailing backslash`
- `registry path "…" contains an empty segment: check for a doubled backslash`
- `registry path "…" does not start with a supported hive: use HKLM or HKCU (short or long spelling)`

**Why:** exact, well-formed addresses are what the ownership guard and snapshot keys depend on; a
trailing backslash was historically a real delete-the-wrong-thing hazard.

```yaml
# ❌
registry: { key: 'HKCR\Software\X', name: N, type: REG_DWORD } # unsupported hive
registry: { key: 'HKLM\Software\X\', name: N, type: REG_DWORD } # trailing backslash
# ✅
registry: { key: 'HKLM\Software\X', name: N, type: REG_DWORD }
```

#### 3. `InvalidSharedSetting`: a `shared:` block's address or value failed to parse

> **Message:** ``shared `id`: {reason}``

**Trigger:** a `shared:` entry's address is malformed, or its `value:` is not legal in the setting's
domain. **Why:** a shared declaration is an address _and_ a value; both must be valid before any tweak
claims it.

```yaml
# ❌ shared REG_DWORD with a non-numeric value
shared:
  - id: bad
    registry: { key: 'HKLM\Software\X', name: N, type: REG_DWORD }
    value: "not a number" # → InvalidSharedSetting
    # ✅
    value: 0
```

#### 4. `InvalidOptionValue`: an option's value doesn't parse or doesn't fit the kind

> **Message:** ``tweak `T` option `O` effect `E`: {reason}``

**Trigger:** the value under an effect id does not parse, or is the wrong shape for the kind: a service
value that is not a start type, an action value that is not `run`, a shared value that is not
`claim`/`unclaimed`, a `null`/empty value (`value is null or empty: write absent to delete it, or supply
a literal`), a list on a non-`REG_MULTI_SZ` type, etc. It also fires when an option keys a value to an
effect id **that does not exist** on the surface (`no effect with this id is declared on the tweak's
surface`). **Why:** every option value must be meaningful for the effect it targets.

```yaml
# ❌ forgotten value
values: { demo_flag } # → "value is null or empty: write `absent` …"
# ❌ wrong keyword for a service
values: { demo_service: on } # not a start type
# ✅
values: { demo_flag: absent, demo_service: manual }
```

#### 5. `InvalidWindowsScope`: a `windows:` block failed to parse

> **Message:** ``tweak `T` {context}: {parse error}``

The wrapped parse error is a bad build/revision expression (`… is not a valid windows build expression:
use N, >=N, <=N, or A..B`), an unknown product (`… is not a supported windows product: use 10 or 11`),
or **revision without a pinned build** (`revision requires build to pin a single exact build …`). Legal
at tweak, effect, or per-option-value level. **Why:** version scoping must be unambiguous, and a revision
only means something inside one exact build (§10.3).

```yaml
# ❌
windows: { build: ">= 26100" } # space after >= → invalid expression
windows: { products: [7] } # unknown product
windows: { revision: ">=2314" } # revision without a pinned build
# ✅
windows: { build: ">=26100" }
windows: { products: [10, 11] }
windows: { build: 26100, revision: ">=2314" }
```

#### 6. `InvalidIfMissing`: an `if_missing:` value doesn't parse in the effect's domain

> **Message:** ``tweak `T` effect `E` if_missing: {reason}``

**Trigger:** `if_missing:` holds a value that is not legal for the effect's kind (a service `if_missing`
that is not a start type, etc.). **Why:** `if_missing` stands in for a real reading of this effect, so it
must be a value the effect could actually have.

```yaml
# ❌
- id: demo_service
  service: { name: RemoteRegistry }
  optional: true
  if_missing: on # not a start type → InvalidIfMissing
  # ✅
  if_missing: disabled
```

### Structural-phase errors (10)

#### 7. `UnresolvedSharedRef`: a `shared: <id>` names no declared shared setting

> **Message:** ``tweak `T` effect `E` references shared `X`, which no `shared:` block declares: check for a typo or add the missing entry``

**Trigger:** an effect's `shared: <id>` id does not exist anywhere in the corpus. **Why:** a dangling
reference cannot be claimed; usually a typo or a missing `shared:` entry.

```yaml
# ❌
effects: [{ id: r, shared: telementry_off }] # typo
shared: [{ id: telemetry_off, … }]
# ✅ ids match
effects: [{ id: r, shared: telemetry_off }]
```

#### 8. `DuplicateAddress`: two owners claim one address

> **Message:** `{address} is claimed by both {first} and {second}: merge them into one effect, reassign one to a different address, or extract a corpus-level shared: entry if they must always agree`

**Trigger:** two effects (in the same or different tweaks), an effect and a `shared:` declaration, or two
`shared:` declarations claim the **same** address, including a whole-value-vs-field mix on one packed
value, and counting all owners across the corpus. **Why:** one address, one owner: dual ownership breaks
tweaks on revert (ADR-0006, §9). With three colliding owners you get **one error per extra owner**.

```yaml
# ❌ two tweaks writing the same value
# tweak_a: registry HKLM\...\Foo   +   tweak_b: registry HKLM\...\Foo
# ✅ extract a shared: entry both claim, or merge the tweaks, or give one a different address
```

#### 9. `DuplicateSharedId`: a `shared:` id is declared twice

> **Message:** ``shared id `X` is declared more than once: shared ids must be unique corpus-wide; rename one of the declarations``

**Trigger:** two `shared:` entries (anywhere in the corpus) use the same id. **Why:** the id must
uniquely resolve; it is checked corpus-wide because `shared:` blocks from all files merge (§2).

#### 10. `NonCanonicalKind`: a raw registry effect reaching a service/task's storage

> **Message:** ``tweak `T` effect `E` addresses {path} as a raw registry value: use the `{Service|Task}` kind instead``

**Trigger:** an HKLM `registry` effect whose value is a service's `…\Services\<name>\Start`, or whose key
is under the Task Scheduler storage tree (`…\Schedule\TaskCache\…`). **Why:** those states have canonical
kinds; reaching them raw would let ownership be dodged via a second address space (ADR-0006).

```yaml
# ❌
registry: { key: 'HKLM\SYSTEM\CurrentControlSet\Services\wuauserv', name: Start, type: REG_DWORD }
# ✅
service: { name: wuauserv }
```

#### 11. `MissingCoverage`: an option omits a Setting effect

> **Message:** ``tweak `T` option `O` does not cover effect `E`: every option must supply a value for every Setting effect on the surface``

**Trigger:** an option's `values:` has no entry for one of the tweak's Setting effects. **Why:** the
coverage rule (§7.1) is what prevents stranded state.

```yaml
# ❌ "Off" omits demo_service
options:
  - label: "On"  { values: { demo_marker: 1, demo_service: manual } }
  - label: "Off" { values: { demo_marker: 0 } }             # missing demo_service
# ✅ every option values every Setting effect
```

#### 12. `SharedNotExplicit`: an option omits a shared effect's claim/unclaimed

> **Message:** ``tweak `T` option `O` does not explicitly say `claim` or `unclaimed` for shared effect `E` ``

**Trigger:** an option's `values:` does not name `claim`/`unclaimed` for a `shared` effect. **Why:**
sharing must always be a visible decision, never implied by omission (§9.2).

```yaml
# ❌ "Off" omits the shared entry
options:
  - label: "On"  { values: { demo_marker: 1, shared_ref: claim } }
  - label: "Off" { values: { demo_marker: 0 } }             # missing shared_ref
# ✅
  - label: "Off" { values: { demo_marker: 0, shared_ref: unclaimed } }
```

#### 13. `ReversibilityMismatch`: the declared flag disagrees with the computed value

> **Message:** ``tweak `T` declares reversible: {declared} but the computed value is {computed}: reversible requires every effect to be a Setting, an undo-carrying Action, or an ephemeral Action``

**Trigger:** `reversible:` does not match what the effects actually support. **Why:** a one-way tweak
must be labelled honestly, before apply (§14). Fix the flag, add an `undo`, or mark the action
`ephemeral`.

#### 14. `TrustedInstallerDisabled`: a typed effect disables TrustedInstaller

> **Message:** ``tweak `T` effect `E` disables the TrustedInstaller service via a typed effect: this would strand the app's own TI elevation path``

**Trigger:** a `service: { name: TrustedInstaller }` effect that any option drives to `disabled`.
**Why:** the app's own TI elevation depends on starting that service (§13.6, ADR-0005).

#### 15. `IfMissingWithoutOptional`: `if_missing` on a non-optional effect

> **Message:** ``tweak `T` effect `E` declares if_missing without optional: true; add `optional: true`, or drop if_missing``

**Trigger:** an effect has `if_missing:` but not `optional: true`. **Why:** a non-optional effect never
reads `Missing` (it errors instead), so `if_missing` there is dead authoring (§8.3).

```yaml
# ❌
- id: s
  service: { name: X }
  if_missing: disabled # no optional: true
# ✅
- id: s
  service: { name: X }
  optional: true
  if_missing: disabled
```

#### 16. `EphemeralWithUndoOrProbe`: an ephemeral action carries undo/probe

> **Message:** ``tweak `T` effect `E` is ephemeral but declares undo/probe: an ephemeral action takes neither (spec §7)``

**Trigger:** an action with `ephemeral: true` also has `undo:` or `probe:`. **Why:** an ephemeral action
is _exempt_ from reversibility/detectability: carrying either would let the engine call `undo`/`probe`
on an action those computations never accounted for (§12.3).

```yaml
# ❌
action: { apply: "ipconfig /flushdns", ephemeral: true, probe: "…", shell: cmd }
# ✅
action: { apply: "ipconfig /flushdns", ephemeral: true, shell: cmd }
```

### Semantic-phase errors (5): quantified per support-matrix milestone

These run **per Windows build** in the support matrix (`19045`, `22621`, `22631`, `26100`), over each
milestone's applicable projection. The message names the **first** build the failure was seen on; fix the
option/pair, not each build (§10.6).

#### 17. `NotDetectable`: an option has no non-optional detectable effect on some build

> **Message:** ``tweak `T` option `O` has no non-optional detectable effect on Windows build {N}: every option must stay distinguishable without effects that may read Missing``

**Trigger:** on build N, this option's only distinguishing effects are optional (may read `Missing`),
probe-less actions, or an `unclaimed`/scoped-out value: nothing reliably detectable remains. **Why:** an
option you cannot detect is an option the user can never see as active (§8.5, §15).

```yaml
# ❌ the only effect is optional, or is version-scoped out on this build for this option
# ✅ add a non-optional detectable Setting (see §8.5), or widen the scope
```

#### 18. `OptionsByteIdentical`: two options are identical on a build

> **Message:** ``tweak `T` options `A` and `B` are byte-identical on Windows build {N}: merge them or give one a distinct value``

**Trigger:** on build N, two options have the _same_ value for every applicable effect. **Why:** two
identical options are one option; the user could never land distinctly on either.

```yaml
# ❌ On and Off both set demo_flag: 1 (once scoping strips the only difference)
# ✅ give them a real, detectable difference, or merge them
```

#### 19. `OptionsNotDetectablyDistinct`: two options differ only where detection can't see

> **Message:** ``tweak `T` options `A` and `B` are identical on their detectable projection on Windows build {N}: they differ only by effects detection cannot observe (e.g. a probe-less Action)``

**Trigger:** two options differ, but only on **probe-less actions**: nothing `detect()` can read.
**Why:** if detection cannot tell two options apart, "which one is active?" has no answer (§12.7).

```yaml
# ❌ On runs a probe-less action, Off omits it; nothing else differs
# ✅ add a probe to the action AND an undo (so it's a reliable distinguisher), or add a Setting difference
```

#### 20. `SharedOnlyDistinguisher`: two options differ only by a shared claim

> **Message:** ``tweak `T` options `A` and `B` differ only by a shared effect on Windows build {N}: a claimed shared value can be held by another tweak too, so it cannot be the sole distinguisher``

**Trigger:** the only difference between two options is a `shared` effect's `claim`/`unclaimed`. **Why:**
a shared value can be held by another tweak, so it cannot reliably tell _this_ tweak's options apart
(§9.4).

```yaml
# ❌ On: shared_ref claim; Off: shared_ref unclaimed; nothing else differs
# ✅ pair the shared effect with a non-shared Setting that differs (§9.4)
```

#### 21. `ResidueOnlyDistinguisher`: two options differ only by a no-undo action's Residue

> **Message:** ``tweak `T` options `A` and `B` have no reliable distinguisher on Windows build {N}: add a Setting or an undo-carrying probeable Action that differs between them; a no-undo Action's Residue lets the omitting option match too once it has run``

**Trigger:** every differing effect between two options is a no-undo (or probe-less, or shared) effect:
none reliably keeps at most one option matching once the state is reached. **Why:** a one-way action's
permanent Residue is tolerated by the omitting option, so both would match after it runs (§12.7). A
**probe + undo** action _is_ reliable; a no-undo one is not.

```yaml
# ❌ On runs a probeable but no-undo action; Off omits it; nothing else differs
# ✅ give the action an `undo` (making it reliable), or add a non-shared Setting difference
```

---

## 17. Complete worked examples

These walk the shipped `examples.yaml` tweaks end-to-end, plus a couple of harder composed cases. Each is
schema-valid against the shipped validator.

### 17.1 Registry tri-state (a value and its absence)

```yaml
- id: example_registry_tristate
  name: "Example: Registry Tri-State"
  description: "A registry value whose two options are a real DWORD and the reserved absent keyword."
  risk_level: low # advisory
  elevation: user # HKCU → in-process as the user
  reversible: true # only a Setting → reversible by construction
  effects:
    - id: demo_flag
      registry: { key: 'HKCU\Software\MagicXToolboxExample\Registry', name: DemoFlag, type: REG_DWORD }
  options:
    - label: "Enabled"
      values: { demo_flag: 1 } # write DemoFlag = 1
    - label: "Disabled"
      values: { demo_flag: absent } # delete DemoFlag entirely
```

Two options, both valuing the one Setting, differing on a detectable value → valid. UI: a dropdown of
Default / Enabled / Disabled.

### 17.2 Service + task with presence (optional / if_missing)

```yaml
- id: example_service_task
  name: "Example: Service + Scheduled Task"
  description: "Optional Service/Task Settings with if_missing, plus a non-optional marker for detectability."
  risk_level: low
  elevation: admin # service/task start-type changes need admin
  reversible: true
  effects:
    - id: demo_marker # NON-optional: keeps both options detectable if a resource is Missing
      registry: { key: 'HKCU\Software\MagicXToolboxExample\ServiceTask', name: Marker, type: REG_DWORD }
    - id: demo_service
      service: { name: RemoteRegistry }
      optional: true
      if_missing: disabled # a machine without the service counts as `disabled`
    - id: demo_task
      task: { path: '\Microsoft\Windows\DiskCleanup\SilentCleanup' }
      optional: true
      if_missing: disabled
  options:
    - label: "Enabled"
      values: { demo_marker: 1, demo_service: manual, demo_task: enabled }
    - label: "Disabled"
      values: { demo_marker: 0, demo_service: disabled, demo_task: disabled }
```

Note the marker: without it, on a machine lacking the service _and_ task, both options would map through
`if_missing: disabled` and become indistinguishable → `NotDetectable`. The marker is the non-optional
distinguisher (§8.5).

### 17.3 Hosts + firewall presence

```yaml
- id: example_hosts_firewall
  name: "Example: Hosts + Firewall"
  description: "Hosts and Firewall presence Settings on safe, reserved scratch targets."
  risk_level: medium
  elevation: admin # hosts file + firewall both need admin
  reversible: true
  effects:
    - id: block_host
      hosts: { ip: "0.0.0.0", domain: "magicx-toolbox-example-5f3f1d2e.invalid" }
    - id: block_rule
      firewall:
        name: "MagicX Toolbox Example Rule 5F3F1D2E"
        direction: outbound
        action: block
        protocol: tcp
        remote_addresses: ["203.0.113.0/24"]
        description: "Example firewall rule."
  options:
    - label: "Blocked"
      values: { block_host: present, block_rule: present }
    - label: "Allowed"
      values: { block_host: absent, block_rule: absent }
```

Both are presence kinds → `present`/`absent`. Both options value both effects and differ on detectable
presence → valid.

### 17.4 Action with undo+probe plus a separate ephemeral

See §12.9 for the full `example_action`. Key points: the undo-carrying probeable action is a legal sole
distinguisher, but this tweak _also_ carries a `demo_marker` Setting so it does not need to rely on that;
the ephemeral `flush_dns` is exempt from reversibility/detectability; the "Skip" option legally omits
both actions.

### 17.5 A Windows-scoped tweak walked through the matrix

Tweak-level scope makes a whole tweak apply only on 24H2+ (from `examples.yaml`):

```yaml
- id: example_windows_scoped
  name: "Example: Windows-Scoped Tweak"
  description: "Tweak-level windows: build >=26100, unavailable, with a reason, on earlier builds."
  risk_level: low
  elevation: user
  reversible: true
  windows: { build: ">=26100" }
  effects:
    - id: modern_flag
      registry: { key: 'HKCU\Software\MagicXToolboxExample\Modern', name: ModernFlag, type: REG_DWORD }
  options:
    - label: "Enabled"
      values: { modern_flag: 1 }
    - label: "Disabled"
      values: { modern_flag: 0 }
```

Walking the support matrix: on `19045`, `22621`, `22631` the tweak's applicable surface is **empty** →
the tweak is **skipped** (shown unavailable, not an error). On `26100` it applies, and both options
differ on the detectable `modern_flag` → valid.

Now a harder composed case mixing all three scoping levels with an always-in-scope base marker (so no
option is ever stranded, cf. §10.6):

```yaml
- id: scoped_composed
  name: "Scoped Composed"
  description: "tweak-, effect-, and per-option-value windows scoping, with an always-in-scope base."
  risk_level: low
  elevation: user
  reversible: true
  windows: { products: [10, 11] } # tweak-level: all supported builds
  effects:
    - id: base # always in scope, non-optional → keeps every option detectable
      registry: { key: 'HKCU\Software\Example\WinBase', name: BaseFlag, type: REG_DWORD }
    - id: modern
      registry: { key: 'HKCU\Software\Example\Modern', name: ModernFlag, type: REG_DWORD }
      windows: { build: ">=22621" } # effect-level: only exists on 22621+
  options:
    - label: "On"
      values:
        base: 1
        modern: { value: 1, windows: { build: ">=26100" } } # per-value: only on 26100+
    - label: "Off"
      values:
        base: 0
        modern: 0
```

Matrix check:

- **19045:** `modern` (effect-scoped `>=22621`) is out → surface = `[base]`. On: `base:1`, Off: `base:0`.
  Distinct on a Setting, both detectable. ✅
- **22621 / 22631:** surface = `[base, modern]`. On's `modern` value is per-value-scoped `>=26100` → **no
  answer** here; Off's `modern` = 0. On and Off differ on `base` (a Setting) → distinct; both detectable
  via `base`. ✅
- **26100:** On: `base:1, modern:1`; Off: `base:0, modern:0`. Distinct, detectable. ✅

Every milestone passes because `base` is always the reliable, non-optional distinguisher.

### 17.6 A packed field pair

See §11.5 for `example_packed_field`. The packed REG_SZ `DemoPacked` is field-addressed at `DemoFlag`;
the two options write `"1"`/`"0"` into just that field, preserving any other fields in the value.

---

## 18. What is gone from the old schema

The redesign deleted the option-centric schema wholesale. If you saw the old MagicX schema (or any
option-centric tool), here is what no longer exists and what replaces it:

| gone                                                                                    | replaced by                                                                                     |
| --------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| `requires_admin` (auto-inferred), `requires_system`, `requires_ti`                      | the four-level `elevation` **floor** + per-effect escalation (§13)                              |
| `skip_validation`                                                                       | nothing: detectability is structural; you cannot opt out (§15)                                  |
| `ignore_not_found`                                                                      | typed presence: `optional` / `if_missing` (§8)                                                  |
| `task_name_pattern` (patterns/wildcards)                                                | **exact** task paths + `optional` (§4.4)                                                        |
| `is_default` / `is_toggle` / numeric option indices                                     | shape computed from option count; "System Default" is a computed status (§1.3, §7.2, ADR-0003)  |
| `options:` as `registry_changes` / `service_changes` / `scheduler_changes` change-lists | the **effect-centric surface** + flat option value-maps (§1, §7)                                |
| `pre_commands` / `pre_powershell` / `post_commands` / `post_powershell`                 | `action` effects (§12)                                                                          |
| `action: delete_value` / `delete_key` / `create_key`                                    | driven state: `absent` deletes a value/field; `registry_key` presence handles keys (§4.2, §6)   |
| `value: null` as a delete spelling                                                      | a **build error** naming `absent`: deletion is the deliberate `absent` keyword (§6.4, ADR-0004) |
| `.reg` import / `-` delete spellings                                                    | the YAML schema is the single source of truth (§5.3)                                            |
| per-tweak `category:`                                                                   | category declared **once per file** (§2.1)                                                      |

There is **no mechanical converter.** If you are porting an old tweak, re-author it from scratch against
this guide. The old content defects (undetectable tweaks, colliding owners, lying reversibility) are
_meant_ to die in the rewrite: the build guards (§16) will not let them through.

---

## 19. Authoring checklist / do's & don'ts

Run through this before you commit a tweak.

### Structure

- ✅ One `category:` per file (id, name, icon, description). ❌ No per-tweak `category`.
- ✅ Each tweak has `id`, `name`, `description`, `risk_level`, `elevation`, `reversible`, `effects`,
  `options`. ❌ Never omit a required field.
- ✅ Each effect has an `id` + **exactly one** kind key. ❌ Never zero or two kind keys.

### Effects & options

- ✅ Every option supplies a value for **every Setting effect** (coverage, §7.1).
- ✅ Every option says **`claim`/`unclaimed`** for each `shared` effect (§9.2).
- ✅ Action entries are `run` **or omitted**: never a made-up keyword.
- ✅ Every option has **≥1 non-optional, detectable, non-shared** distinguishing value on **every**
  support-matrix build (§8.5, §15).
- ❌ Don't let two options differ **only** by an optional effect, a shared claim, a probe-less action, or
  a **no-undo** action (§16 semantic errors).

### Values

- ✅ Use the exact `.reg` type name (`REG_DWORD`, …) and the right literal shape (§5).
- ✅ Deletion is the **`absent`** keyword; `present` only on presence kinds. ❌ Never `null`/omitted as a
  delete (§6).
- ✅ Quote large `REG_QWORD`s (above `i64::MAX`) as strings (§5.1).
- ✅ `REG_MULTI_SZ` is a YAML **list**; `[]` clears it.
- ✅ Use `{ literal: absent }` only when you truly need the string content "absent".

### Addresses & ownership

- ✅ HKLM/HKCU only; no leading/trailing/doubled backslash; no forward slash (§4.1).
- ✅ One address, one owner: use `shared:` for genuine cross-tweak sharing (§9). ❌ Never two effects on
  one address.
- ✅ Use the `service`/`task` kind: ❌ never reach a service/task through raw registry storage (§16 #10).
- ✅ A packed value is whole-owned **XOR** field-addressed; each field owned once (§11.3).

### Presence, scoping, elevation, actions

- ✅ `optional: true` for resources that may be absent; `if_missing` **requires** `optional` (§8).
- ✅ `windows:` axes AND together; `revision` needs a **pinned exact** `build` (§10.3). Walk all four
  milestones for scoped tweaks (§10.6).
- ✅ Pick the **lowest working** `elevation`; HKCU always runs as the user (§13.3). ❌ Never disable
  `TrustedInstaller` via a typed effect (§13.6).
- ✅ `apply` + `shell` required on an action; `undo`/`probe` optional and independent. ❌ `ephemeral`
  takes **no** `undo`/`probe` (§12.3). ✅ Scripts are **inline strings**: no filed form in v1 (§12.6).

### Reversibility & honesty

- ✅ `reversible` must equal the **computed** value: Settings/Shared/undo-actions/ephemeral-actions →
  reversible; one non-ephemeral no-undo action → one-way (§14). ❌ Never declare a `reversible` you can't
  back up.

### Before committing

- ✅ Build the app (`cargo build` / `pnpm run validate`): the tweak validator runs at compile time and
  must pass on every support-matrix build.
- ✅ If it fails, find the error variant in §16, apply the wrong→right fix, and rebuild.
