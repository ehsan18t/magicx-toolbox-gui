<script lang="ts">
  import { tweaksStore } from "$lib/stores/tweaksData.svelte";
  import TweakCollectionView from "./TweakCollectionView.svelte";
</script>

<TweakCollectionView
  title="Snapshots"
  description="Tweaks with a saved snapshot. Each Restore steps a tweak back to the state saved before its last change."
  tweaks={tweaksStore.withSnapshot}
  noun="snapshots"
  countLabel="with snapshots"
  appliedLabel="currently applied"
  restoreTitle="Restore all snapshots?"
  empty={{
    icon: "mdi:backup-restore",
    title: "No snapshots yet",
    description: "Applying a tweak saves the state it replaces as a snapshot, so you can restore it later.",
  }}
/>
