<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { categoriesStore } from "$lib/stores/tweaksData.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import TweakRow from "./TweakRow.svelte";

  interface Props {
    tweaks: TweakWithStatus[];
  }

  let { tweaks }: Props = $props();

  const groups = $derived.by(() => {
    const byCategory: Record<string, TweakWithStatus[]> = {};
    for (const t of tweaks) (byCategory[t.definition.categoryId] ??= []).push(t);
    return Object.entries(byCategory);
  });
</script>

<div class="flex animate-fade-in flex-col gap-6">
  {#each groups as [categoryId, list] (categoryId)}
    <section class="flex flex-col gap-2" aria-labelledby="group-{categoryId}">
      <h2 id="group-{categoryId}" class="m-0">
        <button
          type="button"
          class="group flex w-fit max-w-full cursor-pointer items-center gap-2 rounded-md px-1.5 py-1 text-sm font-semibold hover:bg-muted"
          onclick={() => navigationStore.navigateToTab(categoryId)}
        >
          <Icon icon={categoriesStore.getIcon(categoryId)} width="16" class="shrink-0 text-foreground-muted" />
          <span class="truncate">{categoriesStore.getName(categoryId)}</span>
          <span class="text-xs font-normal text-foreground-subtle tabular-nums">{list.length}</span>
          <Icon
            icon="mdi:chevron-right"
            width="16"
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
