<script lang="ts" module>
  const ROWS = 4;
</script>

<script lang="ts">
  import { cn } from "$lib/utils/cn";
  import Skeleton from "./Skeleton.svelte";
  import { card } from "./variants";

  let { class: className }: { class?: string } = $props();
</script>

<div class={cn("flex flex-col gap-2 pb-4", className)}>
  {#each { length: ROWS }, i (i)}
    <div class={card({ class: "relative flex animate-pulse overflow-hidden" })}>
      <div class="flex flex-1 flex-col gap-3 p-4">
        <div class="flex items-start justify-between">
          <div class="flex flex-col gap-2">
            <Skeleton pulse={false} class="h-5 w-40" />
            <Skeleton soft pulse={false} class="h-4 w-56" />
          </div>
          <Skeleton pulse={false} class="h-9 w-12 rounded-lg" />
        </div>
        <div class="flex items-center gap-2">
          <Skeleton soft pulse={false} class="h-5 w-16 rounded-full" />
          <Skeleton soft pulse={false} class="h-5 w-20 rounded-full" />
        </div>
      </div>
    </div>
  {/each}
</div>
