# MagicX Toolbox

A Windows tweaking tool. The user picks a target state for a Windows setting; the app captures what
was there before, applies the change, and can walk back through the states the machine actually had.

This file is the glossary — the words we use and the ones we deliberately don't. For what the app
*is*, see [docs/APP_CONTEXT.md](./docs/APP_CONTEXT.md). For how to author tweaks, see
[docs/TWEAK_AUTHORING.md](./docs/TWEAK_AUTHORING.md). For the redesigned engine's rules, see
[the design spec](./docs/superpowers/specs/2026-07-21-tweak-system-redesign-design.md) and ADRs 0001–0007.

## Language

### The two meanings of "default"

These are different concepts and conflating them has caused real doc contradictions. Never write
bare "Default" without qualifying which one you mean.

**Stock Default**:
The value Windows itself ships with, before anyone touched the machine. A property of Windows,
identical on every clean install. Appears in option labels like "500 KB (Stock Default)".
_Avoid_: Default, Windows Default, factory setting

**System Default**:
A computed *status*: the live state of a Tweak's Surface matches none of its authored Options. A
property of that machine at that moment. Never authored, never selectable, never a restore target —
the user leaves it by applying an Option and returns toward prior states only via Restore Snapshot.
_Avoid_: Default, original, revert to default, unmatched state

### Tweaks

**Tweak**:
One configurable piece of Windows behavior the app manages: a declared Surface of Effects plus one
or more mutually exclusive authored Options.
_Avoid_: setting, hack, mod

**Option**:
One complete authored target state — a value for every Setting on the Tweak's Surface. A Tweak has
one or more; System Default is never one of them.
_Avoid_: value, choice, mode, state

**Active Option**:
The authored Option the live Surface currently matches. At most one can ever match; it is shown as
selected.
_Avoid_: applied option, current option, selected option

**Surface**:
Everything a Tweak manages, declared once. Every Option covers all of it, which is what makes
Options comparable and drift impossible to author.
_Avoid_: changes, change list

**Effect**:
One atomic unit of managed state on a Surface — a Setting, a Shared Setting reference, or an
Action. Effects apply in declaration order.
_Avoid_: Effect Phase, change type, action group

**Setting**:
A declarative Effect: an address in Windows state plus a target value. Readable, comparable, and
reversible by construction.
_Avoid_: registry change (as a generic term)

**Action**:
An imperative Effect: a script with a required apply, optional undo, optional probe. One-way unless
it carries undo.
_Avoid_: command, script step

**Ephemeral Action**:
A transient activation step (flush a cache, restart the shell) that changes no persistent state.
Runs on apply; exempt from reversibility and detectability; never makes a Tweak one-way.
_Avoid_: side effect, post step

**Probe**:
An Action's state-based check — answers "is the state this Action produces currently present?" —
used identically as the apply-time did-it-work check and the detect-time contribution.
_Avoid_: verification script, test

**Missing**:
A resource a Tweak addresses that does not exist on this machine (service not installed, task
absent). A capture-only reading with author-declared meaning; the app never installs or uninstalls
anything.
_Avoid_: absent (that is a deletable value-state, not a missing resource), not found, inferred status

**Residue**:
The persisted product of a one-way Action (no undo) that remains after leaving the Option that ran
it. Disclosed via its probe as an info marker; never disqualifies the Active Option match.
_Avoid_: leftover, stale state, orphan

**Shared Setting**:
A single address and target value declared once at corpus level and referenced by several Tweaks.
Applying a referencing Option **claims** it; releasing the last claim restores the captured
original. Two Tweaks can never own one address any other way.
_Avoid_: common registry, overlapping key

**Milestone**:
One Windows build in the declared support matrix. The validator proves every build-time guarantee
against every Milestone.
_Avoid_: version (alone), release

### Statuses

**Unknown**:
The status when a Tweak's Surface cannot be read — access denied, unparseable state, a Missing
resource with no declared meaning. Shown as itself, with the reason; never guessed, never presented
as System Default.
_Avoid_: error state, undetected, inferred status

**Unavailable**:
An Option — or a whole Tweak — that cannot be offered on this machine: a needed resource is Missing,
or the running build is outside the Tweak's scope. Shown with the reason, never silently hidden
behind a failure.
_Avoid_: disabled (that means "app not elevated"), unsupported

**Needs Attention**:
The state of a Tweak after an operation (Rollback or Restore) that could not fully complete, leaving
the machine partly changed. Retains its Return-point and names the exact resources that could not be
restored.
_Avoid_: failed, broken, partial, error state

### Safety

**Snapshot / Return-point**:
One captured pre-Apply state in a Tweak's history. A state that is an authored Option is stored as a
reference to it; an unauthored state (System Default, drift) is stored as a full capture. At most
one entry per authored Option (re-capture moves it to the head); unauthored entries are all kept.
_Avoid_: backup, restore point, save state

**Original State**:
The state a Return-point holds. The oldest entry in a Tweak's history is the machine's state before
the app first touched that Tweak.
_Avoid_: initial state, baseline, before state

**Apply**:
Putting the machine into a chosen Option. Every Apply captures a Return-point first; applying the
already-Active Option is a verified no-op. Switching Options is just an Apply.
_Avoid_: option switch (obsolete term), re-apply

**Restore Snapshot**:
The only restore action: return the machine to the most recent Return-point — consumed on verified
success — and pressing it again walks further back through the history.
_Avoid_: revert, undo, restore to default

**Rollback**:
The automatic attempt to undo a *failed* Apply, using the Return-point captured at its start.
Distinct from Restore Snapshot, which is a deliberate user action.
_Avoid_: revert, undo
