# Profile v1 (`.mgx`) — format record

> **Status: historical.** The v1 profile system was deleted in the 2026-07 cleanup and will be rebuilt
> from scratch. This file records the parts of the format that a future implementation cannot recover by
> reading a `.mgx` file, so the rebuild can still read anything produced by v3.0.0.
>
> Full source is in git history before the deletion commit:
> `src-tauri/src/models/profile.rs`, `src-tauri/src/services/profile/`.

## Why this note exists

Everything about the archive is self-describing except **one thing**: the option content hash. It is
embedded in stored profiles and used to resolve options that moved index between the profile being
exported and imported. It cannot be reconstructed from the archive alone — only from the algorithm.

## Archive layout

A Deflate-compressed ZIP containing up to three entries:

| Entry | Present |
| --- | --- |
| `profile.json` | always |
| `system_state.json` | only when exported with `include_system_state` |
| `manifest.json` | always |

`manifest.json` carries `format_version: u32`, written as `PROFILE_SCHEMA_VERSION = 1`, plus
`profile_checksum` and `system_state_checksum`.

Checksums are **SHA-256 over the pretty-printed JSON bytes** — i.e. over the output of
`serde_json::to_string_pretty`, not compact JSON. They are verified on read and a mismatch is a hard
error. They are integrity checks, not signatures: they carry no cryptographic trust.

Known v1 defect for the rebuild to fix: `format_version` was **written but never checked on read**.
Version gating happened later and separately, via `ConfigurationProfile.schema_version`.

## The option content hash — the part that must be preserved

Two algorithms are in play. On import, a stored hash matched **either** of them.

### Current (`profile-option-v2`)

```rust
fn hash_option_content(option: &TweakOption) -> String {
    let canonical = serde_json::to_vec(option).expect(..);   // COMPACT json, not pretty
    let mut hasher = Sha256::new();
    hasher.update(b"profile-option-v2");                     // domain separator, exact bytes
    hasher.update(canonical);
    hex::encode(hasher.finalize())[..32].to_string()         // first 32 hex chars = 128 bits
}
```

Three details that will silently break a reimplementation:

1. The domain separator `b"profile-option-v2"` is hashed **before** the payload.
2. The payload is `serde_json::to_vec` — **compact**, whereas the archive checksums use **pretty**.
3. The digest is truncated to the first **32 hex characters**, not the full 64.

This hash is therefore sensitive to `TweakOption`'s serde representation. Any field added, renamed,
reordered, or any change to `skip_serializing_if`, changes every hash. That fragility is itself a reason
the rebuild may want a structural hash over semantically-meaningful fields instead of over serde output.

### Legacy fallback

An earlier field-order-dependent variant, retained so older profiles still matched:

```rust
// registry: hive.as_str(), key, value_name, and format!("{:?}", value) when Some
// services: name, startup.as_str()
// scheduler: task_path, task_name (when Some), action.as_str()
// no domain separator; same [..32] truncation
```

Note it hashed the registry value via `format!("{:?}", v)` — the Rust `Debug` representation of a
`serde_json::Value`. That is not a stable serialization format and is the main reason v2 replaced it.

## Carried into the rebuild

- `system_state.json` was exported and **never read**. It was meant for an import-time compatibility
  check that was never built. That is the same problem the snapshot install-ID solves — see
  `docs/TWEAK_SYSTEM_PLAN.md`. The rebuilt system should share one machine-identity mechanism rather
  than inventing a second.
- Per-tweak progress events (`ProfileProgressEvent`, `TweakCompleteEvent`) were modelled and never
  emitted, so profile apply had no progress stream.
- The `aliases` field on `TweakDefinition` existed solely to let import resolve renamed tweak IDs. It
  was deleted along with the profile system; the rebuild will need to reintroduce it if it wants ID
  migration.
