<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import {
    Badge,
    Button,
    Card,
    EmptyState,
    IconTile,
    META_LINE,
    MetaItem,
    ModalBody,
    ModalFooter,
    SectionCard,
    SettingRow,
    Switch,
  } from "$lib/components/ui";
  import { HEADING } from "$lib/design";
  import type { ConfigurationProfile, ProfileValidation, TweakChangePreview } from "$lib/types";
  import { plural } from "$lib/utils/format";
  import { formatDate } from "$lib/utils/time";
  import { RISK_TONE, SYSTEM_DEFAULT_LABEL, toRiskLevel } from "$lib/utils/tweakPresentation";
  import type { SvelteSet } from "svelte/reactivity";
  import SelectableRow from "../SelectableRow.svelte";
  import IssueList, { type Issue } from "./IssueList.svelte";

  interface Props {
    profile: ConfigurationProfile;
    validation: ProfileValidation;
    /** Owned by the wizard, so the choices survive a failed apply. */
    skipTweakIds: SvelteSet<string>;
    skipAlreadyApplied: boolean;
    onback: () => void;
    onapply: () => void;
  }

  let { profile, validation, skipTweakIds, skipAlreadyApplied = $bindable(), onback, onapply }: Props = $props();

  const meta = $derived(profile.metadata);
  const applicable = $derived(validation.preview.filter((p) => p.applicable));
  const isAutoSkipped = (p: TweakChangePreview) => skipAlreadyApplied && p.already_applied;
  const isIncluded = (p: TweakChangePreview) => !isAutoSkipped(p) && !skipTweakIds.has(p.tweak_id);
  const includedCount = $derived(applicable.filter(isIncluded).length);

  const toIssue = (i: { tweak_id: string; code: string; message: string }): Issue => ({
    key: i.tweak_id + i.code,
    message: i.message,
  });
  const warnings = $derived(validation.warnings.map(toIssue));
  const errors = $derived(validation.errors.map(toIssue));

  function toggle(tweakId: string) {
    if (!skipTweakIds.delete(tweakId)) skipTweakIds.add(tweakId);
  }
</script>

<ModalBody class="animate-fade-in space-y-4">
  <Card class="flex items-start gap-3 p-4">
    <IconTile icon="mdi:file-document" size="lg" />
    <div class="min-w-0 flex-1">
      <h3 class={["m-0 truncate", HEADING.section]}>{meta.name}</h3>
      {#if meta.description}<p class="m-0 mt-1 text-sm text-foreground-muted">{meta.description}</p>{/if}
      <p class={["m-0 mt-2 text-foreground-muted", META_LINE]}>
        <MetaItem size="md" icon="mdi:calendar" label={formatDate(meta.created_at, { month: "short" })} />
        <MetaItem size="md" icon="mdi:microsoft-windows" label="Windows {meta.source_windows_version}" />
        <MetaItem size="md" icon="mdi:tune-variant" label={plural(validation.stats.total_tweaks, "tweak")} />
      </p>
    </div>
  </Card>

  {#if warnings.length > 0}
    <IssueList tone="warning" title={plural(warnings.length, "warning")} issues={warnings} />
  {/if}
  {#if errors.length > 0}
    <IssueList tone="error" title="{plural(errors.length, 'error')}: those tweaks are skipped" issues={errors} />
  {/if}

  <SectionCard title="Changes to apply">
    {#snippet actions()}<Badge class="mr-2">{plural(includedCount, "tweak")}</Badge>{/snippet}
    {#if applicable.length === 0}
      <EmptyState icon="mdi:alert-circle-outline" description="No tweak in this profile applies to this PC." />
    {:else}
      <div class="max-h-64 divide-y divide-border overflow-y-auto">
        {#each applicable as preview (preview.tweak_id)}
          <SelectableRow
            checked={isIncluded(preview)}
            disabled={isAutoSkipped(preview)}
            label={preview.tweak_name}
            onclick={() => toggle(preview.tweak_id)}
          >
            <span class="flex items-center gap-2">
              <span class="truncate text-sm font-medium">{preview.tweak_name}</span>
              {#if preview.already_applied}<Badge class="shrink-0">Already applied</Badge>{/if}
            </span>
            <span class="mt-0.5 flex items-center gap-1 text-xs text-foreground-muted">
              {preview.current_option_label ?? SYSTEM_DEFAULT_LABEL}
              <Icon icon="mdi:arrow-right" size="3xs" />
              <span class="text-accent">{preview.target_option_label}</span>
            </span>
            {#snippet trailing()}
              <Badge tone={RISK_TONE[toRiskLevel(preview.risk_level)]} class="shrink-0">
                {plural(preview.changes.length, "change")}
              </Badge>
            {/snippet}
          </SelectableRow>
        {/each}
      </div>
    {/if}
  </SectionCard>

  <Card>
    <SettingRow title="Skip already-applied tweaks">
      <Switch
        checked={skipAlreadyApplied}
        label="Skip already-applied tweaks"
        onchange={(on) => (skipAlreadyApplied = on)}
      />
    </SettingRow>
  </Card>
</ModalBody>

<ModalFooter>
  <Button variant="secondary" icon="mdi:arrow-left" onclick={onback}>Back</Button>
  <Button variant="primary" icon="mdi:check" disabled={includedCount === 0} onclick={onapply}>
    Apply {plural(includedCount, "tweak")}
  </Button>
</ModalFooter>
