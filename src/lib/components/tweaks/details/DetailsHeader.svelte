<script lang="ts" module>
  import type { IconName } from "$lib/components/shared";

  const STANDARD_USER: { name: string; icon: IconName } = { name: "Standard user", icon: "mdi:account" };
</script>

<script lang="ts">
  import { IconButton } from "$lib/components/ui";
  import { pendingChangesStore } from "$lib/stores/tweaksPending.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { permissionInfoFor, RISK_INFO, RISK_TONE, stateSummary } from "$lib/utils/tweakPresentation";
  import FavoriteButton from "../FavoriteButton.svelte";
  import MetaItem from "../MetaItem.svelte";
  import TweakControl from "../TweakControl.svelte";
  import RestoreButton from "./RestoreButton.svelte";

  interface Props {
    tweak: TweakWithStatus;
    titleId: string;
    onclose: () => void;
  }

  let { tweak, titleId, onclose }: Props = $props();

  const def = $derived(tweak.definition);
  const status = $derived(tweak.status);
  const summary = $derived(stateSummary(status));
  const risk = $derived(RISK_INFO[def.riskLevel]);
  const permission = $derived(permissionInfoFor(def.requiredLevel));
  const pendingChange = $derived(pendingChangesStore.change(def.id));
</script>

<header class="shrink-0 border-b border-border px-6 pt-5 pb-4">
  <div class="flex items-start gap-4">
    <div class="min-w-0 flex-1">
      <h2 id={titleId} class="m-0 font-display text-xl leading-snug font-semibold wrap-break-word">{def.name}</h2>
      <p class="m-0 mt-1 text-ui leading-relaxed text-foreground-muted">{def.description}</p>
    </div>
    <div class="flex shrink-0 items-center gap-0.5">
      <FavoriteButton tweakId={def.id} size="md" />
      <IconButton icon="mdi:close" label="Close details" onclick={onclose} />
    </div>
  </div>

  <div class="mt-3 flex flex-wrap items-center justify-between gap-x-6 gap-y-3">
    <div class="flex flex-wrap items-center gap-x-3.5 gap-y-1 text-xs">
      <MetaItem size="md" icon={summary.icon} label={summary.label} tone={summary.tone} tooltip="Current state" />
      {#if pendingChange}
        <MetaItem size="md" icon="mdi:arrow-right" label="{pendingChange.optionLabel} pending" tone="warning" />
      {/if}
      <MetaItem
        size="md"
        icon="mdi:shield-half-full"
        label="{risk.name} risk"
        tone={RISK_TONE[def.riskLevel]}
        tooltip={risk.description}
      />
      <MetaItem
        size="md"
        icon={permission?.icon ?? STANDARD_USER.icon}
        label={permission?.name ?? STANDARD_USER.name}
        tone="neutral"
        tooltip={permission?.description}
      />
      {#if def.requiresReboot}
        <MetaItem
          size="md"
          icon="mdi:restart"
          label="Restart needed"
          tone="neutral"
          tooltip="After applying or restoring"
        />
      {/if}
      {#if !def.reversible}
        <MetaItem size="md" icon="mdi:undo-variant" label="Not reversible" tone="warning" />
      {/if}
      <MetaItem
        size="md"
        icon="mdi:history"
        label={status.hasHistory ? "Snapshot saved" : "No snapshot"}
        tone={status.hasHistory ? "neutral" : "subtle"}
      />
    </div>
    <div class="flex flex-wrap items-center gap-2">
      {#if status.hasHistory && !status.attention}<RestoreButton {tweak} />{/if}
      <TweakControl {tweak} class="max-w-md" />
    </div>
  </div>
</header>
