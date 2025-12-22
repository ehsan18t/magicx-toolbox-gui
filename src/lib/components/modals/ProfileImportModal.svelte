<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import {
    Badge,
    Button,
    Checkbox,
    IconButton,
    Modal,
    ModalBody,
    ModalFooter,
    ModalHeader,
    ProgressBar,
    Switch,
  } from "$lib/components/ui";
  import { closeModal, modalStore } from "$lib/stores/modal.svelte";
  import { profileStore } from "$lib/stores/profile.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { tweaksStore } from "$lib/stores/tweaks.svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { SvelteSet } from "svelte/reactivity";

  const isOpen = $derived(modalStore.current === "profileImport");

  // Wizard step state
  type Step = "select" | "review" | "applying" | "complete";
  let step = $state<Step>("select");

  // Options
  let skipAlreadyApplied = $state(true);
  let createRestorePoint = $state(true);

  // Tweaks to skip - use const since we only mutate, never reassign
  const skipTweakIds = new SvelteSet<string>();

  // Drag state
  let isDragOver = $state(false);

  // Derived state from profile store
  const profile = $derived(profileStore.currentProfile);
  const validation = $derived(profileStore.validation);
  const isImporting = $derived(profileStore.isImporting);
  const applyProgress = $derived(profileStore.applyProgress);
  const applyResult = $derived(profileStore.applyResult);

  // Computed values
  const applicableTweaks = $derived.by(() => {
    if (!validation) return [];
    return validation.preview.filter((p) => p.applicable);
  });

  const tweaksToApply = $derived.by(() => {
    return applicableTweaks.filter((p) => !skipTweakIds.has(p.tweak_id) && !(skipAlreadyApplied && p.already_applied));
  });

  const warnings = $derived(validation?.warnings ?? []);
  const errors = $derived(validation?.errors ?? []);

  // Reset state when modal opens
  $effect(() => {
    if (isOpen) {
      if (profileStore.currentProfile) {
        step = "review";
      } else {
        step = "select";
        profileStore.clear();
      }
      // Always reset options
      skipAlreadyApplied = true;
      createRestorePoint = true;
      skipTweakIds.clear();
    }
  });

  // Set up native drag-drop listener when modal is open
  $effect(() => {
    if (!isOpen) return;

    let cancelled = false;
    let unlisten: (() => void) | undefined;

    getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === "over") {
          isDragOver = true;
        } else if (event.payload.type === "drop") {
          isDragOver = false;
          const paths = event.payload.paths;
          if (paths && paths.length > 0) {
            const filePath = paths[0];
            if (filePath.endsWith(".mgx")) {
              handleDroppedFile(filePath);
            } else {
              toastStore.show("error", "Please select a .mgx profile file");
            }
          }
        } else {
          // cancelled
          isDragOver = false;
        }
      })
      .then((fn) => {
        // If effect was cancelled before promise resolved, immediately clean up
        if (cancelled) {
          fn();
        } else {
          unlisten = fn;
        }
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  });

  // Auto-advance to review when import completes
  $effect(() => {
    if (profile && validation && step === "select") {
      step = "review";
    }
  });

  // Auto-advance to complete when apply finishes
  $effect(() => {
    if (applyResult && step === "applying") {
      step = "complete";
    }
  });

  function handleClose() {
    profileStore.clear();
    closeModal();
  }

  async function handleBrowse() {
    await profileStore.importProfile();
    if (profileStore.importError) {
      toastStore.show("error", profileStore.importError);
    }
  }

  async function handleDroppedFile(filePath: string) {
    const success = await profileStore.importProfileFromPath(filePath);
    if (!success && profileStore.importError) {
      toastStore.show("error", profileStore.importError);
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    // The native Tauri drag-drop event handles the visual state
  }

  function handleDragLeave() {
    // The native Tauri drag-drop event handles the visual state
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    // Native drag-drop is handled by Tauri's onDragDropEvent
    // This is here to prevent default browser behavior
  }

  function toggleSkipTweak(tweakId: string) {
    if (skipTweakIds.has(tweakId)) {
      skipTweakIds.delete(tweakId);
    } else {
      skipTweakIds.add(tweakId);
    }
  }

  async function handleApply() {
    step = "applying";

    const success = await profileStore.applyProfile({
      skipTweakIds: Array.from(skipTweakIds),
      skipAlreadyApplied,
      createRestorePoint,
    });

    if (!success && profileStore.applyError) {
      toastStore.show("error", profileStore.applyError);
      step = "review"; // Go back to review on error
    }
  }

  async function handleFinish() {
    // Reload tweaks to reflect changes
    await tweaksStore.load();
    handleClose();

    if (applyResult?.requires_reboot) {
      toastStore.show("warning", "Some changes require a system restart to take effect", { duration: 5000 });
    } else {
      toastStore.show("success", `Successfully applied ${applyResult?.applied_count ?? 0} tweaks`);
    }
  }

  function formatDate(dateString: string): string {
    try {
      return new Date(dateString).toLocaleDateString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
      });
    } catch {
      return dateString;
    }
  }
