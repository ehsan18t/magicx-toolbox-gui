---
status: accepted
---

# App items are outside the snapshot model

Removing a built-in app (Clipchamp, Feedback Hub, and the like) does not fit the tweak model. A tweak has authored options over a surface that detection can read back, a snapshot taken before every apply, and a restore that drives the captured state back. A removed app has no state to capture: the package is gone, and the only way back is to download and install it again, which may or may not be possible on a given machine.

We author these as **app items**, a separate list in the same YAML category files (`apps:` beside `tweaks:`). An app item has a presence (Installed, Absent or Unknown) and two actions: Remove, and Install when an install source is usable on this machine. It has no options, no snapshot, no journal, no System Default, no Restore and no Needs Attention. Apps share the tweak id space, so one id never names both.

## Considered Options

- **Model a removal as a tweak with one-way action effects** (the `appx_absent` probe plus a removal script): rejected. The engine then owes the removal a snapshot it can never restore from, a Restore button that cannot work, and a System Default status for an app that is simply installed. Every one of those is a promise the engine cannot keep.
- **Snapshot the package files so Restore can put them back**: rejected. Provisioned and per-user package state is owned by the AppX deployment service; copying it out and back is unsupported and fails in ways the did-it-work check could not always see.

## Consequences

- Removal is final unless an install source exists. An app with no usable install source on this machine carries a Permanent label on its row, and its Remove confirmation says the change cannot be undone.
- Both actions still honour the did-it-work contract: after a removal the app must read Absent, after an install it must read Installed, or the command fails with the reason. Unknown is never treated as Absent, so a row whose presence cannot be read stays visible with its buttons disabled.
- Removal and install run under the same per-id lifecycle lock as an apply, so closing the window, restarting as administrator or installing an update waits for them.
- Favorites, profiles, the pending bar, the applied counters and Restore all ignore apps: none of them has anything to hold for an item without options or history.
- Opening the Store page is not verified by the app. The row checks presence again when the window regains focus.
