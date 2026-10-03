<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Button, IconButton } from "$lib/components/ui";
  import { TONE_TINT } from "$lib/design";
  import { pendingRebootStore } from "$lib/stores/tweaksPending.svelte";
  import { reclaimFocus } from "$lib/utils/focus";
  import { plural } from "$lib/utils/format";
  import { expand } from "$lib/utils/motion";
  import { tick } from "svelte";
  import type { Attachment } from "svelte/attachments";

  // app.css's toast offset reads it, so toasts sit below the banner however it wraps.
  const HEIGHT_VAR = "--reboot-banner-height";

  let showDetails = $state(false);

  const rebootCount = $derived(pendingRebootStore.count);

  async function dismiss() {
    pendingRebootStore.clear();
    await tick();
    reclaimFocus();
  }

  const publishHeight: Attachment<HTMLElement> = (node) => {
    const rootStyle = document.documentElement.style;
    const observer = new ResizeObserver(() => rootStyle.setProperty(HEIGHT_VAR, `${node.offsetHeight}px`));
    observer.observe(node);
    return () => {
      observer.disconnect();
      rootStyle.removeProperty(HEIGHT_VAR);
    };
  };
</script>

{#if rebootCount > 0}
  <div class="shrink-0 border-b px-4 py-2 {TONE_TINT.info}" transition:expand {@attach publishHeight}>
    <div class="flex flex-wrap items-center gap-x-3 gap-y-1">
      <Icon icon="mdi:restart" size="md" class="shrink-0 text-info" />
      <p class="m-0 min-w-0 flex-1 text-ui">
        <span class="font-semibold">Restart required.</span>
        <span class="text-foreground-muted">
          {plural(rebootCount, "change takes", "changes take")} effect after you restart Windows.
        </span>
      </p>
      <div class="flex shrink-0 items-center gap-1">
        <Button
          variant="ghost"
          size="sm"
          tone="foreground"
          class="px-2"
          aria-expanded={showDetails}
          onclick={() => (showDetails = !showDetails)}
        >
          {showDetails ? "Hide list" : "Show which"}
        </Button>
        <IconButton
          icon="mdi:close"
          size="sm"
          label="Dismiss restart notice"
          tooltip="Dismiss (changes still apply after restart)"
          onclick={dismiss}
        />
      </div>
    </div>
    {#if showDetails}
      <ul class="m-0 mt-1.5 flex list-none flex-wrap gap-x-4 gap-y-1 p-0 pl-7" transition:expand>
        {#each pendingRebootStore.tweaks as tweak (tweak.definition.id)}
          <li class="min-w-0 text-xs wrap-break-word text-foreground-muted">{tweak.definition.name}</li>
        {/each}
      </ul>
    {/if}
  </div>
{/if}
