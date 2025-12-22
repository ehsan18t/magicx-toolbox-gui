# Profile System - Configuration Export/Import

> Comprehensive guide for the profile-based configuration management system.

## Overview

The Profile System allows users to export their tweak configurations and import them on other machines or after reinstalling Windows. Unlike the internal snapshot system (which captures pre-apply state for rollback), profiles capture the user's **intent** - what they want their system configured as.

## Key Concepts

| Concept                   | Purpose                          | Storage                        |
| ------------------------- | -------------------------------- | ------------------------------ |
| **Internal Snapshots**    | Atomic rollback for single tweak | `snapshots/{id}.json` (hidden) |
| **Configuration Profile** | User's desired tweak selections  | `.mgx` archive file            |
| **System State Snapshot** | Full system state for validation | Optional in `.mgx` archive     |

## Profile Archive Format

Profiles use the `.mgx` extension (ZIP archive):

```
profile.mgx
├── manifest.json          # Version, checksums, metadata
├── profile.json           # Tweak selections
├── system_state.json      # Optional: state at export time
└── signatures/
    └── sha256.txt         # Integrity verification
```

## Schema Versioning

Profiles include a schema version for forward/backward compatibility:

- **Version 1**: Initial release
- Migration functions handle upgrading old profiles automatically
- Tweak ID aliases support renaming tweaks without breaking profiles

## Windows Version Compatibility

The system handles cross-version scenarios:

| Scenario      | Behavior                                                   |
| ------------- | ---------------------------------------------------------- |
| Same version  | Full compatibility                                         |
| Win10 → Win11 | Validates each tweak, warns about version-specific changes |
| Win11 → Win10 | Skips Win11-only tweaks with clear messaging               |
| Unknown tweak | Skips with warning (tweak removed from app)                |

## Validation System

Before applying a profile, the system validates:

1. **Tweak Existence**: Does the tweak ID still exist? (with alias fallback)
2. **Option Validity**: Is the selected option index valid? (with content-hash fallback)
3. **Windows Compatibility**: Does this tweak apply to the target OS version?
4. **Permission Requirements**: Does the user have necessary privileges?
5. **Resource Availability**: Do required services/tasks exist?

## User Workflows

### Export Flow

1. Select tweaks to include (defaults to all applied)
2. Provide profile name and optional description
3. Choose whether to include system state snapshot
4. Save `.mgx` file

### Import Flow

1. Load `.mgx` file
2. View validation results (compatible, warnings, errors)
3. Optionally skip specific tweaks
4. Review changes preview
5. Apply with progress feedback

## API Reference

### Export Commands

```typescript
// Export to file
await exportProfileToFile({
  name: "My Gaming Setup",
  description: "Optimized for gaming",
  tweakIds: ["disable_telemetry", "icon_cache_size"],
  includeSystemState: true
}, "/path/to/profile.mgx");
```

### Import Commands

```typescript
// Validate before applying
const validation = await validateProfileFile("/path/to/profile.mgx");

// Apply with options
const result = await applyProfile(profileData, {
  skipTweakIds: ["incompatible_tweak"],
  createRestorePoint: true
});
```

## Security

- Profiles only contain tweak IDs and option indices
- Actual system changes come from the app's tweak definitions
- SHA-256 checksums detect tampering
- Same permission model as manual tweak application

---

## Future Enhancements

The following features are planned for future releases:

| Feature                   | Description                                            | Priority | Status  |
| ------------------------- | ------------------------------------------------------ | -------- | ------- |
| **Windows Restore Point** | Automatic Windows restore point before batch apply     | P1       | Planned |
| **Cloud Sync**            | Sync profiles via GitHub Gist / OneDrive               | P2       | Planned |
| **Diff View**             | Visual comparison between two profiles                 | P2       | Planned |
| **Profile Templates**     | Pre-built profiles (Gaming, Privacy, Minimal)          | P2       | Planned |
| **Profile Library**       | Community-shared profile repository                    | P3       | Planned |
| **Scheduled Apply**       | Apply profile on schedule (e.g., "gaming mode" toggle) | P3       | Planned |
| **Profile Versioning**    | Track changes to a profile over time                   | P3       | Planned |
| **Selective Sync**        | Sync only specific categories across machines          | P3       | Planned |

### Implementation Notes

#### Windows Restore Point (P1)
- Use `SRSetRestorePoint` Windows API via Rust FFI
- Prompt user before large batch operations
- Store restore point ID for potential automated rollback

#### Cloud Sync (P2)
- GitHub Gist: Public or secret gists for sharing
- OneDrive: Native Windows integration
- Conflict resolution for multi-machine scenarios

#### Profile Templates (P2)
- Ship with app as embedded resources
- Categories: Gaming, Privacy, Performance, Minimal, Developer
- One-click apply with customization option

---

## Technical Implementation

See source files:
- `src-tauri/src/models/profile.rs` - Data models
- `src-tauri/src/services/profile/` - Core logic
- `src-tauri/src/commands/profile.rs` - Tauri commands
- `src/lib/components/profile/` - UI components
