<script lang="ts">
  import { closeTweakDetailsModal, tweakDetailsModalStore } from "$lib/stores/tweakDetailsModal";
  import { pendingChangesStore, systemStore, tweaksStore } from "$lib/stores/tweaks";
  import type { RegistryChange, SchedulerChange, TweakOption } from "$lib/types";
  import { RISK_INFO, type RiskLevel } from "$lib/types";
  import Icon from "./Icon.svelte";

  const isOpen = $derived($tweakDetailsModalStore !== null);

  const tweak = $derived.by(() => {
    const state = $tweakDetailsModalStore;
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

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      closeTweakDetailsModal();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && isOpen) {
      closeTweakDetailsModal();
    }
  }

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

<svelte:window onkeydown={handleKeydown} />

{#if isOpen && tweak}
  <div
    class="fixed inset-0 z-1000 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    role="presentation"
    onclick={handleBackdropClick}
  >
    <div
      class="animate-in zoom-in-95 w-[min(92vw,900px)] overflow-hidden rounded-xl border border-border bg-card shadow-xl duration-200 max-sm:w-[min(92vw,500px)]"
      role="dialog"
      aria-modal="true"
      aria-labelledby="tweak-details-title"
    >
      <!-- Header -->
      <div class="flex items-start justify-between gap-3 border-b border-border px-5 py-4">
        <div class="min-w-0">
          <h2 id="tweak-details-title" class="m-0 truncate text-lg font-bold text-foreground">
            {tweak.definition.name}
          </h2>
          <p class="m-0 mt-1 text-sm text-foreground-muted">{tweak.definition.description}</p>
        </div>

        <button
          class="flex h-8 w-8 shrink-0 cursor-pointer items-center justify-center rounded-lg border-0 bg-transparent text-foreground-muted transition-colors hover:bg-[hsl(var(--muted))] hover:text-foreground"
          onclick={closeTweakDetailsModal}
          aria-label="Close"
        >
          <Icon icon="mdi:close" width="20" />
        </button>
      </div>

      <!-- Body -->
      <div class="max-h-[calc(100dvh-2.5rem-6rem)] overflow-y-auto px-5 py-5">
        <!-- Summary -->
        <div class="flex flex-wrap gap-2">
          {#if riskInfo}
            <span
              class="inline-flex items-center gap-1.5 rounded-md bg-[hsl(var(--muted)/0.5)] px-2 py-1 text-xs text-foreground"
            >
              <Icon icon="mdi:alert" width="14" class="text-foreground-muted" />
              <span class="font-semibold tracking-wide uppercase">{riskInfo.name}</span>
            </span>
          {/if}

          {#if tweak.definition.requires_admin}
            <span
              class="inline-flex items-center gap-1.5 rounded-md bg-[hsl(var(--muted)/0.5)] px-2 py-1 text-xs text-foreground"
            >
              <Icon icon="mdi:shield-account-outline" width="14" class="text-foreground-muted" />
              <span class="font-semibold tracking-wide uppercase">Admin</span>
            </span>
          {/if}

          {#if tweak.definition.requires_system}
            <span
              class="inline-flex items-center gap-1.5 rounded-md bg-[hsl(var(--muted)/0.5)] px-2 py-1 text-xs text-foreground"
            >
              <Icon icon="mdi:shield-lock" width="14" class="text-foreground-muted" />
              <span class="font-semibold tracking-wide uppercase">System</span>
            </span>
          {/if}

          {#if tweak.definition.requires_ti}
            <span
              class="inline-flex items-center gap-1.5 rounded-md bg-[hsl(var(--muted)/0.5)] px-2 py-1 text-xs text-foreground"
            >
              <Icon icon="mdi:shield-key" width="14" class="text-foreground-muted" />
              <span class="font-semibold tracking-wide uppercase">TrustedInstaller</span>
            </span>
          {/if}

          {#if tweak.definition.requires_reboot}
            <span
              class="inline-flex items-center gap-1.5 rounded-md bg-[hsl(var(--muted)/0.5)] px-2 py-1 text-xs text-foreground"
            >
              <Icon icon="mdi:restart" width="14" class="text-foreground-muted" />
              <span class="font-semibold tracking-wide uppercase">Reboot</span>
            </span>
          {/if}

          <span
            class="inline-flex items-center gap-1.5 rounded-md bg-[hsl(var(--muted)/0.5)] px-2 py-1 text-xs text-foreground"
          >
            <Icon
              icon={tweak.status.is_applied ? "mdi:check-circle" : "mdi:circle-outline"}
              width="14"
              class="text-foreground-muted"
            />
            <span class="font-semibold tracking-wide uppercase"
              >{tweak.status.is_applied ? "Applied" : "Not applied"}</span
            >
          </span>
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
                  <span
                    class="inline-flex rounded bg-warning/15 px-2 py-1 text-[10px] font-semibold tracking-wide text-warning uppercase"
                  >
                    pending
                  </span>
                {:else if tweak.status.current_option_index === i}
                  <span
                    class="inline-flex rounded bg-accent/15 px-2 py-1 text-[10px] font-semibold tracking-wide text-accent uppercase"
                  >
                    current
                  </span>
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
                      <span
                        class="inline-flex h-4.5 min-w-4.5 items-center justify-center rounded-full bg-[hsl(var(--muted))] px-1.5 text-[10px] font-semibold text-foreground"
                      >
                        {option.pre_commands.length}
                      </span>
                    </h4>
                    <div class="space-y-2">
                      {#each option.pre_commands as cmd, idx (idx)}
                        <div class="rounded-md border border-border/60 bg-surface px-3 py-2">
                          <code class="block font-mono text-xs break-all whitespace-pre-wrap text-foreground"
                            >{cmd}</code
                          >
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
                      <span
                        class="inline-flex h-4.5 min-w-4.5 items-center justify-center rounded-full bg-[hsl(var(--muted))] px-1.5 text-[10px] font-semibold text-foreground"
                      >
                        {option.pre_powershell.length}
                      </span>
                    </h4>
                    <div class="space-y-2">
                      {#each option.pre_powershell as cmd, idx (idx)}
                        <div class="rounded-md border border-border/60 bg-surface px-3 py-2">
                          <code class="block font-mono text-xs break-all whitespace-pre-wrap text-foreground"
                            >{cmd}</code
                          >
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
                      <span
                        class="inline-flex h-4.5 min-w-4.5 items-center justify-center rounded-full bg-[hsl(var(--muted))] px-1.5 text-[10px] font-semibold text-foreground"
                      >
                        {option.registry_changes.length}
                      </span>
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
                                <span
                                  class="rounded bg-[hsl(var(--muted))] px-1.5 py-0.5 text-[9px] font-semibold text-foreground-muted"
                                >
                                  Win {change.windows_versions.join(",")}
                                </span>
                              {/if}
                              {#if !windowsApplies(change.windows_versions)}
                                <span
                                  class="rounded bg-warning/15 px-1.5 py-0.5 text-[9px] font-semibold text-warning uppercase"
                                  >not active</span
                                >
                              {/if}
                              {#if change.skip_validation}
                                <span
                                  class="rounded bg-[hsl(var(--muted))] px-1.5 py-0.5 text-[9px] font-semibold text-foreground-muted"
                                  >skip_validation</span
                                >
                              {/if}
                            </div>
                          </div>
                          <div class="px-3 py-2">
                            <div class="mb-1.5 flex flex-wrap items-center gap-2">
                              <span class="font-mono text-xs font-semibold text-foreground"
                                >{change.value_name || "(Default)"}</span
                              >
                              <span
                                class="rounded bg-[hsl(var(--muted))] px-1.5 py-0.5 font-mono text-[9px] text-foreground-muted"
                                >{change.value_type}</span
                              >
                            </div>
                            <div class="flex items-center gap-2 text-xs">
                              <span
                                class="rounded bg-accent/15 px-1.5 py-0.5 text-[9px] font-bold text-accent uppercase"
                                >Value</span
                              >
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
                      <span
                        class="inline-flex h-4.5 min-w-4.5 items-center justify-center rounded-full bg-[hsl(var(--muted))] px-1.5 text-[10px] font-semibold text-foreground"
                      >
                        {option.service_changes.length}
                      </span>
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
                              <span
                                class="rounded bg-[hsl(var(--muted))] px-1.5 py-0.5 text-[9px] font-semibold text-foreground-muted"
                              >
                                startup: {change.startup}
                              </span>
                              {#if change.skip_validation}
                                <span
                                  class="rounded bg-[hsl(var(--muted))] px-1.5 py-0.5 text-[9px] font-semibold text-foreground-muted"
                                  >skip_validation</span
                                >
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
                      <span
                        class="inline-flex h-4.5 min-w-4.5 items-center justify-center rounded-full bg-[hsl(var(--muted))] px-1.5 text-[10px] font-semibold text-foreground"
                      >
                        {option.scheduler_changes.length}
                      </span>
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
                                <span
                                  class="rounded bg-[hsl(var(--muted))] px-1.5 py-0.5 text-[9px] font-semibold text-foreground-muted"
                                >
                                  action: {change.action}
                                </span>
                                {#if change.ignore_not_found}
                                  <span
                                    class="rounded bg-[hsl(var(--muted))] px-1.5 py-0.5 text-[9px] font-semibold text-foreground-muted"
                                    >ignore_not_found</span
                                  >
                                {/if}
                                {#if change.skip_validation}
                                  <span
                                    class="rounded bg-[hsl(var(--muted))] px-1.5 py-0.5 text-[9px] font-semibold text-foreground-muted"
                                    >skip_validation</span
                                  >
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
                      <span
                        class="inline-flex h-4.5 min-w-4.5 items-center justify-center rounded-full bg-[hsl(var(--muted))] px-1.5 text-[10px] font-semibold text-foreground"
                      >
                        {option.post_commands.length}
                      </span>
                    </h4>
                    <div class="space-y-2">
                      {#each option.post_commands as cmd, idx (idx)}
                        <div class="rounded-md border border-border/60 bg-surface px-3 py-2">
                          <code class="block font-mono text-xs break-all whitespace-pre-wrap text-foreground"
                            >{cmd}</code
                          >
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
                      <span
                        class="inline-flex h-4.5 min-w-4.5 items-center justify-center rounded-full bg-[hsl(var(--muted))] px-1.5 text-[10px] font-semibold text-foreground"
                      >
                        {option.post_powershell.length}
                      </span>
                    </h4>
                    <div class="space-y-2">
                      {#each option.post_powershell as cmd, idx (idx)}
                        <div class="rounded-md border border-border/60 bg-surface px-3 py-2">
                          <code class="block font-mono text-xs break-all whitespace-pre-wrap text-foreground"
                            >{cmd}</code
                          >
                        </div>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>
            </section>
          {/each}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  @keyframes zoom-in-95 {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .animate-in {
    animation: zoom-in-95 0.2s ease-out;
  }
</style>
