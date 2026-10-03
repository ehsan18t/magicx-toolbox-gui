<script lang="ts" module>
  import { Button, type ButtonVariants, Callout } from "$lib/components/ui";

  /** `compact` sums the failed items up inside a row; `panel` lists them. */
  type Density = "compact" | "panel";

  const DENSITY: Record<Density, { text: string; button: ButtonVariants["size"]; actions: string }> = {
    compact: { text: "leading-relaxed", button: "sm", actions: "mt-2" },
    panel: { text: "text-ui leading-relaxed", button: "md", actions: "mt-3" },
  };
</script>

<script lang="ts">
  import { elevationStore } from "$lib/stores/elevation.svelte";
  import { tweakActionsStore } from "$lib/stores/tweakActions.svelte";
  import type { AttentionItem, TweakWithStatus } from "$lib/types";
  import { attentionCause, ELEVATE_REMEDY, ensureSentence } from "$lib/utils/tweakPresentation";
  import RestoreButton from "./RestoreButton.svelte";

  interface Props {
    tweak: TweakWithStatus;
    density: Density;
  }

  let { tweak, density }: Props = $props();

  const def = $derived(tweak.definition);
  const status = $derived(tweak.status);
  const look = $derived(DENSITY[density]);

  let keeping = $state(false);

  function itemHint(item: AttentionItem): string {
    if (item.kind === "no_undo") return "(this one cannot be retried)";
    if (item.class === "busy") return "(retrying later may succeed)";
    if (item.class === "access_denied" && elevationStore.level === "User") return `(${ELEVATE_REMEDY})`;
    return "";
  }

  async function keepCurrent() {
    keeping = true;
    try {
      await tweakActionsStore.keepWithConfirm(def);
    } finally {
      keeping = false;
    }
  }
</script>

<!-- Needs Attention (ADR-0001/0002): the engine's own record, kept per tweak. -->
{#if status.attention}
  {@const items = status.attention.items}
  <Callout tone="error" {density} icon="mdi:alert-circle" class={look.text}>
    <div class="min-w-0 flex-1">
      <p class="m-0">
        <span class="font-semibold">Needs attention.</span>
        {ensureSentence(attentionCause(status.attention.reason))}
        <span class="text-foreground-muted">
          {#if density === "compact"}{items.map((item) => ensureSentence(item.message)).join(" ")}{/if}
          {status.hasHistory
            ? "Your snapshot is safe: restore it, or keep things as they are."
            : "No snapshot is left to restore, so you can only keep the current state."}
        </span>
      </p>
      {#if density === "panel" && items.length}
        <ul class="m-0 mt-2 list-none space-y-1 p-0">
          {#each items as item, i (`${item.effect}-${i}`)}
            <li class="text-xs text-foreground-muted">
              {#if item.effect}<span class="font-mono break-all text-foreground">{item.effect}</span>:{/if}
              {item.message}
              {itemHint(item)}
            </li>
          {/each}
        </ul>
      {/if}
      <div class="{look.actions} flex flex-wrap gap-2">
        {#if status.hasHistory}<RestoreButton {tweak} size={look.button} />{/if}
        <!-- Consent stays reachable whenever a record exists, entries left or not (ADR-0002). -->
        <Button
          size={look.button}
          icon="mdi:check"
          loading={keeping}
          disabled={tweakActionsStore.isRunning(def.id)}
          onclick={keepCurrent}
        >
          Keep current state
        </Button>
      </div>
    </div>
  </Callout>
{/if}
