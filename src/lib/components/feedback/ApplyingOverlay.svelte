<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { loadingStore } from "$lib/stores/tweaks.svelte";

  const isApplying = $derived(loadingStore.isAnyLoading);

  let visible = $state(false);
  let hideTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    const applying = isApplying;

    if (applying) {
      if (hideTimer) {
        clearTimeout(hideTimer);
        hideTimer = null;
      }
      visible = true;
      return;
    }

    if (hideTimer) {
      clearTimeout(hideTimer);
    }

    // Small delay prevents flicker during sequential/batch operations.
    hideTimer = setTimeout(() => {
      visible = false;
      hideTimer = null;
    }, 250);
  });

  $effect(() => {
    return () => {
      if (hideTimer) {
        clearTimeout(hideTimer);
        hideTimer = null;
      }
    };
  });
</script>

{#if visible}
  <div
    class="fixed inset-x-0 top-12 bottom-0 z-1000 flex items-center justify-center bg-black/40 p-4"
    role="presentation"
    aria-busy="true"
  >
    <div class="w-full max-w-sm rounded-xl border border-border bg-elevated px-6 py-5 shadow-dialog">
      <div class="flex items-center gap-3">
        <span class="inline-flex animate-spin text-accent">
          <Icon icon="mdi:loading" width="24" class="text-accent" />
        </span>
        <div class="min-w-0">
          <div class="text-base font-semibold text-foreground">Applying tweaks…</div>
          <div class="mt-0.5 text-sm text-foreground-muted">Please wait and do not close the app.</div>
        </div>
      </div>
    </div>
  </div>
{/if}
