use tauri::App;

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Register the handle debug events are emitted through. Held in debug.rs rather
    // than threaded through the apply chain as a parameter -- see the note on
    // DEBUG_APP there. Must happen before anything that might emit.
    crate::debug::set_debug_app(app.handle().clone());

    // The old pipeline's startup stale-snapshot cleanup was schema/store-specific to
    // `services::backup` (deleted with the hard cutover, spec §12: old on-disk snapshots are
    // invalidated by the schema-version bump regardless). The new engine's equivalent
    // (an SD-snapshot/stale-entry startup sweep over `tweaks::snapshot::SnapshotStore`) is
    // deferred future work (spec §14), not yet wired to any command surface -- nothing to call
    // here yet.

    Ok(())
}
