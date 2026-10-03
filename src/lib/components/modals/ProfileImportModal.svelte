<script lang="ts" module>
  const MAX_LISTED = 5;

  type Step = "select" | "review" | "applying" | "complete";

  interface Issue {
    key: string;
    message: string;
    tweakName?: string;
    rolledBack?: boolean;
  }
</script>

<script lang="ts">
  import { Icon, type IconName } from "$lib/components/shared";
  import {
    Badge,
    Button,
    Callout,
    Checkbox,
    ICON_SIZE,
    Modal,
    ModalBody,
    ModalFooter,
    ProgressBar,
    Switch,
    TONE_SOFT,
    TONE_TEXT,
  } from "$lib/components/ui";
  import { PROFILE_EXT } from "$lib/config/app";
  import { bootStore } from "$lib/stores/boot.svelte";
  import { modalStore } from "$lib/stores/modal.svelte";
  import { profileStore } from "$lib/stores/profile.svelte";
  import { TOAST_DURATION, toastStore } from "$lib/stores/toast.svelte";
  import { listenFileDrop } from "$lib/utils/fileDrop";
  import { formatDate, plural } from "$lib/utils/format";
  import { RISK_TONE, toRiskLevel } from "$lib/utils/tweakPresentation";
  import { untrack } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import WizardHeader from "./WizardHeader.svelte";

  const STEP_TITLE: Record<Step, string> = {
    select: "Select a profile file",
    review: "Review and configure",
    applying: "Applying changes…",
    complete: "Complete",
  };

  const isOpen = $derived(modalStore.current === "profileImport");

  // From the store, so Back (clear) and a failed apply land on the right step.
  function readView() {
    const { isApplying, applyResult, currentProfile: profile, validation } = profileStore;
    const step: Step = isApplying ? "applying" : applyResult ? "complete" : profile && validation ? "review" : "select";
    return { step, profile, validation, applyResult };
  }

  // Frozen while closed: Done and Cancel clear the store, and the exit animation must keep the last step.
  let lastView = readView();
  const view = $derived.by(() => (isOpen ? (lastView = readView()) : lastView));

  const step = $derived(view.step);
  const profile = $derived(view.profile);
  const validation = $derived(view.validation);
  const applyResult = $derived(view.applyResult);
  const isImporting = $derived(profileStore.isImporting);
  const applyProgress = $derived(profileStore.applyProgress);

  let skipAlreadyApplied = $state(true);
  const skipTweakIds = new SvelteSet<string>();
  let isDragOver = $state(false);

  const applicableTweaks = $derived(validation?.preview.filter((p) => p.applicable) ?? []);
  const isAutoSkipped = (p: { already_applied: boolean }) => skipAlreadyApplied && p.already_applied;
  const tweaksToApply = $derived(applicableTweaks.filter((p) => !skipTweakIds.has(p.tweak_id) && !isAutoSkipped(p)));

  const warnings = $derived<Issue[]>(
    (validation?.warnings ?? []).map((w) => ({ key: w.tweak_id + w.code, message: w.message })),
  );
  const errors = $derived<Issue[]>(
    (validation?.errors ?? []).map((e) => ({ key: e.tweak_id + e.code, message: e.message })),
  );
  const failures = $derived<Issue[]>(
    (applyResult?.failures ?? []).map((f) => ({
      key: f.tweak_id,
      message: f.error,
      tweakName: f.tweak_name,
      rolledBack: f.was_rolled_back,
    })),
  );

  // Open edge only: tracking store state here would reset options on every import.
  $effect(() => {
    if (!isOpen) return;
    untrack(() => {
      if (!profileStore.currentProfile) profileStore.clear();
      skipAlreadyApplied = true;
      skipTweakIds.clear();
    });
  });

  // Drops outside "select" would reset state mid-review or mid-apply.
  $effect(() => {
    if (!isOpen) return;
    return listenFileDrop({
      extension: `.${PROFILE_EXT}`,
      onOver: () => (isDragOver = step === "select"),
      onLeave: () => (isDragOver = false),
      onDrop: (path) => {
        if (step === "select") void reportImport(profileStore.importProfileFromPath(path));
      },
      onReject: () => {
        if (step === "select") toastStore.error(`Please select a .${PROFILE_EXT} profile file`);
      },
    });
  });

  function handleClose() {
    profileStore.clear();
    modalStore.close();
  }

  async function reportImport(importing: Promise<boolean>) {
    if (!(await importing) && profileStore.importError) toastStore.error(profileStore.importError);
  }

  function toggleSkipTweak(tweakId: string) {
    if (!skipTweakIds.delete(tweakId)) skipTweakIds.add(tweakId);
  }

  async function handleApply() {
    const success = await profileStore.applyProfile({ skipTweakIds: Array.from(skipTweakIds), skipAlreadyApplied });
    if (!success && profileStore.applyError) toastStore.error(profileStore.applyError);
  }

  async function handleFinish() {
    await bootStore.rescan();
    const result = applyResult;
    handleClose();

    if (result?.requires_reboot) {
      toastStore.warning("Some changes require a system restart to take effect", { duration: TOAST_DURATION.error });
    } else {
      toastStore.success(`Successfully applied ${plural(result?.applied_count ?? 0, "tweak")}`);
    }
  }
