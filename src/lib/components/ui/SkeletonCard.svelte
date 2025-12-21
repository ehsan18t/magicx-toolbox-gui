<script lang="ts">
  /**
   * SkeletonCard - Loading skeleton for tweak cards
   *
   * Used in CategoryTab, FavoritesTab, SnapshotsTab during loading state.
   */

  interface Props {
    /** Number of skeleton cards to render */
    count?: number;
  }

  const { count = 4 }: Props = $props();
</script>

<div class="flex flex-col gap-3 pb-4 lg:grid lg:grid-cols-2 lg:gap-4">
  <!-- eslint-disable-next-line @typescript-eslint/no-unused-vars -->
  {#each Array(count) as _, i (`tweak-skeleton-${i}`)}
    <div class="animate-pulse relative flex overflow-hidden rounded-lg border border-border bg-card">
      <div class="flex flex-1 flex-col gap-3 p-4">
        <div class="flex items-start justify-between">
          <div class="flex flex-col gap-2">
            <div class="bg-muted h-5 w-40 rounded"></div>
            <div class="bg-muted/60 h-4 w-56 rounded"></div>
          </div>
          <div class="bg-muted h-9 w-12 rounded-lg"></div>
        </div>
        <div class="flex items-center gap-2">
          <div class="bg-muted/60 h-5 w-16 rounded-full"></div>
          <div class="bg-muted/60 h-5 w-20 rounded-full"></div>
        </div>
      </div>
    </div>
  {/each}
</div>
