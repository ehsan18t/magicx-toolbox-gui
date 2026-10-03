<script lang="ts" module>
  // On the collapsed rail, where the label and its markers are hidden.
  const RAIL_MARKER = "absolute -top-0.5 -right-1 ring-2 ring-background";
</script>

<script lang="ts">
  import { tooltip } from "$lib/attachments/tooltip.svelte";
  import { Icon } from "$lib/components/shared";
  import { Dot } from "$lib/components/ui";
  import { type IconName, type TextTone, TONE_FILL, TONE_TEXT } from "$lib/design";
  import { sidebarStore } from "$lib/stores/sidebar.svelte";
  import { SEP } from "$lib/utils/format";

  // Markers mean "act here": `alert` is attention, `pending` a change staged but not applied.
  interface Props {
    label: string;
    icon: IconName;
    active: boolean;
    onclick: () => void;
    trailing?: string;
    trailingTone?: TextTone | undefined;
    alert?: string;
    pending?: string;
  }

  let {
    label,
    icon,
    active,
    onclick,
    trailing = "",
    trailingTone = "subtle",
    alert = "",
    pending = "",
  }: Props = $props();

  const isOpen = $derived(sidebarStore.isOpen);
</script>

<button
  type="button"
  class="group relative flex h-9 w-full shrink-0 cursor-pointer items-center gap-3 rounded-md px-3 text-left text-sm text-foreground {active
    ? 'bg-muted'
    : 'hover:bg-muted'}"
  aria-current={active ? "page" : undefined}
  aria-label={[label, trailing, alert, pending].filter(Boolean).join(", ")}
  {@attach tooltip(() =>
    isOpen
      ? [alert, pending].filter(Boolean).join(SEP) || null
      : [label, trailing, alert, pending].filter(Boolean).join(SEP),
  )}
  {onclick}
>
  <span class="relative flex w-5 shrink-0 justify-center">
    <Icon {icon} size="lg" class={active ? "text-accent" : "text-foreground-muted group-hover:text-foreground"} />
    {#if alert && !isOpen}
      <Dot fill={TONE_FILL.error} class={RAIL_MARKER} />
    {:else if pending && !isOpen}
      <Dot fill={TONE_FILL.warning} class={RAIL_MARKER} />
    {/if}
  </span>
  {#if isOpen}
    <span class="min-w-0 flex-1 truncate">{label}</span>
    {#if alert}
      <Icon icon="mdi:alert-circle" size="xs" class="shrink-0 text-error" />
    {/if}
    {#if pending}
      <Dot fill={TONE_FILL.warning} class="shrink-0" />
    {/if}
    {#if trailing}
      <span class="shrink-0 text-xs tabular-nums {TONE_TEXT[trailingTone]}">{trailing}</span>
    {/if}
  {/if}
</button>
