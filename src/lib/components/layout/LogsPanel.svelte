<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import type { LogLevel, LogSource } from "$lib/api/logs";
  import { Icon } from "$lib/components/shared";
  import { Badge, IconButton, SearchInput } from "$lib/components/ui";
  import { sidebarStore } from "$lib/stores/layout.svelte";
  import { formatLogLine, isGap, LOGS_PANEL_ID, LOGS_TOGGLE_ID, logsStore } from "$lib/stores/logs.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { systemStore } from "$lib/stores/tweaks.svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { fromAction, type Attachment } from "svelte/attachments";

  const RANK: Record<LogLevel, number> = { error: 0, warn: 1, info: 2, debug: 3, trace: 4 };
  const LEVEL_FILTERS: { value: LogLevel; label: string }[] = [
    { value: "trace", label: "All levels" },
    { value: "info", label: "Info and above" },
    { value: "warn", label: "Warnings and errors" },
    { value: "error", label: "Errors only" },
  ];
  const SOURCE_FILTERS: { value: "all" | LogSource; label: string }[] = [
    { value: "all", label: "All sources" },
    { value: "app", label: "App" },
    { value: "ui", label: "Interface" },
    { value: "helper", label: "Helper" },
  ];
  const LEVEL_CLASS: Record<LogLevel, string> = {
    error: "text-error",
    warn: "text-warning",
    info: "text-info",
    debug: "text-foreground-muted",
    trace: "text-foreground-subtle",
  };
  const selectClass =
    "h-8 rounded-md border border-border bg-surface px-2 text-xs text-foreground focus:border-accent focus:ring-2 focus:ring-accent/20 focus:outline-none";

  let minLevel = $state<LogLevel>("trace");
  let source = $state<"all" | LogSource>("all");
  let query = $state("");
  let stuck = $state(true);

  const needle = $derived(query.trim().toLowerCase());
  const visible = $derived(
    logsStore.rows.filter(
      (row) =>
        isGap(row) ||
        (RANK[row.level] <= RANK[minLevel] &&
          (source === "all" || row.source === source) &&
          (needle === "" || row.msgLower.includes(needle) || row.targetLower.includes(needle))),
    ),
  );
  const hasLines = $derived(visible.some((row) => !isGap(row)));

  // Re-runs whenever the visible rows change; reading `stuck` re-pins once the user scrolls back down.
  function follow(rows: unknown[]): Attachment<HTMLElement> {
    return (node) => {
      if (stuck && rows.length > 0) node.scrollTop = node.scrollHeight;
    };
  }

  const tip = (text: string) => fromAction(tooltip, () => text);

  function close() {
    logsStore.closePanel();
    document.getElementById(LOGS_TOGGLE_ID)?.focus();
  }

  const panel: Attachment<HTMLElement> = (node) => {
    stuck = true;
    node.focus({ preventScroll: true });
    const onKeydown = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      // The search box clears itself on the first Escape.
      if (e.target instanceof HTMLInputElement && e.target.value !== "") return;
      close();
    };
    node.addEventListener("keydown", onKeydown);
    return () => node.removeEventListener("keydown", onKeydown);
  };

  async function copyVisible() {
    const info = systemStore.info;
    const version = await getVersion().catch(() => "unknown");
    const header = `MagicX Toolbox ${version}, Windows build ${info?.windows.build_number ?? "unknown"}, elevated: ${
      info ? (info.is_admin ? "yes" : "no") : "unknown"
    }`;
    const lines = visible.map((row) => (isGap(row) ? `[${row.gap} lines skipped]` : formatLogLine(row)));
    try {
      await navigator.clipboard.writeText([header, ...lines].join("\n") + "\n");
      toastStore.success("Visible lines copied");
    } catch {
      toastStore.error("Could not copy the lines");
    }
  }
</script>

