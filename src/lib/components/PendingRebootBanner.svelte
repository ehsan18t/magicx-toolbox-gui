<script lang="ts">
  import { pendingRebootCount, pendingRebootStore, pendingRebootTweaks } from "$lib/stores/tweaks";
  import Icon from "./Icon.svelte";

  let showDetails = $state(false);
</script>

{#if $pendingRebootCount > 0}
  <div
    class="mb-4 flex items-center justify-between gap-4 rounded-lg border border-[hsl(24_94%_50%/0.3)] bg-linear-to-br from-[hsl(24_94%_50%/0.15)] to-[hsl(45_93%_47%/0.1)] px-4 py-3"
  >
    <div class="flex items-center gap-3">
      <div
        class="flex h-10 w-10 items-center justify-center rounded-lg bg-[hsl(24_94%_50%/0.2)] text-[hsl(24_94%_50%)]"
      >
        <Icon icon="mdi:restart-alert" width="24" />
      </div>
      <div class="flex flex-col gap-0.5">
        <span class="text-sm font-semibold text-foreground">Restart Required</span>
        <span class="text-xs text-foreground-muted">
          {$pendingRebootCount} tweak{$pendingRebootCount === 1 ? "" : "s"} need a system restart to take effect
        </span>
      </div>
    </div>

    <div class="flex items-center gap-2">
      <button
        class="flex cursor-pointer items-center gap-1 rounded-md border-0 bg-[hsl(var(--muted))] px-3 py-1.5 text-xs font-medium text-foreground transition-colors duration-150 hover:bg-[hsl(var(--muted)/0.8)]"
        onclick={() => (showDetails = !showDetails)}
      >
        <Icon icon={showDetails ? "mdi:chevron-up" : "mdi:chevron-down"} width="16" />
        {showDetails ? "Hide" : "Details"}
      </button>
      <button
        class="flex h-7 w-7 cursor-pointer items-center justify-center rounded border-0 bg-transparent text-foreground-muted transition-all duration-150 hover:bg-[hsl(var(--muted))] hover:text-foreground"
        onclick={() => pendingRebootStore.clear()}
        title="Dismiss (changes still apply after restart)"
      >
        <Icon icon="mdi:close" width="16" />
      </button>
    </div>
  </div>

  {#if showDetails}
    <div class="-mt-2 mb-4 rounded-md border border-border bg-[hsl(var(--muted)/0.3)] px-4 py-3">
      <ul class="m-0 flex list-none flex-col gap-1.5 p-0">
        {#each $pendingRebootTweaks as tweak (tweak.definition.id)}
          <li class="flex items-center gap-2 text-xs text-foreground/80">
            <Icon icon="mdi:restart" width="14" class="text-[hsl(24_94%_50%)]" />
            {tweak.definition.name}
          </li>
        {/each}
      </ul>
    </div>
  {/if}
{/if}
