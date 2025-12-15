<script lang="ts">
  import { getBackupInfo, type BackupInfo } from "$lib/api/tweaks";
  import { closeTweakDetailsModal, tweakDetailsModalStore } from "$lib/stores/tweakDetailsModal.svelte";
  import { pendingChangesStore, systemStore, tweaksStore } from "$lib/stores/tweaks.svelte";
  import type { TweakOption } from "$lib/types";
  import { getHighestPermission, PERMISSION_INFO, RISK_INFO, type RiskLevel } from "$lib/types";
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
      getBackupInfo(t.definition.id)
        .then((info) => {
          snapshotInfo = info;
        })
        .catch(() => {
          snapshotInfo = null;
        });
    } else {
      snapshotInfo = null;
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

  function optionLabel(optionIndex: number | null | undefined, options: TweakOption[]): string {
    if (optionIndex === null || optionIndex === undefined) return "System Default";
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
      <!-- Summary badges -->
      <div class="flex flex-wrap gap-2">
        {#if riskInfo}
          <Badge variant="default" class="gap-1.5">
            <Icon icon="mdi:alert" width="14" class="text-foreground-muted" />
            {riskInfo.name}
          </Badge>
        {/if}

        <!-- Permission level (only show highest: ti > system > admin) -->
        {#if permissionInfo}
          <Badge variant="default" class="gap-1.5">
            <Icon icon={permissionInfo.icon} width="14" class="text-foreground-muted" />
            {permissionInfo.name}
          </Badge>
        {/if}

        {#if tweak.definition.requires_reboot}
          <Badge variant="default" class="gap-1.5">
            <Icon icon="mdi:restart" width="14" class="text-foreground-muted" />
            Reboot
          </Badge>
        {/if}

        <Badge variant={tweak.status.is_applied ? "success" : "default"} class="gap-1.5">
          <Icon
            icon={tweak.status.is_applied ? "mdi:check-circle" : "mdi:circle-outline"}
            width="14"
            class="text-foreground-muted"
          />
          {tweak.status.is_applied ? "Applied" : "Not applied"}
        </Badge>

        {#if tweak.status.has_backup}
          <Badge variant="info" class="gap-1.5">
            <Icon icon="mdi:history" width="14" class="text-foreground-muted" />
            Snapshot available
          </Badge>
        {/if}
      </div>

      {#if tweak.status.has_backup && snapshotInfo}
        <div class="mt-4 rounded-lg border border-accent/30 bg-accent/5 p-3">
          <div class="flex items-start gap-2">
            <Icon icon="mdi:backup-restore" width="16" class="mt-0.5 shrink-0 text-accent" />
            <div class="text-sm">
              <span class="font-medium text-foreground">Snapshot saved</span>
              <span class="text-foreground-muted">
                — Original state captured with {snapshotInfo.registry_values_count} registry
                {snapshotInfo.registry_values_count === 1 ? "value" : "values"}
                {#if snapshotInfo.service_snapshots_count > 0}
                  and {snapshotInfo.service_snapshots_count}
                  {snapshotInfo.service_snapshots_count === 1 ? "service" : "services"}
                {/if}
              </span>
            </div>
          </div>
        </div>
      {/if}

      {#if tweak.definition.info}
        <div class="mt-4 rounded-lg border border-border/50 bg-surface/50 p-3">
          <div class="flex items-start gap-2">
            <Icon icon="mdi:information-outline" width="16" class="mt-0.5 shrink-0 text-accent" />
            <p class="m-0 text-sm leading-relaxed text-foreground-muted">{tweak.definition.info}</p>
          </div>
        </div>
      {/if}

      <div class="mt-4 rounded-lg border border-border bg-surface p-4">
        <div class="flex flex-wrap gap-4">
          <div class="min-w-60">
            <div class="text-xs font-semibold tracking-wide text-foreground-muted uppercase">Current option</div>
            <div class="mt-1 text-sm text-foreground">
              {optionLabel(tweak.status.current_option_index ?? null, tweak.definition.options)}
            </div>
          </div>
          <div class="min-w-60">
            <div class="text-xs font-semibold tracking-wide text-foreground-muted uppercase">Pending option</div>
            <div class="mt-1 text-sm text-foreground">
              {#if pendingChange}
                {optionLabel(pendingChange.optionIndex, tweak.definition.options)}
              {:else}
                <span class="text-foreground-muted">None</span>
              {/if}
            </div>
          </div>
          <div class="min-w-60">
            <div class="text-xs font-semibold tracking-wide text-foreground-muted uppercase">Windows</div>
            <div class="mt-1 text-sm text-foreground">
              {#if currentWindowsVersion}
                Windows {currentWindowsVersion}
              {:else}
                <span class="text-foreground-muted">Unknown</span>
              {/if}
            </div>
          </div>
        </div>
      </div>

      <!-- Options + Changes -->
      <div class="mt-5 space-y-4">
        {#each tweak.definition.options as option, i (option.label)}
          <section class="rounded-lg border border-border bg-background">
            <div class="bg-muted/30 flex items-center justify-between gap-2 border-b border-border/50 px-4 py-3">
              <div class="min-w-0">
                <div class="text-xs font-semibold tracking-wide text-foreground-muted uppercase">Option {i + 1}</div>
                <div class="mt-0.5 truncate text-sm font-semibold text-foreground">{option.label}</div>
              </div>

              {#if pendingChange?.optionIndex === i}
                <Badge variant="warning" size="sm">pending</Badge>
              {:else if tweak.status.current_option_index === i}
                <Badge variant="info" size="sm">current</Badge>
              {/if}
            </div>

            <div class="space-y-4 px-4 py-4">
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
          </section>
        {/each}
      </div>
    </ModalBody>
  {/if}
</Modal>
