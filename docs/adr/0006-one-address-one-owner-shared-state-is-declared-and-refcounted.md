---
status: accepted
---

# One address, one owner; shared state is declared and refcounted

Two tweaks that both manage the same registry value / service / task / hosts entry / firewall rule
break each other in a way no runtime mediation can fix: when tweak A's revert changes the shared
value, tweak B's option is *genuinely* no longer in effect on the machine — detection honestly shows
B at System Default, and the user experiences a tweak flipping itself off. This happened in practice
with the old corpus (multiple tweaks sharing policy values and `Services\…\Start` keys).

**Decision — build-time, corpus-wide:**

1. **Every address appears at most once**, counting direct effects and `shared` declarations
   together. A direct effect and a shared declaration on one address, two shared declarations on one
   address, or two effects on one address inside a single tweak are all build errors. A packed value
   is whole-owned **xor** field-addressed (each field owned once) — never both.
2. **Kind canonicalization closes the alias routes:** a raw registry effect under
   `…\Services\<X>\Start`, or on a task's registry storage path, is a build error — the Service/Task
   kind must be used, so one underlying state cannot be claimed through two address spaces.
3. **Genuine sharing is a declared, refcounted claim.** A corpus-level `shared` block declares the
   address **and the single target value**; tweaks reference it, and each option says `claim` or
   `unclaimed` (always explicit). At runtime: the **first** claim captures the live original once
   (engine-level claims record — atomic, machine-stamped, in the snapshots directory) and drives the
   value; further claims are verified no-ops; releasing while others still claim leaves the value
   alone and reports *"held by …"* as info; the **last** release restores the captured original,
   verified (ADR-0002 applies). Detection counts a claimed setting as matching for every claimant
   while any claim holds.
4. **Tweaks that want *different* values on one address cannot ship.** That is a semantic conflict no
   mechanism can paper over; the build error names both tweaks and the playbook: merge them (they are
   usually one feature wearing two names), reassign the address to the one tweak it belongs to, or
   extract the shared knob into its own tweak.

## Considered Options

- **Runtime warnings on overlap** — rejected: when the value changes, the other tweak's state is
  truly gone; a warning only narrates the breakage. Detection showing System Default was never the
  bug — the dual claim was.
- **Declared compatibility relations between overlapping options** — rejected: the compatibility
  predicate is undecidable in general, and even "compatible" values still flip the other tweak's
  detected state on revert.
- **Unrestricted sharing with last-writer-wins** — rejected: order-dependent state, cross-tweak write
  races (per-tweak locks cannot cover shared addresses), and unrestorable interleaved snapshots.

## Consequences

- Per-tweak locking is sufficient for everything except the claims record (its own lock) and
  packed-value field writes (a registry-kind mutex), because no other cross-tweak write path exists.
- The corpus rewrite draws ownership boundaries deliberately; overlaps discovered while authoring are
  design feedback ("these are one tweak"), not obstacles to mediate.
- The claims record is a first-class persisted artifact with the same atomicity, machine-stamping,
  and verified-release rules as snapshots.
