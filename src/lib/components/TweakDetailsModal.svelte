<script lang="ts">
  import { closeTweakDetailsModal, tweakDetailsModalStore } from "$lib/stores/tweakDetailsModal.svelte";
  import { pendingChangesStore, systemStore, tweaksStore } from "$lib/stores/tweaks";
  import type { RegistryChange, SchedulerChange, TweakOption } from "$lib/types";
  import { RISK_INFO, type RiskLevel } from "$lib/types";
  import Icon from "./Icon.svelte";
  import { Badge, IconButton, Modal, ModalBody, ModalHeader } from "./ui";

  const isOpen = $derived(tweakDetailsModalStore.isOpen);

  const tweak = $derived.by(() => {
    const state = tweakDetailsModalStore.state;
    if (!state) return null;
    return $tweaksStore.find((t) => t.definition.id === state.tweakId) ?? null;
  });

  const pendingChange = $derived.by(() => {
    const t = tweak;
    if (!t) return undefined;
    return $pendingChangesStore.get(t.definition.id);
  });

  const currentWindowsVersion = $derived.by(() => {
    const system = $systemStore;
    if (!system) return null;
    return system.windows.is_windows_11 ? 11 : 10;
  });

  const riskInfo = $derived.by(() => {
    const t = tweak;
    if (!t) return null;
    return RISK_INFO[t.definition.risk_level as RiskLevel];
  });

  function formatRegistryPath(change: RegistryChange): string {
    return `${change.hive}\\${change.key}`;
  }

  function formatRegistryValue(value: unknown): string {
    if (value === null || value === undefined) return "(delete)";
    if (typeof value === "number") return `0x${value.toString(16).toUpperCase()} (${value})`;
    if (typeof value === "string") return value === "" ? '""' : `"${value}"`;
    return JSON.stringify(value);
  }

  function windowsApplies(windowsVersions: number[] | undefined): boolean {
    const current = currentWindowsVersion;
    if (!windowsVersions || windowsVersions.length === 0) return true;
    if (!current) return true;
    return windowsVersions.includes(current);
  }

  function optionLabel(optionIndex: number | null | undefined, options: TweakOption[]): string {
    if (optionIndex === null || optionIndex === undefined) return "System Default";
    const opt = options[optionIndex];
    return opt ? opt.label : `Option ${optionIndex}`;
  }

  function schedulerTarget(change: SchedulerChange): string {
    if (change.task_name) return `${change.task_path}\\${change.task_name}`;
    if (change.task_name_pattern) return `${change.task_path}\\(pattern: ${change.task_name_pattern})`;
    return change.task_path;
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

        {#if tweak.definition.requires_admin}
          <Badge variant="default" class="gap-1.5">
            <Icon icon="mdi:shield-account-outline" width="14" class="text-foreground-muted" />
            Admin
          </Badge>
        {/if}

        {#if tweak.definition.requires_system}
          <Badge variant="default" class="gap-1.5">
            <Icon icon="mdi:shield-lock" width="14" class="text-foreground-muted" />
            System
          </Badge>
        {/if}

        {#if tweak.definition.requires_ti}
          <Badge variant="default" class="gap-1.5">
            <Icon icon="mdi:shield-key" width="14" class="text-foreground-muted" />
            TrustedInstaller
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
      </div>

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
        {#each tweak.definition.options as option, i (i)}
          <section class="rounded-lg border border-border bg-background">
            <div
              class="flex items-center justify-between gap-2 border-b border-border/50 bg-[hsl(var(--muted)/0.3)] px-4 py-3"
            >
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
              <!-- Pre Commands -->
              {#if option.pre_commands.length > 0}
                <div>
                  <h4
                    class="m-0 mb-2 flex items-center gap-2 text-xs font-semibold tracking-wide text-foreground-muted uppercase"
                  >
                    <Icon icon="mdi:console" width="14" />
                    Pre Commands
                    <Badge size="sm">{option.pre_commands.length}</Badge>
                  </h4>
                  <div class="space-y-2">
                    {#each option.pre_commands as cmd, idx (idx)}
                      <div class="rounded-md border border-border/60 bg-surface px-3 py-2">
                        <code class="block font-mono text-xs break-all whitespace-pre-wrap text-foreground">{cmd}</code>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Pre PowerShell -->
              {#if option.pre_powershell.length > 0}
                <div>
                  <h4
                    class="m-0 mb-2 flex items-center gap-2 text-xs font-semibold tracking-wide text-foreground-muted uppercase"
                  >
                    <Icon icon="mdi:powershell" width="14" />
                    Pre PowerShell
                    <Badge size="sm">{option.pre_powershell.length}</Badge>
                  </h4>
                  <div class="space-y-2">
                    {#each option.pre_powershell as cmd, idx (idx)}
                      <div class="rounded-md border border-border/60 bg-surface px-3 py-2">
                        <code class="block font-mono text-xs break-all whitespace-pre-wrap text-foreground">{cmd}</code>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}

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
                      <div class="overflow-hidden rounded-lg border border-border/60 bg-background">
                        <div
                          class="flex flex-wrap items-center justify-between gap-2 border-b border-border/40 bg-[hsl(var(--muted)/0.3)] px-3 py-2"
                        >
                          <div class="flex min-w-0 items-center gap-2">
                            <Icon icon="mdi:key-variant" width="12" class="text-foreground-muted" />
                            <code class="bg-transparent p-0 font-mono text-[10px] break-all text-primary"
                              >{formatRegistryPath(change)}</code
                            >
                          </div>
                          <div class="flex items-center gap-2">
                            {#if change.windows_versions && change.windows_versions.length > 0}
                              <Badge size="sm" variant="default">Win {change.windows_versions.join(",")}</Badge>
                            {/if}
                            {#if !windowsApplies(change.windows_versions)}
                              <Badge size="sm" variant="warning">not active</Badge>
                            {/if}
                            {#if change.skip_validation}
                              <Badge size="sm" variant="default">skip_validation</Badge>
                            {/if}
                          </div>
                        </div>
                        <div class="px-3 py-2">
                          <div class="mb-1.5 flex flex-wrap items-center gap-2">
                            <span class="font-mono text-xs font-semibold text-foreground"
                              >{change.value_name || "(Default)"}</span
                            >
                            <Badge size="sm" variant="default">{change.value_type}</Badge>
                          </div>
                          <div class="flex items-center gap-2 text-xs">
                            <Badge size="sm" variant="info">Value</Badge>
                            <code class="bg-transparent p-0 font-mono text-[10px] text-foreground/80"
                              >{formatRegistryValue(change.value)}</code
                            >
                          </div>
                        </div>
                      </div>
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
                      <div class="rounded-lg border border-border/60 bg-background px-3 py-2">
                        <div class="flex flex-wrap items-center justify-between gap-2">
                          <div class="flex items-center gap-2">
                            <Icon icon="mdi:server" width="14" class="text-foreground-muted" />
                            <span class="font-mono text-xs font-semibold text-foreground">{change.name}</span>
                          </div>
                          <div class="flex items-center gap-2">
                            <Badge size="sm" variant="default">startup: {change.startup}</Badge>
                            {#if change.skip_validation}
                              <Badge size="sm" variant="default">skip_validation</Badge>
                            {/if}
                          </div>
                        </div>
                      </div>
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
                      <div class="rounded-lg border border-border/60 bg-background px-3 py-2">
                        <div class="flex flex-wrap items-center justify-between gap-2">
                          <div class="min-w-0">
                            <div class="flex items-center gap-2">
                              <Icon icon="mdi:calendar" width="14" class="text-foreground-muted" />
                              <code class="bg-transparent p-0 font-mono text-[10px] break-all text-foreground"
                                >{schedulerTarget(change)}</code
                              >
                            </div>
                            <div class="mt-1 flex flex-wrap items-center gap-2">
                              <Badge size="sm" variant="default">action: {change.action}</Badge>
                              {#if change.ignore_not_found}
                                <Badge size="sm" variant="default">ignore_not_found</Badge>
                              {/if}
                              {#if change.skip_validation}
                                <Badge size="sm" variant="default">skip_validation</Badge>
                              {/if}
                            </div>
                          </div>
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Post Commands -->
              {#if option.post_commands.length > 0}
                <div>
                  <h4
                    class="m-0 mb-2 flex items-center gap-2 text-xs font-semibold tracking-wide text-foreground-muted uppercase"
                  >
                    <Icon icon="mdi:console" width="14" />
                    Post Commands
                    <Badge size="sm">{option.post_commands.length}</Badge>
                  </h4>
                  <div class="space-y-2">
                    {#each option.post_commands as cmd, idx (idx)}
                      <div class="rounded-md border border-border/60 bg-surface px-3 py-2">
                        <code class="block font-mono text-xs break-all whitespace-pre-wrap text-foreground">{cmd}</code>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Post PowerShell -->
              {#if option.post_powershell.length > 0}
                <div>
                  <h4
                    class="m-0 mb-2 flex items-center gap-2 text-xs font-semibold tracking-wide text-foreground-muted uppercase"
                  >
                    <Icon icon="mdi:powershell" width="14" />
                    Post PowerShell
                    <Badge size="sm">{option.post_powershell.length}</Badge>
                  </h4>
                  <div class="space-y-2">
                    {#each option.post_powershell as cmd, idx (idx)}
                      <div class="rounded-md border border-border/60 bg-surface px-3 py-2">
                        <code class="block font-mono text-xs break-all whitespace-pre-wrap text-foreground">{cmd}</code>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          </section>
        {/each}
      </div>
    </ModalBody>
  {/if}
</Modal>