</script>

{#snippet issueList(tone: "warning" | "error", icon: IconName, heading: string, issues: Issue[], className = "")}
  <Callout {tone} density="panel" class={className}>
    <div class="flex items-center gap-2 text-sm font-medium {TONE_TEXT[tone]}">
      <Icon {icon} width={ICON_SIZE.lg} />
      {heading}
    </div>
    <ul class="m-0 mt-2 list-inside list-disc space-y-1 pl-1 text-sm text-foreground-muted">
      {#each issues.slice(0, MAX_LISTED) as issue (issue.key)}
        <li>
          {#if issue.tweakName}<span class="font-medium">{issue.tweakName}</span>:{/if}
          {issue.message}
          {#if issue.rolledBack}<span class="text-xs text-warning">(rolled back)</span>{/if}
        </li>
      {/each}
      {#if issues.length > MAX_LISTED}
        <li class={TONE_TEXT[tone]}>…and {issues.length - MAX_LISTED} more</li>
      {/if}
    </ul>
  </Callout>
{/snippet}

<Modal
  open={isOpen}
  onclose={handleClose}
  size="lg"
  closeOnEscape={step !== "applying"}
  closeOnBackdrop={step !== "applying"}
>
  <WizardHeader
    title="Import Profile"
    icon="mdi:import"
    subtitle={STEP_TITLE[step]}
    onclose={step !== "applying" ? handleClose : undefined}
  />

  <ModalBody>
    {#if step === "select"}
      <div class="animate-fade-in space-y-4">
        <button
          type="button"
          class="flex w-full flex-col items-center justify-center gap-3 rounded-lg border-2 border-dashed p-10 transition-colors
            {isDragOver ? 'border-accent bg-accent/10' : 'border-border hover:border-accent/50 hover:bg-muted/30'}"
          onclick={() => reportImport(profileStore.importProfile())}
        >
          <div
            class="flex h-16 w-16 items-center justify-center rounded-full {isDragOver ? 'bg-accent/20' : 'bg-muted'}"
          >
            <Icon
              icon="mdi:file-import"
              width={ICON_SIZE["2xl"]}
              class={isDragOver ? "text-accent" : "text-foreground-muted"}
            />
          </div>
          <div class="text-center">
            <p class="font-medium text-foreground">
              {isDragOver ? "Drop file here" : "Click to browse for a profile"}
            </p>
            <p class="mt-1 text-sm text-foreground-muted">or drag and drop a .{PROFILE_EXT} file</p>
          </div>
        </button>

        {#if isImporting}
          <div class="flex items-center justify-center gap-2 py-4">
            <Icon icon="mdi:loading" width="20" class="animate-spin text-accent" />
            <span class="text-sm text-foreground-muted">Loading profile…</span>
          </div>
        {/if}

        <Callout tone="neutral" density="panel" icon="mdi:information">
          <p class="m-0 text-xs leading-relaxed text-foreground-muted">
            Profile files (<code class="rounded bg-muted px-1">.{PROFILE_EXT}</code>) contain tweak configurations that
            can be applied to your system. The profile will be validated against your current Windows version and app.
          </p>
        </Callout>
      </div>
    {:else if step === "review" && profile && validation}
      <div class="animate-fade-in space-y-4">
        <div class="rounded-lg border border-border bg-surface p-4">
          <div class="flex items-start gap-3">
            <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-lg {TONE_SOFT.accent}">
              <Icon icon="mdi:file-document" width={ICON_SIZE.xl} />
            </div>
            <div class="min-w-0 flex-1">
              <h3 class="m-0 truncate text-base font-semibold text-foreground">{profile.metadata.name}</h3>
              {#if profile.metadata.description}
                <p class="m-0 mt-1 text-sm text-foreground-muted">{profile.metadata.description}</p>
              {/if}
              <div class="mt-2 flex flex-wrap items-center gap-2 text-xs text-foreground-muted">
                <span class="flex items-center gap-1">
                  <Icon icon="mdi:calendar" width={ICON_SIZE.sm} />
                  {formatDate(profile.metadata.created_at, { month: "short" }) || profile.metadata.created_at}
                </span>
                <span class="text-border">•</span>
                <span class="flex items-center gap-1">
                  <Icon icon="mdi:microsoft-windows" width={ICON_SIZE.sm} />
                  Windows {profile.metadata.source_windows_version}
                </span>
                <span class="text-border">•</span>
                <span>{plural(validation.stats.total_tweaks, "tweak")}</span>
              </div>
            </div>
          </div>
        </div>

        {#if warnings.length > 0}
          {@render issueList("warning", "mdi:alert", plural(warnings.length, "Warning"), warnings)}
        {/if}

        {#if errors.length > 0}
          {@render issueList(
            "error",
            "mdi:close-circle",
            `${plural(errors.length, "Error")} (will be skipped)`,
            errors,
          )}
        {/if}

        <div class="rounded-lg border border-border">
          <div class="flex items-center justify-between border-b border-border bg-muted/30 px-3 py-2">
            <span class="text-sm font-semibold text-foreground">Changes to Apply</span>
            <Badge>{plural(tweaksToApply.length, "tweak")}</Badge>
          </div>

          {#if applicableTweaks.length === 0}
            <div class="flex flex-col items-center justify-center gap-2 py-8 text-center">
              <Icon icon="mdi:alert-circle-outline" width={ICON_SIZE["2xl"]} class="text-foreground-muted" />
              <p class="text-sm text-foreground-muted">No applicable tweaks found in this profile.</p>
            </div>
          {:else}
            <div class="max-h-64 divide-y divide-border overflow-y-auto">
              {#each applicableTweaks as preview (preview.tweak_id)}
                {@const autoSkipped = isAutoSkipped(preview)}
                {@const included = !autoSkipped && !skipTweakIds.has(preview.tweak_id)}
                <button
                  type="button"
                  class="flex w-full items-center gap-3 px-3 py-2.5 text-left transition-colors hover:bg-muted/50 focus-visible:bg-muted/50 focus-visible:outline-none {included
                    ? ''
                    : 'opacity-50'}"
                  disabled={autoSkipped}
                  onclick={() => toggleSkipTweak(preview.tweak_id)}
                  aria-label="Toggle {preview.tweak_name}"
                >
                  <Checkbox checked={included} disabled={autoSkipped} />
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <span class="truncate text-sm font-medium text-foreground">{preview.tweak_name}</span>
                      {#if preview.already_applied}
                        <Badge class="shrink-0">Already Applied</Badge>
                      {/if}
                    </div>
                    <div class="mt-0.5 flex items-center gap-1 text-xs text-foreground-muted">
                      <span>{preview.current_option_label ?? "Default"}</span>
                      <Icon icon="mdi:arrow-right" width={ICON_SIZE["2xs"]} />
                      <span class="text-accent">{preview.target_option_label}</span>
                    </div>
                  </div>
                  <Badge tone={RISK_TONE[toRiskLevel(preview.risk_level)]} class="shrink-0">
                    {plural(preview.changes.length, "change")}
                  </Badge>
                </button>
              {/each}
            </div>
          {/if}
        </div>

        <div class="flex items-center justify-between rounded-lg border border-border bg-surface px-4 py-3">
          <span class="text-sm text-foreground">Skip already-applied tweaks</span>
          <Switch
            checked={skipAlreadyApplied}
            label="Skip already-applied tweaks"
            onchange={(v) => (skipAlreadyApplied = v)}
          />
        </div>
      </div>
    {:else if step === "applying"}
      <div class="flex animate-fade-in flex-col items-center justify-center gap-6 py-8">
        <div class="flex h-20 w-20 items-center justify-center rounded-full {TONE_SOFT.accent}">
          <Icon icon="mdi:cog" width={ICON_SIZE["3xl"]} class="animate-spin" />
        </div>

        <div class="w-full max-w-sm text-center">
          <p class="mb-4 font-medium text-foreground">Applying profile changes…</p>

          {#if applyProgress}
            <ProgressBar value={applyProgress.current} max={applyProgress.total} label="Applying profile" showValue />
            <p class="mt-2 text-sm text-foreground-muted">
              {applyProgress.current} of {plural(applyProgress.total, "tweak")}
            </p>
          {:else}
            <ProgressBar value={0} label="Applying profile" />
          {/if}
        </div>

        <p class="text-sm text-foreground-muted">Please wait, do not close this window…</p>
      </div>
    {:else if step === "complete" && applyResult}
      {@const tone = applyResult.success ? "success" : "warning"}
      <div class="flex animate-fade-in flex-col items-center justify-center gap-6 py-8">
        <div class="flex h-20 w-20 items-center justify-center rounded-full {TONE_SOFT[tone]}">
          <Icon icon={applyResult.success ? "mdi:check-circle" : "mdi:alert-circle"} width="48" />
        </div>

        <div class="text-center">
          <h3 class="m-0 text-xl font-bold text-foreground">
            {applyResult.success ? "Profile Applied!" : "Partially Applied"}
          </h3>
          <p class="m-0 mt-2 text-foreground-muted">
            Successfully applied {plural(applyResult.applied_count, "tweak")}
            {#if applyResult.skipped_count > 0}
              ({applyResult.skipped_count} skipped)
            {/if}
          </p>
        </div>

        {#if failures.length > 0}
          {@render issueList("error", "mdi:close-circle", `${failures.length} Failed`, failures, "w-full max-w-md")}
        {/if}

        {#if applyResult.requires_reboot}
          <Callout tone="warning" icon="mdi:restart">
            <span class="text-sm text-warning">Some changes require a system restart</span>
          </Callout>
        {/if}
      </div>
    {/if}
  </ModalBody>

  {#if step !== "applying"}
    <ModalFooter>
      {#if step === "select"}
        <Button variant="secondary" onclick={handleClose}>Cancel</Button>
        <Button
          variant="primary"
          onclick={() => reportImport(profileStore.importProfile())}
          disabled={isImporting}
          loading={isImporting}
        >
          <Icon icon="mdi:folder-open" width={ICON_SIZE.lg} />
          Browse Files
        </Button>
      {:else if step === "review"}
        <Button variant="secondary" onclick={() => profileStore.clear()}>
          <Icon icon="mdi:arrow-left" width={ICON_SIZE.lg} />
          Back
        </Button>
        <Button variant="primary" onclick={handleApply} disabled={tweaksToApply.length === 0}>
          <Icon icon="mdi:check" width={ICON_SIZE.lg} />
          Apply {plural(tweaksToApply.length, "Tweak")}
        </Button>
      {:else if step === "complete"}
        <Button variant="primary" onclick={handleFinish}>
          <Icon icon="mdi:check" width={ICON_SIZE.lg} />
          Done
        </Button>
      {/if}
    </ModalFooter>
  {/if}
</Modal>
