<script lang="ts">
  import { getBackupInfo, inspectTweak, type BackupInfo } from "$lib/api/tweaks";
  import { closeTweakDetailsModal, tweakDetailsModalStore } from "$lib/stores/tweakDetailsModal.svelte";
  import { pendingChangesStore, systemStore, tweaksStore } from "$lib/stores/tweaks.svelte";
  import type { TweakOption } from "$lib/types";
  import { getHighestPermission, PERMISSION_INFO, RISK_INFO, type RiskLevel, type TweakInspection } from "$lib/types";
  import Icon from "./Icon.svelte";
  import { CommandList, RegistryChangeItem, SchedulerChangeItem, ServiceChangeItem } from "./tweak-details";
  import { Badge, IconButton, Modal, ModalBody, ModalHeader } from "./ui";

  const isOpen = $derived(tweakDetailsModalStore.isOpen);

  const tweak = $derived.by(() => {
    const state = tweakDetailsModalStore.state;
    if (!state) return null;
    return tweaksStore.list.find((t) => t.definition.id === state.tweakId) ?? null;
  });

  // Load snapshot info when modal opens with a tweak that has a backup
  let snapshotInfo = $state<BackupInfo | null>(null);

  $effect(() => {
    const t = tweak;
    if (isOpen && t?.status.has_backup) {
      let cancelled = false;
      getBackupInfo(t.definition.id)
        .then((info) => {
          if (!cancelled) snapshotInfo = info;
        })
        .catch(() => {
          if (!cancelled) snapshotInfo = null;
        });
      return () => {
        cancelled = true;
      };
    } else {
      snapshotInfo = null;
    }
  });

  // Inspection State
  let inspection = $state<TweakInspection | null>(null);
  let isInspecting = $state(false);
  let inspectionError = $state<string | null>(null);
  let showInspectionDetails = $state(false);

  $effect(() => {
    const t = tweak;
    // Reset state when tweak changes or modal closes
    if (!isOpen || !t || inspection?.tweak_id !== t.definition.id) {
      inspection = null;
      isInspecting = false;
      inspectionError = null;
      showInspectionDetails = false;

      if (isOpen && t) {
        // load inspection
        isInspecting = true;
        let cancelled = false;
        inspectTweak(t.definition.id)
          .then((res) => {
            if (!cancelled) {
              inspection = res;
              isInspecting = false;
            }
          })
          .catch((err) => {
            if (!cancelled) {
              console.error("Inspection failed", err);
              inspectionError = "Failed to analyze system state";
              isInspecting = false;
            }
          });
        return () => {
          cancelled = true;
        };
      }
    }
  });

  const pendingChange = $derived.by(() => {
    const t = tweak;
    if (!t) return undefined;
    return pendingChangesStore.get(t.definition.id);
  });

  const currentWindowsVersion = $derived.by(() => {
    const system = systemStore.info;
    if (!system) return null;
    return system.windows.is_windows_11 ? 11 : 10;
  });

  const riskInfo = $derived.by(() => {
    const t = tweak;
    if (!t) return null;
    return RISK_INFO[t.definition.risk_level as RiskLevel];
  });

  // Get highest permission level (hierarchy: ti > system > admin > none)
  const highestPermission = $derived.by(() => {
    const t = tweak;
    if (!t) return "none" as const;
    return getHighestPermission(t.definition);
  });
  const permissionInfo = $derived(highestPermission !== "none" ? PERMISSION_INFO[highestPermission] : null);

  // Derive inspection summary stats
  const inspectionSummary = $derived.by(() => {
    if (!inspection) return null;

    const matchedOption = inspection.options.find((o) => o.all_match);
    const totalChecks = inspection.options.reduce(
      (sum, o) => sum + o.registry_results.length + o.service_results.length + o.scheduler_results.length,
      0,
    );

    // Count mismatches for the current/pending option or first option
    // Count mismatches for the current/pending option or first option
    const pendingIdx = pendingChange?.optionIndex;
    const relevantOption =
      inspection.options.find((o) => o.is_current || (pendingIdx !== undefined && o.option_index === pendingIdx)) ??
      inspection.options[0];
    const mismatches = relevantOption
      ? relevantOption.registry_results.filter((r) => !r.is_match).length +
        relevantOption.service_results.filter((s) => !s.is_match).length +
        relevantOption.scheduler_results.filter((s) => !s.is_match).length
      : 0;

    return {
      matchedOption,
      totalChecks,
      mismatches,
      hasCustomState: !matchedOption,
    };
  });

  function optionLabel(optionIndex: number | null, options: TweakOption[]): string {
    if (optionIndex === null) return "Custom Configuration";
    const opt = options[optionIndex];
    return opt ? opt.label : `Option ${optionIndex}`;
  }
