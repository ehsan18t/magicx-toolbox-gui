<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { openTweakDetailsModal } from "$lib/stores/tweakDetailsModal.svelte";
  import {
    applyPendingChanges,
    loadingStore,
    pendingChangesStore,
    tweaksStore,
    unstageChange,
  } from "$lib/stores/tweaks.svelte";
  import { slide } from "svelte/transition";

  let expanded = $state(false);
  let applying = $state(false);

  const count = $derived(pendingChangesStore.count);
  const items = $derived(
    Array.from(pendingChangesStore.all.values()).map((change) => ({
      change,
      tweak: tweaksStore.getById(change.tweakId),
    })),
  );
  const needsReboot = $derived(items.some((i) => i.tweak?.definition.requires_reboot));
  const busy = $derived(applying || loadingStore.isAnyLoading);

  async function apply() {
    applying = true;
    try {
      await applyPendingChanges();
    } finally {
      applying = false;
    }
  }
</script>

{#if count > 0}
  <div
    class="pointer-events-none absolute inset-x-0 bottom-0 z-30 flex justify-center px-4 pb-4"
    transition:slide={{ duration: 180 }}
  >
    <div
      class="pointer-events-auto w-full max-w-xl overflow-hidden rounded-lg border border-border bg-elevated shadow-flyout"
      role="region"
      aria-label="Pending changes"
    >
      {#if expanded}
        <ul
          class="m-0 max-h-56 list-none overflow-y-auto border-b border-border p-1"
          transition:slide={{ duration: 150 }}
        >
          {#each items as { change, tweak } (change.tweakId)}
            <li class="flex items-center gap-2 rounded px-2 py-1.5 hover:bg-muted">
              <button
                type="button"
                class="min-w-0 flex-1 cursor-pointer truncate text-left text-[13px]"
                onclick={() => openTweakDetailsModal(change.tweakId)}
              >
                <span class="text-foreground">{tweak?.definition.name ?? change.tweakId}</span>
                <span class="text-foreground-subtle"> → {change.optionLabel}</span>
              </button>
              <button
                type="button"
                class="flex h-6 w-6 shrink-0 cursor-pointer items-center justify-center rounded text-foreground-muted hover:bg-muted hover:text-foreground"
                aria-label="Unstage {tweak?.definition.name ?? change.tweakId}"
                onclick={() => unstageChange(change.tweakId)}
              >
                <Icon icon="mdi:close" width="14" />
              </button>
            </li>
          {/each}
        </ul>
      {/if}

      <div class="flex flex-wrap items-center gap-x-3 gap-y-2 px-3 py-2.5">
        <button
          type="button"
          class="flex min-w-0 flex-1 cursor-pointer items-center gap-2 text-left"
          aria-expanded={expanded}
          onclick={() => (expanded = !expanded)}
        >
          <span
            class="flex h-6 min-w-6 shrink-0 items-center justify-center rounded-full bg-warning px-1.5 text-xs font-bold text-warning-foreground tabular-nums"
          >
            {count}
          </span>
          <span class="min-w-0">
            <span class="block truncate text-[13px] font-semibold text-foreground">
              {count === 1 ? "1 change" : `${count} changes`} ready to apply
            </span>
            {#if needsReboot}
              <span class="block truncate text-xs text-foreground-muted">Some need a restart to take effect</span>
            {/if}
          </span>
          <Icon
            icon={expanded ? "mdi:chevron-down" : "mdi:chevron-up"}
            width="16"
            class="shrink-0 text-foreground-muted"
          />
        </button>

        <div class="ml-auto flex shrink-0 items-center gap-2">
          <button
            type="button"
            class="h-8 cursor-pointer rounded-md border border-border bg-secondary px-3 text-[13px] font-medium hover:bg-secondary-hover disabled:cursor-not-allowed disabled:opacity-50"
            disabled={busy}
            onclick={() => pendingChangesStore.clearAll()}
          >
            Discard
          </button>
          <button
            type="button"
            class="inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md bg-accent px-3.5 text-[13px] font-semibold text-accent-foreground hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-60"
            disabled={busy}
            onclick={apply}
          >
            {#if applying}<Icon icon="mdi:loading" width="14" class="animate-spin" />{/if}
            Apply
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
