<script lang="ts" module>
  import type { IconName, Tone } from "$lib/design";

  const OUTCOME = {
    applied: { icon: "mdi:check-circle", tone: "success", title: "Profile applied" },
    partial: { icon: "mdi:alert-circle", tone: "warning", title: "Partially applied" },
  } satisfies Record<string, { icon: IconName; tone: Tone; title: string }>;
</script>

<script lang="ts">
  import { Button, Callout, IconTile, ModalBody, ModalFooter } from "$lib/components/ui";
  import type { ProfileApplyResult } from "$lib/types";
  import { plural } from "$lib/utils/format";
  import IssueList, { type Issue } from "./IssueList.svelte";

  let { result, onfinish }: { result: ProfileApplyResult; onfinish: () => void } = $props();

  const outcome = $derived(result.success ? OUTCOME.applied : OUTCOME.partial);
  const failures = $derived<Issue[]>(
    result.failures.map((f) => ({
      key: f.tweak_id,
      message: f.error,
      tweakName: f.tweak_name,
      rolledBack: f.was_rolled_back,
    })),
  );
</script>

<ModalBody class="flex animate-fade-in flex-col items-center gap-6 py-8 text-center">
  <IconTile icon={outcome.icon} size="3xl" shape="circle" tone={outcome.tone} iconSize="7xl" />

  <div>
    <h3 class="m-0 text-xl font-bold">{outcome.title}</h3>
    <p class="m-0 mt-2 text-foreground-muted">
      Applied {plural(result.applied_count, "tweak")}{#if result.skipped_count > 0}, skipped {result.skipped_count}{/if}.
    </p>
  </div>

  {#if failures.length > 0}
    <IssueList tone="error" title="{failures.length} failed" issues={failures} class="w-full max-w-md text-left" />
  {/if}

  {#if result.requires_reboot}
    <Callout tone="warning" icon="mdi:restart" class="text-sm text-warning">Some changes need a restart.</Callout>
  {/if}
</ModalBody>

<ModalFooter>
  <Button variant="primary" icon="mdi:check" onclick={onfinish}>Done</Button>
</ModalFooter>
