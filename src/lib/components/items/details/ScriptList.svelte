<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { card, CodeBlock, PanelSection } from "$lib/components/ui";
  import type { TweakEffectOption } from "$lib/types";
  import { plural } from "$lib/utils/format";

  let { options }: { options: TweakEffectOption[] } = $props();
</script>

<PanelSection title="Scripts" icon="mdi:console">
  <div class="space-y-1.5">
    {#each options as option (option.label)}
      <details class={card({ class: "group" })}>
        <summary class="flex cursor-pointer list-none items-center gap-2 px-3 py-2 text-ui">
          <Icon
            icon="mdi:chevron-right"
            size="md"
            class="shrink-0 transition-transform duration-normal group-open:rotate-90"
          />
          <span class="font-medium">{option.label}</span>
          <span class="text-xs text-foreground-muted">runs {plural(option.commands.length, "script")}</span>
        </summary>
        <div class="space-y-1.5 border-t border-border p-3">
          {#each option.commands as cmd, idx (idx)}
            <CodeBlock kind="command">{cmd}</CodeBlock>
          {/each}
        </div>
      </details>
    {/each}
  </div>
</PanelSection>
