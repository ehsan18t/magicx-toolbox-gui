<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { filterStore, pendingRebootStore } from "$lib/stores/tweaks.svelte";
  import { expand } from "$lib/utils/motion";

  let showDetails = $state(false);

  const rebootCount = $derived(pendingRebootStore.count);
  const rebootTweaks = $derived(filterStore.pendingRebootTweaks);
</script>

{#if rebootCount > 0}
  <div class="shrink-0 border-b border-border bg-info/8 px-4 py-2" transition:expand>
    <div class="flex flex-wrap items-center gap-x-3 gap-y-1">
      <Icon icon="mdi:restart" width="16" class="shrink-0 text-info" />
      <p class="m-0 min-w-0 flex-1 text-[13px]">
        <span class="font-semibold">Restart required.</span>
        <span class="text-foreground-muted">
          {rebootCount === 1 ? "1 change takes" : `${rebootCount} changes take`} effect after you restart Windows.
        </span>
      </p>
      <div class="flex shrink-0 items-center gap-1">
        <button
          type="button"
          class="h-7 cursor-pointer rounded-md px-2 text-xs font-medium text-foreground hover:bg-muted"
          aria-expanded={showDetails}
          onclick={() => (showDetails = !showDetails)}
        >
          {showDetails ? "Hide list" : "Show which"}
        </button>
        <button
          type="button"
          class="flex h-7 w-7 cursor-pointer items-center justify-center rounded-md text-foreground-muted hover:bg-muted hover:text-foreground"
          aria-label="Dismiss restart notice"
          onclick={() => pendingRebootStore.clear()}
          use:tooltip={"Dismiss (changes still apply after restart)"}
        >
          <Icon icon="mdi:close" width="16" />
        </button>
      </div>
    </div>
    {#if showDetails}
      <ul class="m-0 mt-1.5 flex list-none flex-wrap gap-x-4 gap-y-1 p-0 pl-7" transition:expand>
        {#each rebootTweaks as tweak (tweak.definition.id)}
          <li class="min-w-0 text-xs wrap-break-word text-foreground-muted">{tweak.definition.name}</li>
        {/each}
      </ul>
    {/if}
  </div>
{/if}
