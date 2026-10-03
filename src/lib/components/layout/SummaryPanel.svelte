<script lang="ts" module>
  import { TONE_FILL } from "$lib/design";
  import { CHECKING, SYSTEM_DEFAULT_LABEL, type Tallies } from "$lib/utils/tweakPresentation";

  // Bar and legend order.
  const SEGMENTS: { state: keyof Tallies["byState"]; label: string; fill: string }[] = [
    { state: "active", label: "Applied", fill: TONE_FILL.accent },
    { state: "system_default", label: SYSTEM_DEFAULT_LABEL, fill: "bg-foreground-subtle" },
    { state: "unknown", label: "Unknown", fill: TONE_FILL.warning },
    { state: "unavailable", label: "Unavailable", fill: "bg-border-hover" },
    { state: "loading", label: CHECKING.label, fill: "bg-border" },
  ];
</script>

<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { HEADING, type IconName, type TextTone } from "$lib/design";
  import { Card, Count, Dot, PanelHeading, rowButton } from "$lib/components/ui";
  import { tweakDetailsModalStore } from "$lib/stores/detailsModal.svelte";
  import { pendingChangesStore, pendingRebootStore } from "$lib/stores/tweaksPending.svelte";
  import type { TweakWithStatus } from "$lib/types";
  import { capitalize } from "$lib/utils/format";
  import { expand, reducedMotion } from "$lib/utils/motion";
  import { attentionCause, ELEVATE_REMEDY, rowDomId, tallies } from "$lib/utils/tweakPresentation";
  import AppliedMeter from "./AppliedMeter.svelte";

  interface Props {
    /** Names the pane for assistive tech, e.g. "Security at a glance". */
    label: string;
    tweaks: TweakWithStatus[];
  }

  let { label, tweaks }: Props = $props();

  const stats = $derived(tallies(tweaks));
  const breakdown = $derived(
    SEGMENTS.map((s) => ({ ...s, key: s.state, value: stats.byState[s.state] })).filter((s) => s.value > 0),
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
              class={rowButton({ class: "group flex w-full items-start gap-2 px-2 py-1.5" })}
              onclick={() => reveal(t.definition.id)}
            >
              <span class="min-w-0 flex-1">
                <span class="block text-ui wrap-break-word">{t.definition.name}</span>
                {#if d}<span class="block text-xs text-foreground-muted">{d}</span>{/if}
              </span>
              <Icon
                icon="mdi:chevron-right"
                size="md"
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
    <h2 class={["m-0", HEADING.pane]}>At a glance</h2>
    <p class="m-0 mt-0.5 text-ui text-foreground-muted">Select a tweak to open its details.</p>
  </header>

  <div class="min-h-0 flex-1 space-y-5 overflow-y-auto px-5 py-4">
    {#if stats.total > 0}
      <Card as="section" class="p-3">
        <AppliedMeter applied={stats.applied} total={stats.total} segments={breakdown} />
        <ul class="m-0 mt-2.5 flex list-none flex-wrap gap-x-4 gap-y-1 p-0">
          {#each breakdown as b (b.state)}
            <li class="flex items-center gap-1.5 text-xs text-foreground-muted">
              <Dot fill={b.fill} />
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
      t.status.needsElevation ? capitalize(ELEVATE_REMEDY) : "",
    )}
    {@render group("mdi:restart", "info", "Waiting for a restart", reboot, () => "")}

    {#if allClear && stats.total > 0}
      <Card class="flex animate-fade-in items-center gap-2.5 p-3 text-ui">
        <Icon icon="mdi:check-circle" size="lg" class="shrink-0 text-success" />
        <span class="text-foreground-muted">Nothing here needs your attention.</span>
      </Card>
    {/if}
  </div>
</aside>
