# Commands and UI

This is everything between the engine and the user. A thin Tauri command layer builds the engine's dependencies, gates and serializes every operation that changes the machine, and translates engine results into view types. On the frontend, Svelte rune stores hold the catalog and per-tweak statuses, which the background scan fills in, and the tweak rows render each status.

Code: `src-tauri/src/commands/tweaks.rs` (commands and view types), `src-tauri/src/commands/elevation.rs`, `src-tauri/src/setup.rs`, `src-tauri/src/lib.rs` (registration and window events), `src/lib/stores/{tweaksData,tweakActions,tweaksPending}.svelte.ts`, `src/lib/components/items/`.

[Back to the index](README.md)

## Command surface

| Command | Purpose | Gating |
| --- | --- | --- |
| `get_tweaks` | The catalog as view types, with each tweak's required level and availability computed at call time, and `supported: false` on tweaks this Windows build cannot run. | none |
| `get_categories` | Category metadata. | none |
| `get_statuses_stream` | Starts the full background scan and returns at once. Statuses arrive as events. | none |
| `apply_tweak` | Applies one option to one tweak. Returns the outcome and the new status. | availability, then the tweak's lock |
| `restore_tweak` | Restores the tweak's most recent snapshot entry. | availability, then the tweak's lock |
| `get_tweak_status` | A fresh detect of one tweak. Refused, not queued, while the tweak is locked. | none |
| `list_snapshot_entries` | The tweak's history, valid and invalid, oldest first. | none |
| `discard_snapshot_entry` | Deletes one entry on the user's say-so. Refuses another account's entry. | the tweak's lock |
| `keep_current_state` | Accepts the machine as it is: settles marks, clears Needs Attention, discards entries. | the tweak's lock |
| `get_elevation_state` | The app's level and whether the account guard trips. | none |
| `rescan_after_elevation` | Starts another full scan. | none |
| `restart_as_admin` | Relaunches the app elevated through UAC. | exit latch |

