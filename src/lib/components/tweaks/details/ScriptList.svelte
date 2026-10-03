<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { ICON_SIZE, PanelHeading } from "$lib/components/ui";
  import type { TweakEffectOption } from "$lib/types";
  import { plural } from "$lib/utils/format";

  let { options }: { options: TweakEffectOption[] } = $props();

  const headingId = $props.id();
</script>

<section aria-labelledby={headingId}>
  <PanelHeading id={headingId} icon="mdi:console" class="mb-2.5">Scripts</PanelHeading>
  <div class="space-y-1.5">
    {#each options as option (option.label)}
      <details class="group rounded-lg border border-border bg-card">
        <summary class="flex cursor-pointer list-none items-center gap-2 px-3 py-2 text-ui">
          <Icon
            icon="mdi:chevron-right"
            width={ICON_SIZE.md}
            class="shrink-0 transition-transform duration-normal group-open:rotate-90"
          />
          <span class="font-medium">{option.label}</span>
          <span class="text-xs text-foreground-muted">runs {plural(option.commands.length, "script")}</span>
        </summary>
        <div class="space-y-1.5 border-t border-border p-3">
          {#each option.commands as cmd, idx (idx)}
            <code
              class="block rounded-md border border-border bg-background px-3 py-2 font-mono text-caption break-all whitespace-pre-wrap text-foreground/85"
              >{cmd}</code
            >
          {/each}
        </div>
      </details>
    {/each}
  </div>
</section>
