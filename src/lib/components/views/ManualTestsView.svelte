<script lang="ts">
  import type { ManualTest, ManualTestStatus } from "$lib/api/manualTests";
  import { ConfirmDialog } from "$lib/components/modals";
  import { Icon } from "$lib/components/shared";
  import { Badge, Button, Card, Spinner } from "$lib/components/ui";
  import { manualTestsStore } from "$lib/stores/manualTests.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import type { Attachment } from "svelte/attachments";

  let minutes = $state<Record<string, number>>({});
  let confirming = $state<ManualTest | null>(null);

  const runningId = $derived(manualTestsStore.runningId);

  const statusBadge: Record<ManualTestStatus, { variant: "success" | "error" | "info"; label: string }> = {
    pass: { variant: "success", label: "Pass" },
    fail: { variant: "error", label: "Fail" },
    info: { variant: "info", label: "Info" },
  };

  function minutesFor(test: ManualTest): number | null {
    return test.minutes === null ? null : (minutes[test.id] ?? test.minutes);
  }

  function requestRun(test: ManualTest) {
    if (test.changes_system) {
      confirming = test;
    } else {
      void manualTestsStore.run(test.id, minutesFor(test));
    }
  }

  function confirmRun() {
    const test = confirming;
    confirming = null;
    if (test) void manualTestsStore.run(test.id, minutesFor(test));
  }

  async function copyReport(id: string) {
    try {
      await navigator.clipboard.writeText(manualTestsStore.reportFor(id));
      toastStore.success("Report copied");
    } catch (e) {
      console.error("Copy failed:", e);
      toastStore.error("Could not copy the report");
    }
  }

  // Re-runs whenever the line count changes, keeping the newest line in view.
  function stickToBottom(lines: number): Attachment<HTMLElement> {
    return (node) => {
      if (lines > 0) node.scrollTop = node.scrollHeight;
    };
  }
</script>

<div class="flex h-full flex-col gap-5 overflow-y-auto p-6">
  <header class="flex items-center gap-4">
    <div class="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-warning/15 text-warning">
      <Icon icon="mdi:flask-outline" width="28" />
    </div>
    <div>
      <h1 class="m-0 text-2xl font-bold tracking-tight text-foreground">Manual Tests</h1>
      <p class="mt-1 mb-0 text-sm text-foreground-muted">
        Test build only. Run the app as administrator, run one test at a time, then copy its report.
      </p>
    </div>
  </header>

  {#each manualTestsStore.tests as test (test.id)}
    {@const isRunning = runningId === test.id}
    {@const result = manualTestsStore.resultFor(test.id)}
    {@const failure = manualTestsStore.failureFor(test.id)}
    {@const log = manualTestsStore.logFor(test.id)}
    <Card>
      <section class="flex flex-col gap-3" aria-labelledby="manual-test-{test.id}">
        <div class="flex flex-wrap items-center gap-2">
          <h2 id="manual-test-{test.id}" class="m-0 text-base font-semibold text-foreground">{test.title}</h2>
          <Badge variant={test.changes_system ? "warning" : "default"}>
            {test.changes_system ? "Changes this PC" : "Read-only"}
          </Badge>
          <code class="text-xs text-foreground-muted">{test.id}</code>
        </div>
        <p class="m-0 text-sm text-foreground-muted">{test.description}</p>
        <p class="m-0 text-sm text-foreground">
          <span class="font-semibold">What this changes on this PC:</span>
          {test.changes}
        </p>

        <div class="flex flex-wrap items-center gap-3">
          {#if test.minutes !== null}
            <label class="flex items-center gap-2 text-sm text-foreground">
              Duration (minutes)
              <input
                type="number"
                min="1"
                max="1440"
                class="w-20 rounded-md border border-border bg-surface px-2 py-1 text-sm text-foreground focus:border-accent focus:outline-none"
                disabled={runningId !== null}
                bind:value={() => minutesFor(test) ?? 1, (v) => (minutes[test.id] = Math.max(1, Math.floor(v || 1)))}
              />
            </label>
          {/if}
          <Button
            variant={test.changes_system ? "warning" : "primary"}
            size="sm"
            disabled={runningId !== null}
            aria-label="Run {test.title}"
            onclick={() => requestRun(test)}
          >
            <Icon icon="mdi:play" width="16" />
            Run
          </Button>
          {#if isRunning && test.minutes !== null}
            <Button
              variant="outline"
              size="sm"
              loading={manualTestsStore.cancelling}
              aria-label="Cancel {test.title}"
              onclick={() => manualTestsStore.cancel()}
            >
              <Icon icon="mdi:stop" width="16" />
              {manualTestsStore.cancelling ? "Stopping and restoring" : "Cancel"}
            </Button>
          {/if}
          {#if result || failure || log.length > 0}
            <Button
              variant="outline"
              size="sm"
              disabled={isRunning}
              aria-label="Copy the report for {test.title}"
              onclick={() => copyReport(test.id)}
            >
              <Icon icon="mdi:content-copy" width="16" />
              Copy report
            </Button>
          {/if}
          {#if isRunning}
            <span class="flex items-center gap-2 text-sm text-foreground-muted">
              <Spinner size="sm" ariaLabel="Running {test.title}" />
              Running
            </span>
          {/if}
        </div>

        {#if result}
          {@const badge = statusBadge[result.status]}
          <div class="flex flex-col gap-1.5" role="status">
            <div class="flex items-start gap-2">
              <Badge variant={badge.variant} size="md">{badge.label}</Badge>
              <p class="m-0 text-sm font-medium text-foreground">{result.summary}</p>
            </div>
            {#if result.details.length > 0}
              <ul class="m-0 flex flex-col gap-0.5 pl-5 text-xs text-foreground-muted">
                {#each result.details as detail, i (i)}
                  <li class="font-mono">{detail}</li>
                {/each}
              </ul>
            {/if}
          </div>
        {:else if failure}
          <div class="flex items-start gap-2" role="alert">
            <Badge variant="error" size="md">Error</Badge>
            <p class="m-0 text-sm text-foreground">{failure}</p>
          </div>
        {/if}

        {#if log.length > 0}
          <pre
            class="m-0 max-h-72 overflow-auto rounded-md border border-border bg-surface p-3 font-mono text-[11px] leading-relaxed whitespace-pre-wrap text-foreground"
            role="log"
            aria-live="polite"
            aria-label="Log for {test.title}"
            {@attach stickToBottom(log.length)}>{log.join("\n")}</pre>
        {/if}
      </section>
    </Card>
  {/each}
</div>

<ConfirmDialog
  open={confirming !== null}
  title="Run {confirming?.title ?? 'this test'}?"
  message="This test changes this PC: {confirming?.changes ?? ''} It ends by restoring the tweak from its snapshot."
  confirmText="Run test"
  variant="warning"
  onconfirm={confirmRun}
  oncancel={() => (confirming = null)}
/>
