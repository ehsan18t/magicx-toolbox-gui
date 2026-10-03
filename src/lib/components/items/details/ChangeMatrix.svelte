<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { COLUMN_TINT, TABLE_HEAD } from "$lib/design";
  import { PanelSection } from "$lib/components/ui";
  import { pendingChangesStore } from "$lib/stores/tweaksPending.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { buildMatrix, KIND_META, type MatrixCell, optionsMatchingNow, optionTone } from "$lib/utils/changeMatrix";
  import { cn } from "$lib/utils/cn";
  import { labelsOf } from "$lib/utils/tweakPresentation";

  interface Props {
    tweak: TweakWithStatus;
    /** Some option runs scripts, listed below the matrix. */
    scripted: boolean;
  }

  let { tweak, scripted }: Props = $props();

  const def = $derived(tweak.definition);
  const status = $derived(tweak.status);
  const pendingLabel = $derived(pendingChangesStore.change(def.id)?.optionLabel);
  const matrix = $derived(buildMatrix(def.options, status.observed?.changes ?? null));
  const showNow = $derived(matrix.some((r) => r.now !== null));
  const columnCount = $derived(def.options.length + (showNow ? 2 : 1));
  const labels = $derived(labelsOf(def));
  const tones = $derived(labels.map((label) => optionTone(label, status.activeOption, pendingLabel)));
</script>

{#snippet cellText(cell: MatrixCell | null)}
  {#if cell}
    <span class={cell.removal ? "text-foreground-muted italic" : "font-medium"}>
      {cell.text}
    </span>
    {#if cell.note}<span class="ml-1 text-caption text-foreground-subtle">{cell.note}</span>{/if}
  {:else}
    <span class="text-foreground-subtle" use:tooltip={"Left as it is by this option"}>–</span>
  {/if}
{/snippet}

<PanelSection title="What each option sets" icon="mdi:tune-variant">
  {#if matrix.length === 0}
    <p class="m-0 text-ui text-foreground-muted">
      {scripted ? "This tweak works through the scripts below." : "No system settings are listed."}
    </p>
  {:else}
    <div class="overflow-x-auto rounded-lg border border-border">
      <table class="w-full border-collapse text-left text-ui">
        <thead>
          <tr class={cn("border-b border-border text-xs", TABLE_HEAD)}>
            <th scope="col" class="sticky left-0 z-raised min-w-56 bg-card px-3 py-2 font-medium text-foreground-muted">
              Setting
            </th>
            {#if showNow}
              <th scope="col" class={cn("min-w-36 px-3 py-2 font-semibold", COLUMN_TINT.now.head)}>
                This PC now
                <span class="block text-caption font-normal text-foreground-muted">no single option matches</span>
              </th>
            {/if}
            {#each def.options as option, i (option.label)}
              {@const tone = tones[i]}
              {@const unavailable = status.unavailableOptions.find((u) => u.label === option.label)}
              <th scope="col" class={cn("min-w-36 px-3 py-2 font-semibold", tone && COLUMN_TINT[tone].head)}>
                {option.label}
                {#if tone === "current"}
                  <span class="block text-caption font-medium text-accent">Current</span>
                {:else if tone === "pending"}
                  <span class="block text-caption font-medium text-warning">Pending</span>
                {:else if unavailable}
                  <span class="block text-caption font-normal text-warning" use:tooltip={unavailable.reason}>
                    Unavailable here
                  </span>
                {/if}
              </th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each matrix as row, r (`${row.kind}-${row.location}-${row.title}-${r}`)}
            {#if r === 0 || matrix[r - 1].kind !== row.kind}
              <tr class="border-b border-border">
                <th
                  scope="rowgroup"
                  colspan={columnCount}
                  class="px-3 pt-3 pb-1.5 text-caption font-semibold tracking-wide text-foreground-muted uppercase"
                >
                  <span class="sticky left-3 inline-flex items-center gap-1.5">
                    <Icon icon={KIND_META[row.kind].icon} size="2xs" />
                    {KIND_META[row.kind].label}
                  </span>
                </th>
              </tr>
            {/if}
            <tr class="border-b border-border last:border-b-0">
              <th scope="row" class="sticky left-0 z-raised max-w-80 bg-card px-3 py-2 align-top font-normal">
                <span class="flex flex-wrap items-baseline gap-x-2">
                  <span class="font-mono text-code font-semibold break-all">{row.title}</span>
                  {#if row.type}<span class="text-caption text-foreground-subtle">{row.type}</span>{/if}
                </span>
                <span
                  class="mt-0.5 block truncate font-mono text-caption text-foreground-muted"
                  use:tooltip={row.location}
                >
                  {row.location}
                </span>
              </th>
              {#if showNow}
                {@const agrees = optionsMatchingNow(row, labels, status.observed?.agreement ?? [])}
                <td class={cn("px-3 py-2 align-top break-all", COLUMN_TINT.now.cell)}>
                  {@render cellText(row.now)}
                  {#if row.now}
                    {#if agrees.length}
                      <span class="mt-0.5 flex items-center gap-1 text-caption text-success">
                        <Icon icon="mdi:check" size="3xs" class="shrink-0" />
                        {agrees.join(", ")}
                      </span>
                    {:else}
                      <span class="mt-0.5 block text-caption text-warning">No option</span>
                    {/if}
                  {/if}
                </td>
              {/if}
              {#each row.cells as cell, i (i)}
                {@const tone = tones[i]}
                <td class={cn("px-3 py-2 align-top break-all", tone && COLUMN_TINT[tone].cell)}>
                  {@render cellText(cell)}
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</PanelSection>
