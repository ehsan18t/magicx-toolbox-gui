# Profile System UI Implementation Guide

> **Status**: Backend complete, UI pending
> **Last Updated**: December 21, 2025
> **Branch**: `revamp-backup-system`

## What's Already Done

### Backend (Complete ✅)
- `src-tauri/src/services/profile/` - Export, import, validation, archive handling
- `src-tauri/src/commands/profile.rs` - Tauri IPC commands
- `src-tauri/src/models/profile.rs` - Data models

### Frontend API (Complete ✅)
- `src/lib/api/profile.ts` - TypeScript API with full types

### Documentation (Complete ✅)
- `docs/PROFILE_SYSTEM.md` - System overview and future plans
- `docs/ARCHITECTURE.md` - Updated with profile system references

---

## What Needs to Be Built

### 1. Profile Store (`src/lib/stores/profile.svelte.ts`)

Create a Svelte 5 runes-based store following the pattern in existing stores like `tweaks.svelte.ts`.

**State to track:**
- `isExporting: boolean` - Export operation in progress
- `isImporting: boolean` - Import operation in progress
- `isApplying: boolean` - Apply operation in progress
- `currentProfile: ConfigurationProfile | null` - Loaded profile
- `validation: ProfileValidation | null` - Validation result
- `exportError: string | null`
- `importError: string | null`
- `applyError: string | null`

**Actions to implement:**
- `exportProfile(name, description, tweakIds, includeSystemState)` - Calls API, opens save dialog
- `importProfile()` - Opens file picker, loads and validates profile
- `applyProfile(options)` - Applies validated profile
- `clearProfile()` - Resets state

**Reference existing stores** in `src/lib/stores/` for the getter-based pattern used in this codebase.

---

### 2. Export Modal (`src/lib/components/modals/ProfileExportModal.svelte`)

**Trigger**: Add "Export Profile" button to the app (location TBD - could be in settings or header menu).

**Modal Contents:**

```

┌─────────────────────────────────────────────────────────────────┐
│                        EXPORT WIZARD                            │
├─────────────────────────────────────────────────────────────────┤
│ Step 1: Select Tweaks                                           │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ ☑ Select All Applied Tweaks (23)                            │ │
│ │ ☐ Include System Default Selections                         │ │
│ ├─────────────────────────────────────────────────────────────┤ │
│ │ Privacy (5 tweaks)                                          │ │
│ │   ☑ Disable Telemetry            [Disabled]                 │ │
│ │   ☑ Disable Activity History     [Disabled]                 │ │
│ │   ☑ Disable Advertising ID       [Disabled]                 │ │
│ │   ...                                                        │ │
│ │ Performance (8 tweaks)                                       │ │
│ │   ☑ Icon Cache Size              [4 MB]                     │ │
│ │   ☑ Disable SuperFetch           [Disabled]                 │ │
│ │   ...                                                        │ │
│ └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ Step 2: Profile Details                                         │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ Name: [My Gaming Setup                              ]       │ │
│ │ Description: [Optimized for gaming, privacy-focused ]       │ │
│ │                                                              │ │
│ │ ☑ Include system state snapshot (for conflict detection)    │ │
│ └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                              [Cancel]  [Export to File]         │
└─────────────────────────────────────────────────────────────────┘
```

**Behavior:**
1. Pre-select all currently applied tweaks
2. Allow user to uncheck tweaks they don't want to export
3. On export: Call Tauri `save` dialog to pick location, then call `profile_export` command
4. Show success toast or error message

**Components to use:**
- `Modal` from `$lib/components/ui`
- `Button` from `$lib/components/ui`
- `Switch` for checkboxes (or create a Checkbox component)
- Get applied tweaks from `tweaksStore`

---

### 3. Import Modal (`src/lib/components/modals/ProfileImportModal.svelte`)

This is a **multi-step wizard modal**.

#### Step 1: File Selection & Validation

```
┌─────────────────────────────────────────────────────────┐
│  Import Configuration Profile                      [X]  │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │                                                 │   │
│  │     📁 Drop .mgx file here or click to browse  │   │
│  │                                                 │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  [Browse Files...]                                      │
│                                                         │
├─────────────────────────────────────────────────────────┤
│                                        [Cancel]         │
└─────────────────────────────────────────────────────────┘
```

After file is selected and validated:

#### Step 2: Validation Results & Preview

```
┌─────────────────────────────────────────────────────────┐
│  Import Configuration Profile                      [X]  │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  📄 "Gaming Optimizations" by User on Windows 11        │
│  Created: Dec 15, 2025 • 8 tweaks                       │
│                                                         │
│  ─────────────────────────────────────────────────────  │
│                                                         │
│  ⚠️ 2 Warnings                                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │ • Windows version mismatch (profile: Win11,     │   │
│  │   current: Win10) - some tweaks may differ      │   │
│  │ • "Disable Telemetry" schema changed - will     │   │
│  │   use current version                           │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  ❌ 1 Error (will be skipped)                           │
│  ┌─────────────────────────────────────────────────┐   │
│  │ • "Old Tweak Name" not found in current app     │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  ─────────────────────────────────────────────────────  │
│                                                         │
│  Changes to Apply (7 tweaks):                           │
│                                                         │
│  ☑ Disable Telemetry                                   │
│     Current: Default → New: Disabled                    │
│                                                         │
│  ☑ Game Mode Optimizations                             │
│     Current: Default → New: Enabled                     │
│     ⚡ 3 registry changes, 1 service change            │
│                                                         │
│  ☐ Search Indexing (already applied - same setting)    │
│                                                         │
│  ... (scrollable)                                       │
│                                                         │
├─────────────────────────────────────────────────────────┤
│  ☑ Create restore points before applying               │
│  ☑ Skip already-applied tweaks                         │
│  ☑ Rollback all on error                               │
├─────────────────────────────────────────────────────────┤
│              [Cancel]  [Back]  [Apply 7 Tweaks]         │
└─────────────────────────────────────────────────────────┘
```

