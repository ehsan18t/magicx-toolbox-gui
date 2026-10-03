<script lang="ts" module>
  import type { PendingChange, TweakWithStatus } from "$lib/types";

  export interface PendingItem {
    change: PendingChange;
    tweak: TweakWithStatus | undefined;
    name: string;
    fromArrow: string;
    highRisk: boolean;
  }
</script>

<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Button, Modal, ModalBody, ModalFooter, ModalHeader } from "$lib/components/ui";
  import { TONE_TINT } from "$lib/design";
  import { plural } from "$lib/utils/format";
  import { RISK_INFO } from "$lib/utils/tweakPresentation";

  interface Props {
    open: boolean;
    items: PendingItem[];
    busy: boolean;
    onclose: () => void;
    onapply: () => void;
  }

  let { open, items, busy, onclose, onapply }: Props = $props();

  const count = $derived(items.length);
  const highRiskCount = $derived(items.filter((i) => i.highRisk).length);
</script>

<Modal {open} {onclose} size="lg">
  <ModalHeader title="Review {plural(count, 'change')}">
    {#snippet leading()}<Icon icon="mdi:alert" size="2xl" class="shrink-0 text-warning" />{/snippet}
    <p class="m-0 mt-0.5 text-ui text-foreground-muted">
      {plural(highRiskCount, "change is", "changes are")} high risk. Check them before applying.
    </p>
  </ModalHeader>
  <ModalBody>
    <ul class="m-0 list-none space-y-1.5 p-0">
      {#each items as { change, tweak, name, fromArrow, highRisk } (change.tweakId)}
        <li class="rounded-md border px-3 py-2 text-ui {highRisk ? TONE_TINT.error : 'border-border'}">
          <div class="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-0.5">
            <span class="font-medium text-foreground">{name}</span>
            <span class="text-foreground-muted">
              {fromArrow}<span class="font-medium text-foreground">{change.optionLabel}</span>
            </span>
          </div>
          {#if tweak && (highRisk || tweak.definition.requiresReboot)}
            <div class="mt-1 flex flex-wrap gap-x-3 text-xs">
              {#if highRisk}
                <span class="text-error">{RISK_INFO[tweak.definition.riskLevel].name} risk</span>
              {/if}
              {#if tweak.definition.requiresReboot}<span class="text-info">Needs a restart</span>{/if}
            </div>
          {/if}
          {#if highRisk && tweak?.definition.warning}
            <p class="m-0 mt-1 text-xs leading-relaxed text-foreground-muted">{tweak.definition.warning}</p>
          {/if}
        </li>
      {/each}
    </ul>
  </ModalBody>
  <ModalFooter>
    <Button variant="secondary" onclick={onclose}>Back</Button>
    <Button variant="warning" onclick={onapply} disabled={busy}>
      Apply {count === 1 ? "change" : `${count} changes`}
    </Button>
  </ModalFooter>
</Modal>