</script>

<Modal open={isOpen && !!tweak} onclose={closeTweakDetailsModal} size="lg">
  {#if tweak}
    <ModalHeader>
      <div class="min-w-0">
        <h2 class="m-0 truncate text-lg font-bold text-foreground">
          {tweak.definition.name}
        </h2>
        <p class="m-0 mt-1 text-sm text-foreground-muted">{tweak.definition.description}</p>
      </div>
      <IconButton icon="mdi:close" onclick={closeTweakDetailsModal} aria-label="Close" />
    </ModalHeader>

    <ModalBody scrollable class="max-h-[calc(100dvh-2.5rem-6rem)]">
      <!-- Status Overview Card -->
      <div class="rounded-xl border border-border bg-surface/50 p-4">
        <!-- Status Row -->
        <div class="flex flex-wrap items-center gap-2">
          <!-- Applied Status - Primary indicator -->
          <div
            class="flex items-center gap-2 rounded-lg px-3 py-1.5 {tweak.status.is_applied
              ? 'bg-success/10 text-success'
              : 'bg-muted text-foreground-muted'}"
          >
            <Icon icon={tweak.status.is_applied ? "mdi:check-circle" : "mdi:circle-outline"} width="16" />
            <span class="text-sm font-medium">{tweak.status.is_applied ? "Applied" : "Not Applied"}</span>
          </div>

          <!-- Risk Level -->
          {#if riskInfo}
            <div class="bg-muted flex items-center gap-1.5 rounded-lg px-3 py-1.5">
              <Icon icon="mdi:shield-alert-outline" width="14" class="text-foreground-muted" />
              <span class="text-sm text-foreground-muted">{riskInfo.name} Risk</span>
            </div>
          {/if}

          <!-- Permission -->
          {#if permissionInfo}
            <div class="bg-muted flex items-center gap-1.5 rounded-lg px-3 py-1.5">
              <Icon icon={permissionInfo.icon} width="14" class="text-foreground-muted" />
              <span class="text-sm text-foreground-muted">{permissionInfo.name}</span>
            </div>
          {/if}

          <!-- Reboot -->
          {#if tweak.definition.requires_reboot}
            <div class="flex items-center gap-1.5 rounded-lg bg-info/10 px-3 py-1.5 text-info">
              <Icon icon="mdi:restart" width="14" />
              <span class="text-sm">Reboot Required</span>
            </div>
          {/if}

          <!-- Snapshot -->
          {#if tweak.status.has_backup}
            <div class="flex items-center gap-1.5 rounded-lg bg-accent/10 px-3 py-1.5 text-accent">
              <Icon icon="mdi:history" width="14" />
              <span class="text-sm">Snapshot Available</span>
            </div>
          {/if}
        </div>

        <!-- Current Configuration -->
        <div class="mt-4 flex flex-wrap gap-6 border-t border-border/50 pt-4">
          <div>
            <div class="text-[11px] font-medium tracking-wide text-foreground-muted uppercase">Current</div>
            <div class="mt-0.5 text-sm font-medium text-foreground">
              {optionLabel(tweak.status.current_option_index ?? null, tweak.definition.options)}
            </div>
          </div>
          {#if pendingChange}
            <div>
              <div class="text-[11px] font-medium tracking-wide text-warning uppercase">Pending</div>
              <div class="mt-0.5 text-sm font-medium text-warning">
                {optionLabel(pendingChange.optionIndex, tweak.definition.options)}
              </div>
            </div>
          {/if}
          {#if currentWindowsVersion}
            <div>
              <div class="text-[11px] font-medium tracking-wide text-foreground-muted uppercase">System</div>
              <div class="mt-0.5 text-sm text-foreground">Windows {currentWindowsVersion}</div>
            </div>
          {/if}
        </div>
      </div>

      <!-- System State Analysis -->
      <div class="mt-4 overflow-hidden rounded-xl border border-border">
        <button
          class="flex w-full items-center justify-between bg-surface/50 px-4 py-3 text-left transition-colors hover:bg-surface"
          onclick={() => (showInspectionDetails = !showInspectionDetails)}
        >
          <div class="flex items-center gap-3">
            <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-accent/10">
              <Icon icon="mdi:clipboard-check-outline" width="18" class="text-accent" />
            </div>
            <div>
              <div class="text-sm font-semibold text-foreground">System State</div>
              {#if isInspecting}
                <div class="text-xs text-foreground-muted">Analyzing...</div>
              {:else if inspectionError}
                <div class="text-xs text-error">{inspectionError}</div>
              {:else if inspectionSummary}
                <div class="text-xs text-foreground-muted">
                  {#if inspectionSummary.matchedOption}
                    <span class="text-success">Matches "{inspectionSummary.matchedOption.label}"</span>
                  {:else}
                    <span class="text-warning">Custom configuration detected</span>
                  {/if}
                </div>
              {/if}
            </div>
          </div>
          <div class="flex items-center gap-2">
            {#if isInspecting}
              <Icon icon="mdi:loading" width="18" class="animate-spin text-foreground-muted" />
            {:else if inspectionSummary?.matchedOption}
              <Badge variant="success" size="sm">
                <Icon icon="mdi:check" width="12" />
                Match
              </Badge>
            {:else if inspectionSummary?.hasCustomState}
              <Badge variant="warning" size="sm">
                <Icon icon="mdi:alert" width="12" />
                Custom
              </Badge>
            {/if}
            <Icon
              icon="mdi:chevron-down"
              width="18"
              class="text-foreground-muted transition-transform duration-200 {showInspectionDetails
                ? 'rotate-180'
                : ''}"
            />
          </div>
        </button>

        {#if showInspectionDetails && inspection}
          <div class="border-t border-border/50 bg-background">
            {#each inspection.options as opt, optIndex (`inspection-option-${optIndex}`)}
              <div class="border-b border-border/30 last:border-b-0">
                <!-- Option Header -->
                <div class="bg-muted/30 flex items-center gap-3 px-4 py-2.5">
                  <div
                    class="flex h-6 w-6 items-center justify-center rounded-full text-xs font-bold {opt.all_match
                      ? 'bg-success/15 text-success'
                      : 'bg-muted text-foreground-muted'}"
                  >
                    {#if opt.all_match}
                      <Icon icon="mdi:check" width="14" />
                    {:else}
                      {optIndex + 1}
                    {/if}
                  </div>
                  <span class="text-sm font-medium text-foreground">{opt.label}</span>
                  {#if opt.all_match}
                    <Badge variant="success" size="sm">Current State</Badge>
                  {:else if pendingChange?.optionIndex === optIndex}
                    <Badge variant="warning" size="sm">Pending</Badge>
                  {/if}
                </div>

                <!-- Check Results -->
                <div class="space-y-1 px-4 py-3">
                  {#each opt.registry_results as reg, regIndex (`registry-result-${regIndex}`)}
                    <div class="flex items-start gap-2 rounded-lg px-2 py-1.5 {reg.is_match ? '' : 'bg-error/5'}">
                      <Icon
                        icon={reg.is_match ? "mdi:check-circle" : "mdi:close-circle"}
                        width="14"
                        class="mt-0.5 shrink-0 {reg.is_match ? 'text-success' : 'text-error'}"
                      />
                      <div class="min-w-0 flex-1 text-xs">
                        <div class="font-medium text-foreground">{reg.description}</div>
                        {#if !reg.is_match}
                          <div class="mt-1 flex gap-4 font-mono text-[11px]">
                            <span class="text-foreground-muted">
                              Expected: <span class="text-success">{JSON.stringify(reg.expected_value)}</span>
                            </span>
                            <span class="text-foreground-muted">
                              Actual: <span class="text-error">{JSON.stringify(reg.actual_value ?? "Missing")}</span>
                            </span>
                          </div>
                        {/if}
                      </div>
                    </div>
                  {/each}

                  {#each opt.service_results as svc, svcIndex (`service-result-${svcIndex}`)}
                    <div class="flex items-start gap-2 rounded-lg px-2 py-1.5 {svc.is_match ? '' : 'bg-error/5'}">
                      <Icon
                        icon={svc.is_match ? "mdi:check-circle" : "mdi:close-circle"}
                        width="14"
                        class="mt-0.5 shrink-0 {svc.is_match ? 'text-success' : 'text-error'}"
                      />
                      <div class="min-w-0 flex-1 text-xs">
                        <div class="font-medium text-foreground">Service: {svc.name}</div>
                        {#if !svc.is_match}
                          <div class="mt-1 flex gap-4 font-mono text-[11px]">
                            <span class="text-foreground-muted">
                              Expected: <span class="text-success">{svc.expected_startup}</span>
                            </span>
                            <span class="text-foreground-muted">
                              Actual: <span class="text-error">{svc.actual_startup ?? "Unknown"}</span>
                            </span>
                          </div>
                        {/if}
                      </div>
                    </div>
                  {/each}

                  {#each opt.scheduler_results as task, taskIndex (`scheduler-result-${taskIndex}`)}
                    <div class="flex items-start gap-2 rounded-lg px-2 py-1.5 {task.is_match ? '' : 'bg-error/5'}">
                      <Icon
                        icon={task.is_match ? "mdi:check-circle" : "mdi:close-circle"}
                        width="14"
                        class="mt-0.5 shrink-0 {task.is_match ? 'text-success' : 'text-error'}"
                      />
                      <div class="min-w-0 flex-1 text-xs">
                        <div class="font-medium text-foreground">Task: {task.task_name}</div>
                        <div class="truncate text-[10px] text-foreground-muted/70">{task.task_path}</div>
                        {#if !task.is_match}
                          <div class="mt-1 flex gap-4 font-mono text-[11px]">
                            <span class="text-foreground-muted">
                              Expected: <span class="text-success">{task.expected_state}</span>
                            </span>
                            <span class="text-foreground-muted">
                              Actual: <span class="text-error">{task.actual_state ?? "Not Found"}</span>
                            </span>
                          </div>
                        {/if}
                      </div>
                    </div>
                  {/each}

                  {#if opt.registry_results.length === 0 && opt.service_results.length === 0 && opt.scheduler_results.length === 0}
                    <div class="px-2 py-1.5 text-xs text-foreground-muted italic">
                      No detectable changes for this Windows version
                    </div>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Info Box -->
      {#if tweak.definition.info}
        <div class="mt-4 flex items-start gap-3 rounded-xl border border-border/50 bg-surface/30 p-4">
          <Icon icon="mdi:information-outline" width="18" class="mt-0.5 shrink-0 text-accent" />
          <p class="m-0 text-sm leading-relaxed text-foreground-muted">{tweak.definition.info}</p>
        </div>
      {/if}

      <!-- Snapshot Info -->
      {#if tweak.status.has_backup && snapshotInfo}
        <div class="mt-4 flex items-start gap-3 rounded-xl border border-accent/30 bg-accent/5 p-4">
          <Icon icon="mdi:backup-restore" width="18" class="mt-0.5 shrink-0 text-accent" />
          <div class="text-sm">
            <span class="font-medium text-foreground">Snapshot saved</span>
            <span class="text-foreground-muted">
              — {snapshotInfo.registry_values_count} registry
              {snapshotInfo.registry_values_count === 1 ? "value" : "values"}
              {#if snapshotInfo.service_snapshots_count > 0}
                , {snapshotInfo.service_snapshots_count}
                {snapshotInfo.service_snapshots_count === 1 ? "service" : "services"}
              {/if}
              {#if snapshotInfo.scheduler_snapshots_count > 0}
                , {snapshotInfo.scheduler_snapshots_count}
                {snapshotInfo.scheduler_snapshots_count === 1 ? "task" : "tasks"}
              {/if}
              captured
            </span>
          </div>
        </div>
      {/if}

      <!-- Options + Changes -->
      <div class="mt-6">
        <h3 class="mb-3 flex items-center gap-2 text-sm font-semibold text-foreground">
          <Icon icon="mdi:tune-variant" width="16" class="text-foreground-muted" />
          Configuration Options
        </h3>

        <div class="space-y-3">
          {#each tweak.definition.options as option, i (option.label)}
            {@const isCurrent = tweak.status.current_option_index === i}
            {@const isPending = pendingChange?.optionIndex === i}
            {@const hasChanges =
              option.registry_changes.length > 0 ||
              option.service_changes.length > 0 ||
              option.scheduler_changes.length > 0 ||
              option.pre_commands.length > 0 ||
              option.post_commands.length > 0 ||
              option.pre_powershell.length > 0 ||
              option.post_powershell.length > 0}

            <section
              class="overflow-hidden rounded-xl border transition-colors {isCurrent
                ? 'border-accent/40 bg-accent/3'
                : isPending
                  ? 'border-warning/40 bg-warning/3'
                  : 'border-border bg-background'}"
            >
              <div class="flex items-center justify-between gap-3 px-4 py-3">
                <div class="flex items-center gap-3">
                  <div
                    class="flex h-7 w-7 items-center justify-center rounded-lg text-xs font-bold {isCurrent
                      ? 'bg-accent/15 text-accent'
                      : isPending
                        ? 'bg-warning/15 text-warning'
                        : 'bg-muted text-foreground-muted'}"
                  >
                    {i + 1}
                  </div>
                  <span class="text-sm font-semibold text-foreground">{option.label}</span>
                </div>

                <div class="flex items-center gap-2">
                  {#if isCurrent}
                    <Badge variant="accent" size="sm">Current</Badge>
                  {/if}
                  {#if isPending}
                    <Badge variant="warning" size="sm">Pending</Badge>
                  {/if}
                </div>
              </div>

              {#if hasChanges}
                <div class="space-y-4 border-t border-border/50 bg-surface/30 px-4 py-4">
                  <CommandList title="Pre Commands" commands={option.pre_commands} icon="mdi:console" />
                  <CommandList title="Pre PowerShell" commands={option.pre_powershell} icon="mdi:powershell" />

                  <!-- Registry Changes -->
                  {#if option.registry_changes.length > 0}
                    <div>
                      <h4
                        class="m-0 mb-2 flex items-center gap-2 text-xs font-semibold tracking-wide text-foreground-muted uppercase"
                      >
                        <Icon icon="mdi:database-cog-outline" width="14" />
                        Registry Changes
                        <Badge size="sm">{option.registry_changes.length}</Badge>
                      </h4>
                      <div class="space-y-2">
                        {#each option.registry_changes as change, idx (idx)}
                          <RegistryChangeItem {change} {currentWindowsVersion} />
                        {/each}
                      </div>
                    </div>
                  {/if}

                  <!-- Service Changes -->
                  {#if option.service_changes.length > 0}
                    <div>
                      <h4
                        class="m-0 mb-2 flex items-center gap-2 text-xs font-semibold tracking-wide text-foreground-muted uppercase"
                      >
                        <Icon icon="mdi:cog-outline" width="14" />
                        Service Changes
                        <Badge size="sm">{option.service_changes.length}</Badge>
                      </h4>
                      <div class="space-y-2">
                        {#each option.service_changes as change, idx (idx)}
                          <ServiceChangeItem {change} />
                        {/each}
                      </div>
                    </div>
                  {/if}

                  <!-- Scheduler Changes -->
                  {#if option.scheduler_changes.length > 0}
                    <div>
                      <h4
                        class="m-0 mb-2 flex items-center gap-2 text-xs font-semibold tracking-wide text-foreground-muted uppercase"
                      >
                        <Icon icon="mdi:calendar-clock" width="14" />
                        Scheduled Tasks
                        <Badge size="sm">{option.scheduler_changes.length}</Badge>
                      </h4>
                      <div class="space-y-2">
                        {#each option.scheduler_changes as change, idx (idx)}
                          <SchedulerChangeItem {change} />
                        {/each}
                      </div>
                    </div>
                  {/if}

                  <CommandList title="Post Commands" commands={option.post_commands} icon="mdi:console" />
                  <CommandList title="Post PowerShell" commands={option.post_powershell} icon="mdi:powershell" />
                </div>
              {:else}
                <div class="border-t border-border/50 bg-surface/30 px-4 py-3 text-xs text-foreground-muted italic">
                  No changes configured for this option
                </div>
              {/if}
            </section>
          {/each}
        </div>
      </div>
    </ModalBody>
  {/if}
</Modal>
