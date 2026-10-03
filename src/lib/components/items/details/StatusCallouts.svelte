<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Callout } from "$lib/components/ui";
  import { tweaksStore } from "$lib/stores/tweaksData.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { availabilityTitle, heldSharedText, residueText, UNKNOWN_ICON } from "$lib/utils/tweakPresentation";
  import AttentionNotice from "../AttentionNotice.svelte";

  let { tweak }: { tweak: TweakWithStatus } = $props();

  const def = $derived(tweak.definition);
  const status = $derived(tweak.status);
</script>

{#if def.warning}
  <Callout tone="warning" density="panel" icon="mdi:alert">
    <p class="m-0 text-ui leading-relaxed text-foreground">{def.warning}</p>
  </Callout>
{/if}

{#if def.availability.state !== "available"}
  <Callout tone="warning" density="panel" icon="mdi:shield-lock-outline">
    <p class="m-0 text-ui leading-relaxed">
      <span class="font-semibold">{availabilityTitle(def.availability)}.</span>
      <span class="text-foreground-muted">{def.availability.reason}</span>
    </p>
  </Callout>
{/if}

{#if status.state === "unavailable" && status.unavailableReason}
  <Callout tone="surface" density="panel" icon="mdi:cancel">
    <p class="m-0 text-ui text-foreground-muted">{status.unavailableReason}</p>
  </Callout>
{/if}

{#if status.state === "unknown" && status.unknownReasons.length > 0}
  <Callout tone="warning" density="panel" icon={UNKNOWN_ICON}>
    <div class="min-w-0">
      <p class="m-0 mb-1.5 text-ui font-semibold">Could not determine state</p>
      <ul class="m-0 list-none space-y-1 p-0">
        {#each status.unknownReasons as reason, i (`${reason.effect}-${i}`)}
          <li class="text-xs text-foreground-muted">
            <span class="font-mono break-all text-foreground">{reason.effect}</span>
            ({reason.cause}{reason.needs_elevation ? ", restart as admin to resolve" : ""})
          </li>
        {/each}
      </ul>
    </div>
  </Callout>
{/if}

<AttentionNotice {tweak} density="panel" />

<!-- One icon per note, so not the Callout's single icon. -->
{#if status.residues.length > 0 || status.heldShared.length > 0}
  <Callout tone="surface" density="panel" class="space-y-2 text-ui text-foreground-muted">
    {#if status.residues.length > 0}
      <p class="m-0 flex gap-2">
        <Icon icon="mdi:information-outline" size="md" class="mt-0.5 shrink-0 text-info" />
        <span>{residueText(status)}</span>
      </p>
    {/if}
    {#if status.heldShared.length > 0}
      <p class="m-0 flex gap-2">
        <Icon icon="mdi:link-variant" size="md" class="mt-0.5 shrink-0" />
        <span>{heldSharedText(status, tweaksStore.tweak)}</span>
      </p>
    {/if}
  </Callout>
{/if}