{#if logsStore.isPanelOpen}
  <section
    id={LOGS_PANEL_ID}
    aria-label="Logs"
    tabindex="-1"
    {@attach panel}
    class="fixed right-0 bottom-0 z-50 flex h-72 flex-col border-t border-border bg-background shadow-lg transition-[left] duration-250 ease-out outline-none {sidebarStore.contentLeftOffset}"
  >
    <div class="flex flex-wrap items-center gap-x-3 gap-y-1.5 border-b border-border bg-elevated px-3 py-1.5">
      <div class="flex items-center gap-2">
        <Icon icon="tabler:file-text" width="18" height="18" class="text-accent" />
        <h2 class="m-0 text-sm font-semibold">Logs</h2>
        {#if logsStore.settings?.detailed}
          <Badge variant="info">Detailed</Badge>
        {/if}
      </div>
      <p class="m-0 text-xs text-foreground-muted">This session. Earlier sessions: Export or Open folder.</p>

      <div class="ml-auto flex items-center gap-1.5">
        <select bind:value={minLevel} aria-label="Level" class={selectClass}>
          {#each LEVEL_FILTERS as f (f.value)}
            <option value={f.value}>{f.label}</option>
          {/each}
        </select>
        <select bind:value={source} aria-label="Source" class={selectClass}>
          {#each SOURCE_FILTERS as f (f.value)}
            <option value={f.value}>{f.label}</option>
          {/each}
        </select>
        <SearchInput value={query} placeholder="Search logs" class="w-48" onchange={(v) => (query = v)} />

        <IconButton
          icon="mdi:broom"
          size={16}
          aria-label="Clear view"
          {@attach tip("Clears this view. Saved logs are not changed.")}
          onclick={() => logsStore.clearView()}
        />
        <IconButton
          icon="mdi:content-copy"
          size={16}
          aria-label="Copy visible lines"
          {@attach tip("Copy visible lines")}
          disabled={!hasLines}
          onclick={copyVisible}
        />
        <IconButton
          icon="mdi:export"
          size={16}
          aria-label="Export diagnostics"
          {@attach tip("Export diagnostics")}
          disabled={logsStore.exporting}
          onclick={() => logsStore.exportDiagnostics()}
        />
        <IconButton
          icon="mdi:folder-open"
          size={16}
          aria-label="Open logs folder"
          {@attach tip("Open logs folder")}
          onclick={() => logsStore.openFolder()}
        />
        <IconButton icon="mdi:close" size={16} aria-label="Close logs" {@attach tip("Close logs")} onclick={close} />
      </div>
    </div>

    <div
      role="log"
      aria-live="off"
      aria-label="Log lines"
      class="flex-1 overflow-y-auto py-1 font-mono text-xs"
      onscroll={(e) => {
        const el = e.currentTarget;
        stuck = el.scrollHeight - el.scrollTop - el.clientHeight < 24;
      }}
      {@attach follow(visible)}
    >
      {#if !hasLines}
        <div class="flex h-full items-center justify-center px-3 text-center font-sans text-sm text-foreground-muted">
          {logsStore.rows.length === 0
            ? "No log lines yet. Lines appear here as the app works."
            : "No lines match the filters."}
        </div>
      {:else}
        {#each visible as row (isGap(row) ? `gap-${row.after}` : row.seq)}
          {#if isGap(row)}
            <div class="log-row px-3 py-0.5 text-center text-foreground-muted italic">
              {row.gap} lines skipped
            </div>
          {:else}
            <div class="log-row flex gap-2 px-3 py-0.5 hover:bg-foreground/5">
              <span class="shrink-0 text-foreground-muted">{row.ts.slice(11, 23)}</span>
              <span class="w-10 shrink-0 font-semibold uppercase {LEVEL_CLASS[row.level]}">{row.level}</span>
              <span class="w-11 shrink-0 text-foreground-muted">{row.source}</span>
              <span class="min-w-0 flex-1 wrap-break-word whitespace-pre-wrap text-foreground"
                ><span class="text-foreground-muted">{row.target.replace(/^app_lib::/, "")}:</span> {row.msg}</span
              >
            </div>
          {/if}
        {/each}
      {/if}
    </div>
  </section>
{/if}

<style>
  .log-row {
    content-visibility: auto;
    contain-intrinsic-size: auto 1.25rem;
  }
</style>
