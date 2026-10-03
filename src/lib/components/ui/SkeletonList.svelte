<script lang="ts" module>
  const ROWS = 4;
</script>

<div class="flex flex-col gap-2 pb-4">
  {#each { length: ROWS }, i (i)}
    <div class="relative flex animate-pulse overflow-hidden rounded-lg border border-border bg-card">
      <div class="flex flex-1 flex-col gap-3 p-4">
        <div class="flex items-start justify-between">
          <div class="flex flex-col gap-2">
            <div class="h-5 w-40 rounded bg-muted"></div>
            <div class="h-4 w-56 rounded bg-muted/60"></div>
          </div>
          <div class="h-9 w-12 rounded-lg bg-muted"></div>
        </div>
        <div class="flex items-center gap-2">
          <div class="h-5 w-16 rounded-full bg-muted/60"></div>
          <div class="h-5 w-20 rounded-full bg-muted/60"></div>
        </div>
      </div>
    </div>
  {/each}
</div>