App items have their own commands (`get_apps`, `get_app_statuses`, `remove_app`, `install_app`), gated by the same availability check and per-id lock; see [apps.md](apps.md#gates).

The test build adds the Manual Tests run, list and cancel commands, which drive the same gated apply and restore paths (`manual_tests_available` exists in every build); see [MANUAL_TESTS.md](../../MANUAL_TESTS.md).

### Gates

- **Availability**: account guard, then needs elevation, then elevation path unavailable. See [elevation.md](elevation.md#the-availability-gate). Restore is gated like apply; discard and keep current state are not, since they change no Windows state.
- **Per-tweak lock**: one async lock per tweak, held across the engine call, which runs on a blocking thread.
- **Exit latch**: closing the window, relaunching as admin and installing an update all refuse while any tweak or app item is locked. A window close attempt emits a `close-blocked` event that the title bar shows as a toast. Once an exit has started, a new apply fails with `APP_EXITING`.
- **Error shaping**: engine errors reach the frontend as a code and a user-facing message with broker details removed. Snapshot store errors reach it as a generic reason.

## Launch sequence

```mermaid
sequenceDiagram
  autonumber
  participant L as lib.rs and setup.rs
  participant FE as Frontend
  participant CMD as Commands
  participant SCAN as Scan thread

  L->>L: single-instance guard
  L->>L: open snapshot store, claims store, probe cache, read MachineGuid
  L->>L: startup crash scan (records Needs Attention)
  FE->>CMD: get_tweaks and get_categories
  Note over FE: every row shows "Checking"
  FE->>FE: listen for tweak-status
  FE->>CMD: get_statuses_stream
  CMD->>SCAN: spawn full scan
  CMD-->>FE: returns immediately
  loop each tweak, in parallel, in completion order
    SCAN-->>FE: tweak-status event with a stamp
  end
```

- The backend does not scan by itself; the frontend starts the scan once it has the catalog, and registers its event listener first so no status is missed.
- A status that arrives before its tweak is in the model is buffered and applied when the model loads.
- Every status carries a stamp; the frontend ignores a status older than the one it holds.

## User flows

### Apply

```mermaid
sequenceDiagram
  participant U as User
  participant Row as Tweak row
  participant Store as Stores
  participant CMD as apply_tweak
  U->>Row: pick an option
  alt high or critical risk
    Row->>U: confirm first
  end
  Row->>Store: stage the change (row shows it as pending)
  U->>Store: press Apply in the pending bar
  loop each staged tweak, one at a time
    Store->>CMD: apply_tweak(id, option)
    alt success
      CMD-->>Store: outcome with new status
      Store->>Store: adopt status, clear the staged change, note a reboot if needed
    else failure
      CMD-->>Store: error code and message
      Store->>CMD: get_tweak_status to pick up any Needs Attention
    end
  end
```

- Picking an option never applies it. One floating pending bar serves every view: it lists the staged changes (each can be unstaged), its Apply button runs them one after another without a further confirmation, and Discard clears them all.
- An `APP_EXITING` error stops the loop and reports how many changes were skipped, without a re-read, because nothing was touched.

### Restore, discard, keep

- **Restore** (the row's Restore action, shown when a history exists and the tweak does not need attention; while it does, the row's Needs Attention callout and the details window carry Restore instead; disabled unless the tweak is available) calls `restore_tweak` and adopts the returned status. On failure the store re-reads the status; when a `restore_failed` record was written, the button becomes "Retry restore" and the tweak needs attention.
- **Details window** (a large dialog opened from a row's Details action or by clicking the row) carries the same option control as the row, Restore and the favourite star in its header. Its body compares the options in a table: one row per setting the tweak touches, grouped by kind, one column per option with the current one highlighted and a staged one marked Pending, and at System Default a This PC now column with the live values and which options each one matches. Scripts sit in collapsible sections below the table. It also lists the snapshot entries (valid and invalid, with reasons) and lets the user discard one, after confirmation.
- **Keep current state** appears only while the tweak needs attention, and asks for confirmation.

## Frontend state

| Store | Holds |
| --- | --- |
| `tweaksData` | The tweak model with each tweak's status (`tweaksStore`), category metadata (`categoriesStore`), status stamps and buffered early statuses. |
| `tweakActions` | The apply, restore and keep-current-state actions (single and batch), with which tweaks are being changed and per-tweak errors. The details window discards snapshot entries through `snapshotHistory`. |
| `tweaksPending` | Staged option changes, and tweaks waiting for a reboot. |
| `system`, `elevation`, `boot` | System info, the elevation ceiling, and the launch sequence that fills them and starts the status stream. |
| `search`, `pageFilter` | The global search and a list page's in-place filter. |
| `apps` | App item views, presence statuses, per-app busy and error state, and the Remove, Install and Get in Store actions. Outside pending changes, snapshots and profiles. |

### What the user sees

| State | Shown as |
| --- | --- |
| Checking | "Checking" with a spinner on the row's meta line, nothing selected, until the first status arrives. |
| Active | The option is selected in the control, which is the only place it is named while nothing is staged (the meta line repeats it only beside a pending change); accent stripe on the row's left edge. |
| System Default | The control selects nothing and the meta line says System default, except a one-option switch, whose System default segment is selected and the meta line stays silent. The details window shows the observed values in its This PC now column. |
| Unknown | "Unknown" in warning colour on the meta line ("Unknown, needs admin" when elevation is the cause), the unreadable effects in its tooltip, nothing selected. |
| Unavailable | "Unavailable" on the meta line; control disabled, the reason in its tooltip. |
| Unavailable option | The option is labelled "(unavailable)" and cannot be chosen. |
| Blocked by availability | A meta-line label (Needs admin, Different account, Account unconfirmed, Not available on this PC) with the reason in its tooltip; control and Restore disabled. A category view with tweaks that need admin shows one notice with Restart as admin. |
| Needs Attention | Red stripe and a Needs Attention callout on the row with Restore ("Retry restore" after a failed restore) and Keep current state. |
| Shared | "Shared" on the meta line; its tooltip lists the shared settings held and by whom. |
| Residue | "Residue" on the meta line; its tooltip lists the residual settings. |
| Applying | Control disabled with a loading state. |

**Switch or dropdown.** One authored option renders as a segmented switch of System default and the option: the option stages it, System default unstages it or, once applied, restores the snapshot after a confirmation, since unlike every other segment it changes the system at once (disabled when no snapshot exists). Two render as a segmented switch, three or more as a dropdown.

**Rows and panes.** Every view lists tweaks as full-width rows, one per line. A row shows the title, the description, then a meta line (state, pending target, risk, a Warning toggle when the tweak authors a `warning:`, permission level, Restart, availability, Residue, Shared) ending in Restore, the favourite star and Details. The warning callout stays hidden until the toggle opens it or a change to that tweak is staged; the details window always shows it. The title and the control share the first line, the control capped at 45% of the row, and the description spans the full row beneath them; below 520px the control moves under the title, above the description. A switch label that does not fit truncates with an ellipsis and shows in full on hover; only the longest label, or one over 16 characters, gives up width. At 1400px of content width and above, category, Favorites and Snapshots views show an At a glance pane: applied progress by state, and lists of tweaks that need attention, are ready to apply, have an unknown state or wait for a restart, each opening the details window.

**System Default is never a target.** Only the one-option switch offers it, as a Restore; elsewhere the control shows no selection at System Default and the state line names it. A staged change is undone from the row's Undo link or the pending bar. The only way back to an earlier state is Restore (ADR-0003), which steps back one snapshot entry.

**Confirming risk.** Staging never asks for confirmation. Apply opens a review of every staged change (from → to, risk, restart, the authored warning) when any of them is high or critical risk; otherwise it applies at once. Restore on a high or critical risk tweak, Keep current state, and discarding a snapshot entry each confirm through one shared dialog. Bulk Restore buttons count only tweaks the user can restore right now.

**Search.** There is one search box, in the title bar (Ctrl+K). On a category, Favorites or Snapshots it carries a toggle naming the page; while pressed the search fuzzy-matches that page's rows in place, highlighting the matches as the global search does. Every visit to a list page starts scoped and unfiltered. Turning the toggle off (or Backspace in an empty box) makes the search global, carrying any typed text to the Search page, where the toggle stays: turning it back on returns to that page with the text as its filter. A scoped search with no matches offers Search everywhere. List pages have no filter row: the header holds the applied summary, Restore all, and on a category a Needs attention pill (shown only while something needs attention) that narrows the list to those tweaks.

## Profiles

The profile system is not wired to the tweak engine. There is no profile backend; the frontend profile API rejects every call. The profile import flow calls `rescan_after_elevation` when it finishes, so that command has no reachable caller today. [The v1 format record](../../spec/profile-v1.md) documents the archive format; [PROFILE_SYSTEM.md](../../PROFILE_SYSTEM.md) describes a profile backend the current code does not have.

## Traps

- **Availability is computed when the catalog loads** and is not refreshed by a rescan. Elevation changes are covered because Elevate relaunches the app.
- **The availability check runs before the lock**, and commands call the engine's variants that expect the lock to be held already, so it is never taken twice.
- **A re-read during another change fails** (the single-tweak status is refused while locked) and shows as "could not be re-read".
- **Unsupported tweaks are hidden only in the UI.** The catalog and scan events carry them; the engine refuses to apply one, since its surface is empty.
- **The tweak system uses two events**: `tweak-status` and `close-blocked`. App statuses are not streamed; `get_app_statuses` returns one scan.
