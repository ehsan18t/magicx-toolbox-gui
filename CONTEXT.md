# MagicX Toolbox

A Windows tweaking tool. The user picks a target state for a Windows setting; the app captures what was there before, applies the change, and can put it back.

This file is the glossary — the words we use and the ones we deliberately don't. For what the app *is*, see [docs/APP_CONTEXT.md](./docs/APP_CONTEXT.md). For how to author tweaks, see [docs/TWEAK_AUTHORING.md](./docs/TWEAK_AUTHORING.md).

## Language

### The two meanings of "default"

These are different concepts and conflating them has caused real doc contradictions. Never write bare "Default" without qualifying which one you mean.

**Stock Default**:
The value Windows itself ships with, before anyone touched the machine. A property of Windows, identical on every clean install. Appears in option labels like "500 KB (Stock Default)".
_Avoid_: Default, Windows Default, factory setting

**System Default**:
The state one particular machine's setting was in before this app first touched it. A property of *that machine*, and frequently not the Stock Default — a prior tool, a policy, or the user may have changed it. Selectable whenever a Snapshot exists.
_Avoid_: Default, original, unmatched state, unknown state

### Tweaks

**Tweak**:
One configurable Windows setting the app can change, expressed as a set of mutually exclusive target states.
_Avoid_: setting, hack, mod

**Option**:
One complete target state of a Tweak, including every change needed to reach it. A Tweak has at least two.
_Avoid_: value, choice, mode, state

**Applied Option**:
The Option the app most recently put the machine into. Distinct from what the machine currently *matches*, which can drift when something outside the app changes the same setting.
_Avoid_: current option, selected option

**Effect Phase**:
One of the five categories of change an Option can make — registry, services, scheduled tasks, hosts entries, firewall rules. Named as a group because they succeed or fail together.
_Avoid_: change type, action group

### Safety

**Snapshot**:
The captured Original State of everything one Tweak touches, taken before the app first changed it. One per Tweak, not one per Option — switching between Options never replaces it.
_Avoid_: backup, restore point, save state

**Original State**:
What the machine looked like before this app first applied a given Tweak. What a Snapshot holds, and what System Default returns you to.
_Avoid_: initial state, baseline, before state

**Apply**:
Putting the machine into a chosen Option.

**Option Switch**:
Applying a different Option to a Tweak that already has a Snapshot. Preserves the existing Snapshot rather than taking a new one.
_Avoid_: re-apply, change option

**Revert**:
Returning a Tweak to its Original State and releasing the Snapshot. Selecting System Default is a Revert.
_Avoid_: undo, restore, rollback — *rollback* specifically means something else here

**Rollback**:
The automatic attempt to undo a *failed* Apply. Distinct from Revert, which is a deliberate user action on a Tweak that applied successfully.
_Avoid_: revert, undo

**Needs Attention**:
The state of a Tweak whose Rollback did not fully succeed, leaving the machine partly changed. Retains its Snapshot and names the resources that could not be restored.
_Avoid_: failed, broken, partial, error state

**Inferred Status**:
A Tweak status determined by something's *absence* rather than by reading its value — used where a resource does not exist on every Windows edition. Surfaced to the user rather than presented as a confirmed reading.
_Avoid_: assumed, guessed, implied
