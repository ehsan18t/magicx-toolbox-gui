<script lang="ts">
  import { Icon, type IconName } from "$lib/components/shared";
  import { Card, Count, ICON_SIZE, PanelHeading, type TextTone } from "$lib/components/ui";
  import { tweakDetailsModalStore } from "$lib/stores/tweakDetailsModal.svelte";
  import { pendingChangesStore, pendingRebootStore } from "$lib/stores/tweaksPending.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { expand, fade, reducedMotion } from "$lib/utils/motion";
  import { attentionCause, rowDomId, tallies } from "$lib/utils/tweakPresentation";

  interface Props {
    /** Names the pane for assistive tech, e.g. "Security at a glance". */
    label: string;
    tweaks: TweakWithStatus[];
  }

  let { label, tweaks }: Props = $props();

  const stats = $derived(tallies(tweaks));
  const breakdown = $derived(
    [
      { label: "Applied", value: stats.applied, tone: "bg-accent" },
      { label: "System default", value: stats.byState.system_default, tone: "bg-foreground-subtle" },
      { label: "Unknown", value: stats.byState.unknown, tone: "bg-warning" },
      { label: "Unavailable", value: stats.byState.unavailable, tone: "bg-border-hover" },
      { label: "Checking", value: stats.byState.loading, tone: "bg-border" },
    ].filter((b) => b.value > 0),
  );

  const attention = $derived(tweaks.filter((t) => t.status.attention));
  const pending = $derived(tweaks.filter((t) => pendingChangesStore.has(t.definition.id)));
  const unknown = $derived(tweaks.filter((t) => t.status.state === "unknown"));
  const reboot = $derived(tweaks.filter((t) => pendingRebootStore.has(t.definition.id)));
  const allClear = $derived(
    attention.length + pending.length + unknown.length + reboot.length + stats.byState.loading === 0,
  );

  function reveal(id: string) {
    tweakDetailsModalStore.open(id);
    document.getElementById(rowDomId("tweak", id))?.scrollIntoView({
      block: "nearest",
      behavior: reducedMotion() ? "auto" : "smooth",
    });
  }
</script>

{#snippet group(
  icon: IconName,
  tone: TextTone,
  title: string,
  items: TweakWithStatus[],
  detail: (t: TweakWithStatus) => string,
)}
  {#if items.length > 0}
    <section transition:expand>
      <PanelHeading {icon} {tone} class="mb-1.5">
        {title}
        <Count value={items.length} class="font-normal" />
      </PanelHeading>
      <ul class="m-0 list-none space-y-0.5 p-0">
        {#each items as t (t.definition.id)}
          {@const d = detail(t)}
          <li>
            <button
              type="button"
              class="group flex w-full cursor-pointer items-start gap-2 rounded-md px-2 py-1.5 text-left hover:bg-muted"
              onclick={() => reveal(t.definition.id)}
            >
              <span class="min-w-0 flex-1">
                <span class="block text-ui wrap-break-word">{t.definition.name}</span>
                {#if d}<span class="block text-xs text-foreground-muted">{d}</span>{/if}
              </span>
              <Icon
                icon="mdi:chevron-right"
                width={ICON_SIZE.md}
                class="mt-0.5 shrink-0 text-foreground-subtle group-hover:text-foreground"
              />
            </button>
          </li>
        {/each}
      </ul>
    </section>
  {/if}
{/snippet}

<aside
  class="flex w-summary-panel shrink-0 animate-fade-in flex-col border-l border-border bg-surface"
  aria-label={label}
>
  <header class="shrink-0 border-b border-border px-5 pt-4 pb-3">
    <h2 class="m-0 font-display text-lg font-semibold">At a glance</h2>
    <p class="m-0 mt-0.5 text-ui text-foreground-muted">Select a tweak to see its details here.</p>
  </header>

  <div class="min-h-0 flex-1 space-y-5 overflow-y-auto px-5 py-4">
    {#if stats.total > 0}
      <Card as="section" class="p-3">
        <div class="flex items-baseline justify-between">
          <span class="text-ui text-foreground-muted">Applied</span>
          <span class="text-sm font-semibold tabular-nums">{stats.applied} of {stats.total}</span>
        </div>
        <div class="mt-2 flex h-1.5 overflow-hidden rounded-full bg-muted">
          {#each breakdown as b (b.label)}
            <div
              class="transition-[width] duration-slower ease-out {b.tone}"
              style:width="{(b.value / stats.total) * 100}%"
            ></div>
          {/each}
        </div>
        <ul class="m-0 mt-2.5 flex list-none flex-wrap gap-x-4 gap-y-1 p-0">
          {#each breakdown as b (b.label)}
            <li class="flex items-center gap-1.5 text-xs text-foreground-muted">
              <span class="h-2 w-2 rounded-full {b.tone}"></span>
              {b.label}
              <span class="text-foreground tabular-nums">{b.value}</span>
            </li>
          {/each}
        </ul>
      </Card>
    {/if}

    {@render group("mdi:alert-circle", "error", "Needs attention", attention, (t) =>
      attentionCause(t.status.attention?.reason),
    )}
    {@render group(
      "mdi:arrow-right",
      "warning",
      "Ready to apply",
      pending,
      (t) => `→ ${pendingChangesStore.change(t.definition.id)?.optionLabel ?? ""}`,
    )}
    {@render group("mdi:help-circle-outline", "warning", "State unknown", unknown, (t) =>
      t.status.needsElevation ? "Restart as administrator to read it" : "",
    )}
    {@render group("mdi:restart", "info", "Waiting for a restart", reboot, () => "")}

    {#if allClear && stats.total > 0}
      <div class="flex items-center gap-2.5 rounded-lg border border-border bg-card p-3 text-ui" in:fade>
        <Icon icon="mdi:check-circle" width={ICON_SIZE.lg} class="shrink-0 text-success" />
        <span class="text-foreground-muted">Nothing here needs your attention.</span>
      </div>
    {/if}
  </div>
</aside>
