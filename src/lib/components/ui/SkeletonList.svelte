<script lang="ts" module>
  const ROWS = 4;
</script>

<script lang="ts">
  import { SKELETON_SOFT } from "$lib/design";
  import { cn } from "$lib/utils/cn";
  import { card } from "./variants";

  let { class: className }: { class?: string } = $props();
</script>

<div class={cn("flex flex-col gap-2 pb-4", className)}>
  {#each { length: ROWS }, i (i)}
    <div class={card({ class: "relative flex animate-pulse overflow-hidden" })}>
      <div class="flex flex-1 flex-col gap-3 p-4">
        <div class="flex items-start justify-between">
          <div class="flex flex-col gap-2">
            <div class="h-5 w-40 rounded bg-muted"></div>
            <div class="h-4 w-56 rounded {SKELETON_SOFT}"></div>
          </div>
          <div class="h-9 w-12 rounded-lg bg-muted"></div>
        </div>
        <div class="flex items-center gap-2">
          <div class="h-5 w-16 rounded-full {SKELETON_SOFT}"></div>
          <div class="h-5 w-20 rounded-full {SKELETON_SOFT}"></div>
        </div>
      </div>
    </div>
  {/each}
</div>
