<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { filterStore, pendingRebootStore } from "$lib/stores/tweaks.svelte";

  let showDetails = $state(false);

  // Derived values from stores
  const rebootCount = $derived(pendingRebootStore.count);
  const rebootTweaks = $derived(filterStore.pendingRebootTweaks);
</script>

{#if rebootCount > 0}
  <div
    class="mb-4 flex w-full min-w-0 items-center justify-between gap-4 border-b border-warning/30 bg-linear-to-br from-warning/15 to-warning/10 px-4 py-3"
  >
    <div class="flex min-w-0 flex-1 items-center gap-3">
      <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-warning/20 text-warning">
        <Icon icon="mdi:restart-alert" width="24" />
      </div>
      <div class="flex min-w-0 flex-col gap-0.5">
        <span class="text-sm font-semibold text-foreground">Restart Required</span>
        <span class="text-xs wrap-break-word text-foreground-muted">
          {rebootCount} tweak{rebootCount === 1 ? "" : "s"} need a system restart to take effect
        </span>
      </div>
    </div>

    <div class="flex items-center gap-2">
      <button
        class="bg-muted hover:bg-muted/80 flex cursor-pointer items-center gap-1 rounded-md border-0 px-3 py-1.5 text-xs font-medium text-foreground transition-colors duration-150"
        onclick={() => (showDetails = !showDetails)}
      >
        <Icon icon={showDetails ? "mdi:chevron-up" : "mdi:chevron-down"} width="16" />
        {showDetails ? "Hide" : "Details"}
      </button>
      <button
        class="hover:bg-muted flex h-7 w-7 cursor-pointer items-center justify-center rounded border-0 bg-transparent text-foreground-muted transition-all duration-150 hover:text-foreground"
        onclick={() => pendingRebootStore.clear()}
        use:tooltip={"Dismiss (changes still apply after restart)"}
      >
        <Icon icon="mdi:close" width="16" />
      </button>
    </div>
  </div>
  {#if showDetails}
    <div class="bg-muted/30 mx-2 -mt-2 mb-4 rounded-md border border-border px-4 py-3">
      <ul class="m-0 flex list-none flex-col gap-1.5 p-0">
        {#each rebootTweaks as tweak (tweak.definition.id)}
          <li class="flex min-w-0 items-start gap-2 text-xs text-foreground/80">
            <Icon icon="mdi:restart" width="14" class="shrink-0 text-warning" />
            <span class="min-w-0 wrap-break-word">{tweak.definition.name}</span>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
{/if}
