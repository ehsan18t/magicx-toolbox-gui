<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { debugState, type DebugLogEntry } from "$lib/stores/debug.svelte";
  import { sidebarStore } from "$lib/stores/layout.svelte";

  let filterLevel = $state<"all" | DebugLogEntry["level"]>("all");
  let autoScroll = $state(true);
  let logContainer: HTMLDivElement | undefined = $state();

  // Derived filtered logs
  const filteredLogs = $derived(
    filterLevel === "all" ? debugState.logs : debugState.logs.filter((l) => l.level === filterLevel),
  );

  // Auto-scroll effect
  $effect(() => {
    if (autoScroll && logContainer && debugState.logs.length > 0) {
      logContainer.scrollTo({ top: 0, behavior: "smooth" });
    }
  });

  const levelConfig = {
    info: { icon: "tabler:info-circle", color: "text-info", bg: "bg-info/10" },
    warn: { icon: "tabler:alert-triangle", color: "text-warning", bg: "bg-warning/10" },
    error: { icon: "tabler:x-circle", color: "text-error", bg: "bg-error/10" },
    success: { icon: "tabler:check-circle", color: "text-success", bg: "bg-success/10" },
  };

  function formatTime(date: Date): string {
    return date.toLocaleTimeString("en-US", {
      hour12: false,
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  }
</script>

{#if debugState.isPanelOpen}
  <div
    class="fixed right-0 bottom-0 z-50 flex h-72 flex-col border-t border-border bg-background shadow-lg transition-[left] duration-250 ease-out {sidebarStore.contentLeftOffset}"
  >
    <!-- Header -->
    <div class="flex items-center justify-between border-b border-border bg-elevated px-3 py-2">
      <div class="flex items-center gap-3">
        <div class="flex items-center gap-1.5">
          <Icon icon="tabler:bug" width="18" height="18" class="text-warning" />
          <span class="text-sm font-semibold">Debug Console</span>
        </div>

        <!-- Filter buttons -->
        <div class="flex items-center gap-1 rounded-md bg-background p-0.5">
          <button
            type="button"
            aria-pressed={filterLevel === "all"}
            onclick={() => (filterLevel = "all")}
            class="rounded px-2 py-0.5 text-xs transition-colors {filterLevel === 'all'
              ? 'bg-accent text-white'
              : 'text-foreground-muted hover:bg-foreground/5'}"
          >
            All ({debugState.logCounts.total})
          </button>
          <button
            type="button"
            aria-pressed={filterLevel === "info"}
            onclick={() => (filterLevel = "info")}
            class="rounded px-2 py-0.5 text-xs transition-colors {filterLevel === 'info'
              ? 'bg-info text-white'
              : 'text-info hover:bg-info/10'}"
          >
            Info ({debugState.logCounts.info})
          </button>
          <button
            type="button"
            aria-pressed={filterLevel === "success"}
            onclick={() => (filterLevel = "success")}
            class="rounded px-2 py-0.5 text-xs transition-colors {filterLevel === 'success'
              ? 'bg-success text-white'
              : 'text-success hover:bg-success/10'}"
          >
            Success ({debugState.logCounts.success})
          </button>
          <button
            type="button"
            aria-pressed={filterLevel === "warn"}
            onclick={() => (filterLevel = "warn")}
            class="rounded px-2 py-0.5 text-xs transition-colors {filterLevel === 'warn'
              ? 'bg-warning text-white'
              : 'text-warning hover:bg-warning/10'}"
          >
            Warn ({debugState.logCounts.warn})
          </button>
          <button
            type="button"
            aria-pressed={filterLevel === "error"}
            onclick={() => (filterLevel = "error")}
            class="rounded px-2 py-0.5 text-xs transition-colors {filterLevel === 'error'
              ? 'bg-error text-white'
              : 'text-error hover:bg-error/10'}"
          >
            Error ({debugState.logCounts.error})
          </button>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <button
          type="button"
          aria-pressed={autoScroll}
          onclick={() => (autoScroll = !autoScroll)}
          use:tooltip={autoScroll ? "Auto-scroll ON" : "Auto-scroll OFF"}
          class="rounded p-1 transition-colors hover:bg-foreground/10 {autoScroll
            ? 'text-accent'
            : 'text-foreground-muted'}"
        >
          <Icon icon="tabler:arrow-bar-to-down" width="16" height="16" />
        </button>
        <button
          type="button"
          onclick={() => debugState.clear()}
          use:tooltip={"Clear logs"}
          class="rounded p-1 text-foreground-muted transition-colors hover:bg-foreground/10 hover:text-foreground"
        >
          <Icon icon="tabler:trash" width="16" height="16" />
        </button>
        <button
          type="button"
          onclick={() => debugState.closePanel()}
          use:tooltip={"Close panel"}
          class="rounded p-1 text-foreground-muted transition-colors hover:bg-foreground/10 hover:text-foreground"
        >
          <Icon icon="tabler:x" width="16" height="16" />
        </button>
      </div>
    </div>

    <!-- Logs -->
    <div bind:this={logContainer} class="flex-1 overflow-y-auto font-mono text-xs">
      {#if filteredLogs.length === 0}
        <div class="flex h-full items-center justify-center text-foreground-muted">
          <div class="flex flex-col items-center gap-2">
            <Icon icon="tabler:file-search" width="32" height="32" />
            <span>No logs yet. Apply or revert a tweak to see registry changes.</span>
          </div>
        </div>
      {:else}
        {#each filteredLogs as log (log.id)}
          {@const config = levelConfig[log.level]}
          <div class="flex items-start gap-2 border-b border-border/50 px-3 py-1.5 hover:bg-foreground/5 {config.bg}">
            <Icon icon={config.icon} width="14" height="14" class="{config.color} mt-0.5 shrink-0" />
            <span class="shrink-0 text-foreground-muted">{formatTime(log.timestamp)}</span>
            <span class="shrink-0 rounded bg-foreground/10 px-1.5 py-0.5 text-[10px] font-medium uppercase">
              {log.source}
            </span>
            <span class="shrink-0 font-medium text-accent">[{log.action}]</span>
            <span class="flex-1 text-foreground">{log.details}</span>
            {#if log.data}
              <span class="shrink-0 text-foreground-muted">
                {JSON.stringify(log.data)}
              </span>
            {/if}
          </div>
        {/each}
      {/if}
    </div>
  </div>
{/if}
