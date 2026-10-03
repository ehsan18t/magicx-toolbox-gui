<script lang="ts" module>
  import type { Tone } from "$lib/design";
  import type { ManualTestStatus } from "$lib/types";

  const MIN_MINUTES = 1;
  const MAX_MINUTES = 1440;

  const STATUS_BADGE: Record<ManualTestStatus, { tone: Tone; label: string }> = {
    pass: { tone: "success", label: "Pass" },
    fail: { tone: "error", label: "Fail" },
    info: { tone: "info", label: "Info" },
  };

  const clampMinutes = (value: number) =>
    Math.min(MAX_MINUTES, Math.max(MIN_MINUTES, Math.floor(value || MIN_MINUTES)));
</script>

<script lang="ts">
  import { PageLayout } from "$lib/components/layout";
  import { Icon } from "$lib/components/shared";
  import { Badge, Button, Card, Spinner } from "$lib/components/ui";
  import { confirmStore } from "$lib/stores/confirm.svelte";
  import { manualTestsStore } from "$lib/stores/manualTests.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import type { ManualTest } from "$lib/types";
  import { copyText } from "$lib/utils/clipboard";
  import type { Attachment } from "svelte/attachments";

  let minutes = $state<Record<string, number>>({});

  const runningId = $derived(manualTestsStore.runningId);

  function minutesFor(test: ManualTest): number | null {
    return test.minutes === null ? null : (minutes[test.id] ?? test.minutes);
  }

  // Written back to the input: an out-of-range entry that clamps to the stored value would otherwise stay shown.
  function commitMinutes(test: ManualTest, input: HTMLInputElement) {
    const value = clampMinutes(input.valueAsNumber);
    minutes[test.id] = value;
    input.valueAsNumber = value;
  }

  async function run(test: ManualTest) {
    if (
      test.changes_system &&
      !(await confirmStore.ask({
        title: `Run ${test.title}?`,
        message: `This test changes this PC: ${test.changes} It ends by restoring the tweak from its snapshot.`,
        confirmText: "Run test",
        variant: "warning",
      }))
    )
      return;
    void manualTestsStore.run(test.id, minutesFor(test));
  }

  async function copyReport(id: string) {
    if (await copyText(manualTestsStore.report(id), "Could not copy the report")) toastStore.success("Report copied");
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
    <Card as="section" class="flex flex-col gap-3 p-4" aria-labelledby="manual-test-{test.id}">
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
              value={minutesFor(test)}
              onchange={(e) => commitMinutes(test, e.currentTarget)}
            />
          </label>
        {/if}
        <Button
          variant={test.changes_system ? "warning" : "primary"}
          size="sm"
          disabled={runningId !== null}
          aria-label="Run {test.title}"
          onclick={() => run(test)}
        >
          <Icon icon="mdi:play" size="md" />
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
            <Icon icon="mdi:stop" size="md" />
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
            <Icon icon="mdi:content-copy" size="md" />
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
        {@const badge = STATUS_BADGE[result.status]}
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
    </Card>
  {/each}
</PageLayout>
