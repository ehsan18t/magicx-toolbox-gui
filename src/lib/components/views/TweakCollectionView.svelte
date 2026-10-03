<script lang="ts">
  import { NoMatches } from "$lib/components/feedback";
  import { PageLayout, PageStats } from "$lib/components/layout";
  import { GroupedTweakList, RestoreAllButton } from "$lib/components/items";
  import { EmptyState, SkeletonList } from "$lib/components/ui";
  import type { IconName } from "$lib/design";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { pageFilterStore } from "$lib/stores/pageFilter.svelte";
  import { tweaksStore } from "$lib/stores/tweaksData.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { canRestore, tallies } from "$lib/utils/tweakPresentation";
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    description: string;
    tweaks: TweakWithStatus[];
    /** The plural noun in "No favorites match …". */
    noun: string;
    countLabel: string;
    appliedLabel: string;
    restoreTitle: string;
    /** Show Restore all disabled, rather than hidden, when nothing is restorable. */
    alwaysShowRestore?: boolean;
    empty: { icon: IconName; title: string; description: string };
    /** Page actions beside Restore all. */
    actions?: Snippet;
  }

  let {
    title,
    description,
    tweaks,
    noun,
    countLabel,
    appliedLabel,
    restoreTitle,
    alwaysShowRestore = false,
    empty,
    actions,
  }: Props = $props();

  const filteredTweaks = $derived(tweaks.filter((t) => pageFilterStore.passes(t.definition.id)));
  const restorable = $derived(tweaks.filter(canRestore));
  const applied = $derived(tallies(tweaks).applied);
</script>

{#snippet summary()}
  <PageStats
    items={[
      { value: tweaks.length, label: countLabel },
      { value: applied, label: appliedLabel },
    ]}
  />
  <div class="ml-auto flex flex-wrap gap-2">
    {#if alwaysShowRestore || restorable.length > 0}
      <RestoreAllButton title={restoreTitle} tweaks={restorable} />
    {/if}
    {@render actions?.()}
  </div>
{/snippet}

<PageLayout {title} {description} aside={tweaks.length > 0 ? summary : undefined}>
  {#if tweaksStore.isLoading && tweaks.length === 0}
    <SkeletonList />
  {:else if tweaks.length === 0}
    <EmptyState
      {...empty}
      action={{ label: "Browse tweaks", onclick: () => navigationStore.navigateToTab("overview") }}
      showIconCircle
    />
  {:else if filteredTweaks.length === 0}
    <NoMatches description={`No ${noun} match "${pageFilterStore.trimmedQuery}"`} />
  {:else}
    <GroupedTweakList tweaks={filteredTweaks} />
  {/if}
</PageLayout>
