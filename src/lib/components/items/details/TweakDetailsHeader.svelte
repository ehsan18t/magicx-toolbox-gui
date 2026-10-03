<script lang="ts">
  import { IconButton, ModalTitle } from "$lib/components/ui";
  import { tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { pendingChangesStore } from "$lib/stores/tweaksPending.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { type MetaFact, pendingFact, tweakMeta } from "$lib/utils/tweakPresentation";
  import FavoriteButton from "../FavoriteButton.svelte";
  import MetaItem from "../MetaItem.svelte";
  import RestoreButton from "../RestoreButton.svelte";
  import TweakControl from "../TweakControl.svelte";

  interface Props {
    tweak: TweakWithStatus;
    onclose: () => void;
  }

  let { tweak, onclose }: Props = $props();

  const def = $derived(tweak.definition);
  const status = $derived(tweak.status);
  const facts = $derived(tweakMeta(tweak, tweaksStore.tweak));
  const pendingChange = $derived(pendingChangesStore.change(def.id));
</script>

{#snippet fact(item: MetaFact | null)}
  {#if item}<MetaItem {...item} size="md" />{/if}
{/snippet}

<header class="shrink-0 border-b border-border px-6 pt-5 pb-4">
  <div class="flex items-start gap-4">
    <div class="min-w-0 flex-1">
      <ModalTitle size="xl">{def.name}</ModalTitle>
      <p class="m-0 mt-1 text-ui leading-relaxed text-foreground-muted">{def.description}</p>
    </div>
    <div class="flex shrink-0 items-center gap-0.5">
      <FavoriteButton tweakId={def.id} size="md" />
      <IconButton icon="mdi:close" label="Close details" onclick={onclose} />
    </div>
  </div>

  <div class="mt-3 flex flex-wrap items-center justify-between gap-x-6 gap-y-3">
    <div class="flex flex-wrap items-center gap-x-3.5 gap-y-1 text-xs">
      {@render fact(facts.state)}
      {#if pendingChange}{@render fact(pendingFact(pendingChange.optionLabel))}{/if}
      {@render fact(facts.risk)}
      {@render fact(facts.permission)}
      {@render fact(facts.restart)}
      {@render fact(facts.irreversible)}
      {@render fact(facts.snapshot)}
    </div>
    <div class="flex flex-wrap items-center gap-2">
      {#if status.hasHistory && !status.attention}<RestoreButton {tweak} />{/if}
      <TweakControl {tweak} class="max-w-md" />
    </div>
  </div>
</header>
