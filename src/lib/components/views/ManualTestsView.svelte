<script lang="ts">
  import type { ManualTest, ManualTestStatus } from "$lib/api/manualTests";
  import { PageLayout } from "$lib/components/layout";
  import { ConfirmDialog } from "$lib/components/modals";
  import { Icon } from "$lib/components/shared";
  import { Badge, Button, Card, Spinner } from "$lib/components/ui";
  import { manualTestsStore } from "$lib/stores/manualTests.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import type { Attachment } from "svelte/attachments";

  const MIN_MINUTES = 1;
  const MAX_MINUTES = 1440;

  let minutes = $state<Record<string, number>>({});
  let confirming = $state<ManualTest | null>(null);

  const runningId = $derived(manualTestsStore.runningId);

  const statusBadge: Record<ManualTestStatus, { tone: "success" | "error" | "info"; label: string }> = {
    pass: { tone: "success", label: "Pass" },
    fail: { tone: "error", label: "Fail" },
    info: { tone: "info", label: "Info" },
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
      await navigator.clipboard.writeText(manualTestsStore.report(id));
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

<PageLayout
  title="Manual Tests"
  description="Test build only. Run the app as administrator, run one test at a time, then copy its report."
>
  {#each manualTestsStore.tests as test (test.id)}
    {@const isRunning = runningId === test.id}
    {@const result = manualTestsStore.result(test.id)}
    {@const failure = manualTestsStore.failure(test.id)}
    {@const log = manualTestsStore.log(test.id)}
    <Card class="p-4">
      <section class="flex flex-col gap-3" aria-labelledby="manual-test-{test.id}">
        <div class="flex flex-wrap items-center gap-2">
          <h2 id="manual-test-{test.id}" class="m-0 text-base font-semibold text-foreground">{test.title}</h2>
          <Badge tone={test.changes_system ? "warning" : "neutral"}>
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
                min={MIN_MINUTES}
                max={MAX_MINUTES}
                class="h-8 w-20 rounded-md border border-border bg-secondary px-2 text-sm text-foreground focus:border-accent focus:outline-none"
                disabled={runningId !== null}
                bind:value={
                  () => minutesFor(test) ?? MIN_MINUTES,
                  (v) => (minutes[test.id] = Math.min(MAX_MINUTES, Math.max(MIN_MINUTES, Math.floor(v || MIN_MINUTES))))
                }
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
              loading={manualTestsStore.isCancelling}
              aria-label="Cancel {test.title}"
              onclick={() => manualTestsStore.cancel()}
            >
              <Icon icon="mdi:stop" width="16" />
              {manualTestsStore.isCancelling ? "Stopping and restoring" : "Cancel"}
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
              <Spinner size="sm" label="Running {test.title}" />
              Running
            </span>
          {/if}
        </div>

        {#if result}
          {@const badge = statusBadge[result.status]}
          <div class="flex animate-fade-in flex-col gap-1.5" role="status">
            <div class="flex items-start gap-2">
              <Badge tone={badge.tone} size="md">{badge.label}</Badge>
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
          <div class="flex animate-fade-in items-start gap-2" role="alert">
            <Badge tone="error" size="md">Error</Badge>
            <p class="m-0 text-sm text-foreground">{failure}</p>
          </div>
        {/if}

        {#if log.length > 0}
          <pre
            class="m-0 max-h-72 overflow-auto rounded-md border border-border bg-surface p-3 font-mono text-caption leading-relaxed whitespace-pre-wrap text-foreground"
            role="log"
            aria-live="polite"
            aria-label="Log for {test.title}"
            {@attach stickToBottom(log.length)}>{log.join("\n")}</pre>
        {/if}
      </section>
    </Card>
  {/each}
</PageLayout>

<ConfirmDialog
  open={confirming !== null}
  title="Run {confirming?.title ?? 'this test'}?"
  message="This test changes this PC: {confirming?.changes ?? ''} It ends by restoring the tweak from its snapshot."
  confirmText="Run test"
  variant="warning"
  onconfirm={confirmRun}
  oncancel={() => (confirming = null)}
/>
