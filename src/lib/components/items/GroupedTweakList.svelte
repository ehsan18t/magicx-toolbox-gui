<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Count, rowButton } from "$lib/components/ui";
  import { HEADING } from "$lib/design";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { categoriesStore } from "$lib/stores/tweaksData.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { groupByCategory } from "$lib/utils/tweakPresentation";
  import TweakRow from "./TweakRow.svelte";

  interface Props {
    tweaks: TweakWithStatus[];
  }

  let { tweaks }: Props = $props();

  const uid = $props.id();
  const groups = $derived(groupByCategory(tweaks));
</script>

<div class="flex animate-fade-in flex-col gap-6">
  {#each groups as [categoryId, list] (categoryId)}
    <section class="flex flex-col gap-2" aria-labelledby="{uid}-{categoryId}">
      <h2 id="{uid}-{categoryId}" class="m-0">
        <button
          type="button"
          class={rowButton({ class: ["group flex w-fit max-w-full items-center gap-2 px-1.5 py-1", HEADING.item] })}
          onclick={() => navigationStore.navigateToCategory(categoryId)}
        >
          <Icon icon={categoriesStore.icon(categoryId)} size="md" class="shrink-0 text-foreground-muted" />
          <span class="truncate">{categoriesStore.name(categoryId)}</span>
          <Count value={list.length} class="font-normal" />
          <Icon
            icon="mdi:chevron-right"
            size="md"
            class="shrink-0 text-foreground-subtle group-hover:text-foreground"
          />
        </button>
      </h2>
      {#each list as tweak (tweak.definition.id)}
        <TweakRow {tweak} />
      {/each}
    </section>
  {/each}
</div>
