<script lang="ts">
  import { autoScroll } from "$lib/attachments/autoScroll";
  import { Icon } from "$lib/components/shared";
  import { Badge, IconButton, SearchInput, Select, type SelectOption } from "$lib/components/ui";
  import { HEADING, type TextTone, TONE_TEXT } from "$lib/design";
  import { diagnosticsHeader } from "$lib/stores/diagnostics";
  import { formatLogLine, gapLabel, isGap, LOGS_PANEL_ID, LOGS_TOGGLE_ID, logsStore } from "$lib/stores/logs.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import type { LogLevel, LogSource } from "$lib/types";
  import { copyText } from "$lib/utils/clipboard";
  import { expand } from "$lib/utils/motion";
  import type { Attachment } from "svelte/attachments";

  type SourceFilter = "all" | LogSource;

  const RANK: Record<LogLevel, number> = { error: 0, warn: 1, info: 2, debug: 3, trace: 4 };
  const LEVEL_FILTERS: SelectOption<LogLevel>[] = [
    { value: "trace", label: "All levels" },
    { value: "info", label: "Info and above" },
    { value: "warn", label: "Warnings and errors" },
    { value: "error", label: "Errors only" },
  ];
  const SOURCE_FILTERS: SelectOption<SourceFilter>[] = [
    { value: "all", label: "All sources" },
    { value: "app", label: "App" },
    { value: "ui", label: "Interface" },
    { value: "helper", label: "Helper" },
  ];
  const LEVEL_TONE: Record<LogLevel, TextTone> = {
    error: "error",
    warn: "warning",
    info: "info",
    debug: "neutral",
    trace: "subtle",
  };
  let minLevel = $state<LogLevel>("trace");
  let source = $state<SourceFilter>("all");
  let query = $state("");

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

  function close() {
    logsStore.closePanel();
    document.getElementById(LOGS_TOGGLE_ID)?.focus();
  }

  const focusOnOpen: Attachment<HTMLElement> = (node) => node.focus({ preventScroll: true });

  // Delegated like the controls inside, so it runs after them: a native listener here would run first.
  function handleKeydown(e: KeyboardEvent) {
    // A control inside (an open dropdown, the search box clearing itself) already used this key.
    if (e.key !== "Escape" || e.defaultPrevented) return;
    e.preventDefault();
    close();
  }

  async function copyVisible() {
    const lines = visible.map((row) => (isGap(row) ? `[${gapLabel(row)}]` : formatLogLine(row)));
    const text = [diagnosticsHeader(), ...lines].join("\n") + "\n";
    if (await copyText(text)) toastStore.success("Visible lines copied");
    else toastStore.error("Could not copy the lines");
  }
</script>

{#if logsStore.isPanelOpen}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <section
    id={LOGS_PANEL_ID}
    aria-label="Logs"
    tabindex="-1"
    {@attach focusOnOpen}
    onkeydown={handleKeydown}
    in:expand={{ speed: "slow" }}
    out:expand
    class="flex h-72 max-h-9/20 min-h-40 shrink-0 flex-col border-t border-border bg-background outline-none"
  >
    <div class="flex flex-wrap items-center gap-x-3 gap-y-1.5 border-b border-border px-3 py-1.5">
      <div class="flex items-center gap-2">
        <Icon icon="tabler:file-text" size="lg" class="text-accent" />
        <h2 class={["m-0", HEADING.item]}>Logs</h2>
        {#if logsStore.settings?.detailed}
          <Badge tone="info">Detailed</Badge>
        {/if}
      </div>
      <p class="m-0 hidden text-xs text-foreground-muted logs-hint:block">
        This session. Earlier sessions: Export or Open folder.
      </p>

      <div class="ml-auto flex min-w-0 flex-wrap items-center gap-1.5">
        <Select value={minLevel} options={LEVEL_FILTERS} label="Level" class="w-32" onchange={(v) => (minLevel = v)} />
        <Select value={source} options={SOURCE_FILTERS} label="Source" class="w-32" onchange={(v) => (source = v)} />
        <SearchInput value={query} placeholder="Search logs" class="w-44 min-w-0" onchange={(v) => (query = v)} />

        <IconButton
          icon="mdi:broom"
          size="sm"
          label="Clear view"
          tooltip="Clears this view. Saved logs are not changed."
          onclick={() => logsStore.clearView()}
        />
        <IconButton
          icon="mdi:content-copy"
          size="sm"
          tooltip="Copy visible lines"
          disabled={!hasLines}
          onclick={copyVisible}
        />
        <IconButton
          icon="mdi:export"
          size="sm"
          tooltip="Export diagnostics"
          loading={logsStore.isExporting}
          onclick={() => logsStore.exportDiagnostics()}
        />
        <IconButton
          icon="mdi:folder-open"
          size="sm"
          tooltip="Open logs folder"
          onclick={() => logsStore.openFolder()}
        />
        <IconButton icon="mdi:close" size="sm" tooltip="Close logs" onclick={close} />
      </div>
    </div>

    <div
      role="log"
      aria-live="off"
      aria-label="Log lines"
      class="flex-1 overflow-y-auto py-1 font-mono text-xs"
      {@attach autoScroll}
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
            <div class="log-row px-3 py-0.5 text-center text-foreground-muted italic">{gapLabel(row)}</div>
          {:else}
            <div class="log-row flex gap-2 px-3 py-0.5 hover:bg-muted">
              <span class="shrink-0 text-foreground-muted">{row.time}</span>
              <span class="w-10 shrink-0 font-semibold uppercase {TONE_TEXT[LEVEL_TONE[row.level]]}">{row.level}</span>
              <span class="w-11 shrink-0 text-foreground-muted">{row.source}</span>
              <span class="min-w-0 flex-1 wrap-break-word whitespace-pre-wrap text-foreground"
                ><span class="text-foreground-muted">{row.module}:</span> {row.msg}</span
              >
            </div>
          {/if}
        {/each}
      {/if}
    </div>
  </section>
{/if}

<style>
  /* Skips layout and paint for off-screen rows of a long log; no utility sets these. */
  .log-row {
    content-visibility: auto;
    contain-intrinsic-size: auto 1.25rem;
  }
</style>