#### Step 3: Apply Progress

```
┌─────────────────────────────────────────────────────────┐
│  Applying Profile...                                    │
├─────────────────────────────────────────────────────────┤
│                                                         │
│   [████████████░░░░░░░░░░░░░░░░░░░] 4/7 tweaks          │
│                                                         │
│  ✅ Disable Telemetry                                   │
│  ✅ Game Mode Optimizations                             │
│  ✅ Disable Activity History                            │
│  ⏳ Applying: Power Plan Settings...                    │
│     Setting HKLM\SOFTWARE\Microsoft\Windows\...         │
│  ⬚ Disable Cortana                                      │
│  ⬚ Disable Windows Tips                                 │
│  ⬚ Disable Consumer Features                            │
│                                                         │
│ Time elapsed: 00:12                                     │
│ Estimated remaining: 00:08                              │
├─────────────────────────────────────────────────────────┤
│  Do not close this window                               │
└─────────────────────────────────────────────────────────┘
```

#### Step 4: Results

```
┌─────────────────────────────────────────────────────────┐
│  Profile Applied                                   [X]  │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ✅ Successfully applied 7 tweaks                       │
│                                                         │
│  ⚠️ System restart recommended for:                     │
│  • Power Plan Settings                                  │
│  • Game Mode Optimizations                              │
│                                                         │
├─────────────────────────────────────────────────────────┤
│                    [Restart Later]  [Restart Now]       │
└─────────────────────────────────────────────────────────┘
```

---

### 4. Entry Points (Where to Add Buttons)

#### Option A: Header/Toolbar
Add to the main layout header area, possibly as a dropdown menu:
- `src/lib/components/layout/Header.svelte` (if exists) or similar

#### Option B: Settings View
Add a "Profiles" section to settings:
- Look in `src/lib/components/settings/` or `src/lib/components/views/`

#### Option C: Sidebar Action
Add as an action button in the sidebar

**Recommendation**: Start with a simple approach - add two buttons somewhere accessible:
- "Export Profile" - Opens export modal
- "Import Profile" - Opens import modal

---

### 5. File Dialog Integration

Use Tauri's dialog API for file picking:

```typescript
import { save, open } from '@tauri-apps/plugin-dialog';

// For export - save dialog
const filePath = await save({
  defaultPath: 'my-profile.mgx',
  filters: [{ name: 'MagicX Profile', extensions: ['mgx'] }]
});

// For import - open dialog
const filePath = await open({
  filters: [{ name: 'MagicX Profile', extensions: ['mgx'] }]
});
```

**Note**: Check if `@tauri-apps/plugin-dialog` is already installed. If not, add it:
```bash
bun add @tauri-apps/plugin-dialog
```
And enable in `tauri.conf.json` capabilities.

---

### 6. Toast Notifications

Use the existing toast system for success/error messages:
- Check `src/lib/stores/toast.svelte.ts` for the toast store
- Use it to show "Profile exported successfully" etc.

---

## Implementation Order

1. **Store** - Create `profile.svelte.ts` store first
2. **Export Modal** - Simpler, good starting point
3. **Entry Point** - Add Export button somewhere
4. **Import Modal** - More complex, multi-step wizard
5. **Polish** - Error handling, loading states, edge cases

---

## API Reference (Already Implemented)

From `src/lib/api/profile.ts`:

```typescript
// Export a profile
exportProfile(filePath, name, selections, options?)

// Import and validate
importProfile(filePath) → [ConfigurationProfile, ProfileValidation]

// Validate existing profile
validateProfile(profile) → ProfileValidation

// Apply validated profile
applyProfile(profile, validation, options?) → ProfileApplyResult
```

See the file for full type definitions.

---

## Design Notes

- Follow existing UI patterns in the codebase
- Use components from `$lib/components/ui/`
- Match the app's color scheme and spacing
- Consider dark/light mode if the app supports it
- Make modals dismissible with Escape key
- Show loading spinners during async operations

---

## Testing Checklist

After implementation, verify:

- [ ] Export with all tweaks selected
- [ ] Export with some tweaks deselected
- [ ] Export with system state included
- [ ] Import valid profile from same machine
- [ ] Import profile from different Windows version (shows warning)
- [ ] Import profile with missing tweaks (shows error, skips them)
- [ ] Apply profile successfully
- [ ] Apply profile with rollback on error
- [ ] Cancel mid-apply (if supported)
- [ ] Error states display correctly
- [ ] Toasts appear for success/failure
