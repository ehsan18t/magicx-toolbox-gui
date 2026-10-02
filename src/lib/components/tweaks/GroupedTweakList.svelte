<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { categoriesStore } from "$lib/stores/tweaks.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import TweakRow from "./TweakRow.svelte";

  interface Props {
    tweaks: TweakWithStatus[];
  }

  let { tweaks }: Props = $props();

  const groups = $derived.by(() => {
    const byCategory: Record<string, TweakWithStatus[]> = {};
    for (const t of tweaks) (byCategory[t.definition.category_id] ??= []).push(t);
    return Object.entries(byCategory);
  });
</script>

<div class="flex flex-col gap-6">
  {#each groups as [categoryId, list] (categoryId)}
    <section class="flex flex-col gap-2" aria-labelledby="group-{categoryId}">
      <button
        type="button"
        class="group flex w-fit max-w-full cursor-pointer items-center gap-2 rounded-md px-1.5 py-1 hover:bg-muted"
        onclick={() => navigationStore.navigateToCategory(categoryId)}
      >
        <Icon icon={categoriesStore.getIcon(categoryId)} width="16" class="shrink-0 text-foreground-muted" />
        <h2 id="group-{categoryId}" class="m-0 truncate text-sm font-semibold">
          {categoriesStore.getName(categoryId)}
        </h2>
        <span class="text-xs text-foreground-subtle tabular-nums">{list.length}</span>
        <Icon icon="mdi:chevron-right" width="16" class="shrink-0 text-foreground-subtle group-hover:text-foreground" />
      </button>
      {#each list as tweak (tweak.definition.id)}
        <TweakRow {tweak} />
      {/each}
    </section>
  {/each}
</div>