</script>

<Modal
  open={isOpen}
  onclose={handleClose}
  size="lg"
  closeOnEscape={step !== "applying"}
  labelledBy="import-modal-title"
>
  <ModalHeader id="import-modal-title">
    <div class="flex items-center gap-3">
      <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-accent/15">
        <Icon icon="mdi:import" width="24" class="text-accent" />
      </div>
      <div>
        <h2 class="m-0 text-lg font-bold text-foreground">Import Profile</h2>
        <p class="m-0 text-sm text-foreground-muted">
          {#if step === "select"}
            Select a profile file
          {:else if step === "review"}
            Review and configure
          {:else if step === "applying"}
            Applying changes...
          {:else}
            Complete
          {/if}
        </p>
      </div>
    </div>
    {#if step !== "applying"}
      <IconButton icon="mdi:close" onclick={handleClose} aria-label="Close" />
    {/if}
  </ModalHeader>

  <ModalBody scrollable maxHeight="calc(100dvh - 14rem)">
    {#if step === "select"}
      <!-- Step 1: File Selection -->
      <div class="space-y-4">
        <!-- Drop zone -->
        <button
          type="button"
          class="flex w-full flex-col items-center justify-center gap-3 rounded-xl border-2 border-dashed p-10 transition-colors
            {isDragOver ? 'border-accent bg-accent/10' : 'hover:bg-muted/30 border-border hover:border-accent/50'}"
          onclick={handleBrowse}
          ondragover={handleDragOver}
          ondragleave={handleDragLeave}
          ondrop={handleDrop}
        >
          <div
            class="flex h-16 w-16 items-center justify-center rounded-full {isDragOver ? 'bg-accent/20' : 'bg-muted'}"
          >
            <Icon icon="mdi:file-import" width="32" class={isDragOver ? "text-accent" : "text-foreground-muted"} />
          </div>
          <div class="text-center">
            <p class="font-medium text-foreground">
              {isDragOver ? "Drop file here" : "Click to browse for a profile"}
            </p>
            <p class="mt-1 text-sm text-foreground-muted">or drag and drop a .mgx file</p>
          </div>
        </button>

        {#if isImporting}
          <div class="flex items-center justify-center gap-2 py-4">
            <Icon icon="mdi:loading" width="20" class="animate-spin text-accent" />
            <span class="text-sm text-foreground-muted">Loading profile...</span>
          </div>
        {/if}

        <!-- Info -->
        <div class="flex items-start gap-3 rounded-lg border border-border/50 bg-surface/50 p-3">
          <Icon icon="mdi:information" width="18" class="mt-0.5 shrink-0 text-accent" />
          <p class="m-0 text-xs leading-relaxed text-foreground-muted">
            Profile files (<code class="bg-muted rounded px-1">.mgx</code>) contain tweak configurations that can be
            applied to your system. The profile will be validated against your current Windows version and app.
          </p>
        </div>
      </div>
    {:else if step === "review" && profile && validation}
      <!-- Step 2: Review -->
      <div class="space-y-4">
        <!-- Profile info -->
        <div class="rounded-lg border border-border bg-surface p-4">
          <div class="flex items-start gap-3">
            <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-lg bg-accent/15">
              <Icon icon="mdi:file-document" width="24" class="text-accent" />
            </div>
            <div class="min-w-0 flex-1">
              <h3 class="m-0 truncate text-base font-semibold text-foreground">{profile.metadata.name}</h3>
              {#if profile.metadata.description}
                <p class="m-0 mt-1 text-sm text-foreground-muted">{profile.metadata.description}</p>
              {/if}
              <div class="mt-2 flex flex-wrap items-center gap-2 text-xs text-foreground-muted">
                <span class="flex items-center gap-1">
                  <Icon icon="mdi:calendar" width="14" />
                  {formatDate(profile.metadata.created_at)}
                </span>
                <span class="text-border">•</span>
                <span class="flex items-center gap-1">
                  <Icon icon="mdi:microsoft-windows" width="14" />
                  Windows {profile.metadata.source_windows_version}
                </span>
                <span class="text-border">•</span>
                <span>{validation.stats.total_tweaks} tweaks</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Warnings -->
        {#if warnings.length > 0}
          <div class="rounded-lg border border-warning/30 bg-warning/10 p-3">
            <div class="flex items-center gap-2 text-sm font-medium text-warning">
              <Icon icon="mdi:alert" width="18" />
              {warnings.length} Warning{warnings.length > 1 ? "s" : ""}
            </div>
            <ul class="m-0 mt-2 list-inside list-disc space-y-1 pl-1 text-sm text-foreground-muted">
              {#each warnings.slice(0, 5) as warning (warning.tweak_id + warning.code)}
                <li>{warning.message}</li>
              {/each}
              {#if warnings.length > 5}
                <li class="text-warning">...and {warnings.length - 5} more</li>
              {/if}
            </ul>
          </div>
        {/if}

        <!-- Errors -->
        {#if errors.length > 0}
          <div class="rounded-lg border border-error/30 bg-error/10 p-3">
            <div class="flex items-center gap-2 text-sm font-medium text-error">
              <Icon icon="mdi:close-circle" width="18" />
              {errors.length} Error{errors.length > 1 ? "s" : ""} (will be skipped)
            </div>
            <ul class="m-0 mt-2 list-inside list-disc space-y-1 pl-1 text-sm text-foreground-muted">
              {#each errors.slice(0, 5) as error (error.tweak_id + error.code)}
                <li>{error.message}</li>
              {/each}
              {#if errors.length > 5}
                <li class="text-error">...and {errors.length - 5} more</li>
              {/if}
            </ul>
          </div>
        {/if}

        <!-- Tweaks to apply -->
        <div class="rounded-lg border border-border">
          <div class="bg-muted/30 flex items-center justify-between border-b border-border px-3 py-2">
            <span class="text-sm font-semibold text-foreground">Changes to Apply</span>
            <Badge variant="default">{tweaksToApply.length} tweaks</Badge>
          </div>

          {#if applicableTweaks.length === 0}
            <div class="flex flex-col items-center justify-center gap-2 py-8 text-center">
              <Icon icon="mdi:alert-circle-outline" width="32" class="text-foreground-muted" />
              <p class="text-sm text-foreground-muted">No applicable tweaks found in this profile.</p>
            </div>
          {:else}
            <div class="max-h-64 divide-y divide-border overflow-y-auto">
              {#each applicableTweaks as preview (preview.tweak_id)}
                {@const isSkipped =
                  skipTweakIds.has(preview.tweak_id) || (skipAlreadyApplied && preview.already_applied)}
                {@const isDisabled = skipAlreadyApplied && preview.already_applied}
                <button
                  type="button"
                  class="hover:bg-muted/50 focus-visible:bg-muted/50 flex w-full items-center gap-3 px-3 py-2.5 text-left transition-colors focus-visible:outline-none {isSkipped
                    ? 'opacity-50'
                    : ''}"
                  disabled={isDisabled}
                  onclick={() => toggleSkipTweak(preview.tweak_id)}
                  aria-label="Toggle {preview.tweak_name}"
                >
                  <Checkbox
                    checked={!skipTweakIds.has(preview.tweak_id) && !(skipAlreadyApplied && preview.already_applied)}
                    disabled={isDisabled}
                    ariaLabel="{preview.tweak_name} selection"
                  />
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <span class="truncate text-sm font-medium text-foreground">{preview.tweak_name}</span>
                      {#if preview.already_applied}
                        <Badge variant="default" class="shrink-0 text-xs">Already Applied</Badge>
                      {/if}
                    </div>
                    <div class="mt-0.5 flex items-center gap-1 text-xs text-foreground-muted">
                      <span>{preview.current_option_label ?? "Default"}</span>
                      <Icon icon="mdi:arrow-right" width="12" />
                      <span class="text-accent">{preview.target_option_label}</span>
                    </div>
                  </div>
                  <Badge
                    variant="default"
                    class="shrink-0 text-xs {preview.risk_level === 'moderate'
                      ? 'bg-warning/15 text-warning'
                      : preview.risk_level === 'advanced'
                        ? 'bg-error/15 text-error'
                        : ''}"
                  >
                    {preview.changes.length} changes
                  </Badge>
                </button>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Options -->
        <div class="space-y-3">
          <div class="flex items-center justify-between rounded-lg border border-border bg-surface px-4 py-3">
            <span class="text-sm text-foreground">Skip already-applied tweaks</span>
            <Switch checked={skipAlreadyApplied} onchange={(v) => (skipAlreadyApplied = v)} />
          </div>
          <div class="flex items-center justify-between rounded-lg border border-border bg-surface px-4 py-3">
            <div>
              <span class="block text-sm text-foreground">Create restore points</span>
              <span class="text-xs text-foreground-muted"
                >Backs up current values before applying each tweak for easy undo</span
              >
            </div>
            <Switch checked={createRestorePoint} onchange={(v) => (createRestorePoint = v)} />
          </div>
        </div>
      </div>
    {:else if step === "applying"}
      <!-- Step 3: Applying -->
      <div class="flex flex-col items-center justify-center gap-6 py-8">
        <div class="flex h-20 w-20 items-center justify-center rounded-full bg-accent/15">
          <Icon icon="mdi:cog" width="40" class="animate-spin text-accent" />
        </div>

        <div class="w-full max-w-sm text-center">
          <p class="mb-4 font-medium text-foreground">Applying profile changes...</p>

          {#if applyProgress}
            <ProgressBar value={applyProgress.current} max={applyProgress.total} size="lg" showLabel />
            <p class="mt-2 text-sm text-foreground-muted">
              {applyProgress.current} of {applyProgress.total} tweaks
            </p>
          {:else}
            <ProgressBar value={0} max={100} size="lg" />
          {/if}
        </div>

        <p class="text-sm text-foreground-muted">Please wait, do not close this window...</p>
      </div>
    {:else if step === "complete" && applyResult}
      <!-- Step 4: Complete -->
      <div class="flex flex-col items-center justify-center gap-6 py-8">
        <div
          class="flex h-20 w-20 items-center justify-center rounded-full {applyResult.success
            ? 'bg-success/15'
            : 'bg-warning/15'}"
        >
          <Icon
            icon={applyResult.success ? "mdi:check-circle" : "mdi:alert-circle"}
            width="48"
            class={applyResult.success ? "text-success" : "text-warning"}
          />
        </div>

        <div class="text-center">
          <h3 class="m-0 text-xl font-bold text-foreground">
            {applyResult.success ? "Profile Applied!" : "Partially Applied"}
          </h3>
          <p class="m-0 mt-2 text-foreground-muted">
            Successfully applied {applyResult.applied_count} tweak{applyResult.applied_count !== 1 ? "s" : ""}
            {#if applyResult.skipped_count > 0}
              ({applyResult.skipped_count} skipped)
            {/if}
          </p>
        </div>

        {#if applyResult.failures.length > 0}
          <div class="w-full max-w-md rounded-lg border border-error/30 bg-error/10 p-3">
            <div class="flex items-center gap-2 text-sm font-medium text-error">
              <Icon icon="mdi:close-circle" width="18" />
              {applyResult.failures.length} Failed
            </div>
            <ul class="m-0 mt-2 list-inside list-disc space-y-1 pl-1 text-sm text-foreground-muted">
              {#each applyResult.failures.slice(0, 5) as failure (failure.tweak_id)}
                <li>
                  <span class="font-medium">{failure.tweak_name}</span>: {failure.error}
                  {#if failure.was_rolled_back}
                    <span class="text-xs text-warning">(rolled back)</span>
                  {/if}
                </li>
              {/each}
            </ul>
          </div>
        {/if}

        {#if applyResult.requires_reboot}
          <div class="flex items-center gap-2 rounded-lg border border-warning/30 bg-warning/10 px-4 py-3">
            <Icon icon="mdi:restart" width="20" class="text-warning" />
            <span class="text-sm text-warning">Some changes require a system restart</span>
          </div>
        {/if}
      </div>
    {/if}
  </ModalBody>

  {#if step !== "applying"}
    <ModalFooter>
      {#if step === "select"}
        <Button variant="secondary" onclick={handleClose}>Cancel</Button>
        <Button variant="primary" onclick={handleBrowse} disabled={isImporting} loading={isImporting}>
          <Icon icon="mdi:folder-open" width="18" />
          Browse Files
        </Button>
      {:else if step === "review"}
        <Button variant="secondary" onclick={() => (step = "select")}>
          <Icon icon="mdi:arrow-left" width="18" />
          Back
        </Button>
        <Button variant="primary" onclick={handleApply} disabled={tweaksToApply.length === 0}>
          <Icon icon="mdi:check" width="18" />
          Apply {tweaksToApply.length} Tweak{tweaksToApply.length !== 1 ? "s" : ""}
        </Button>
      {:else if step === "complete"}
        <Button variant="primary" onclick={handleFinish}>
          <Icon icon="mdi:check" width="18" />
          Done
        </Button>
      {/if}
    </ModalFooter>
  {/if}
</Modal>
