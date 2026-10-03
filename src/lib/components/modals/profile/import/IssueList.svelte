<script lang="ts" module>
  import type { IconName } from "$lib/design";

  export interface Issue {
    key: string;
    message: string;
    tweakName?: string;
    rolledBack?: boolean;
  }

  type IssueTone = "warning" | "error";

  const ICON: Record<IssueTone, IconName> = { warning: "mdi:alert", error: "mdi:close-circle" };
  const MAX_LISTED = 5;
</script>

<script lang="ts">
  import { TONE_TEXT } from "$lib/design";
  import { Callout } from "$lib/components/ui";

  interface Props {
    tone: IssueTone;
    title: string;
    issues: Issue[];
    class?: string;
  }

  let { tone, title, issues, class: className }: Props = $props();
</script>

<Callout {tone} density="panel" icon={ICON[tone]} class={className}>
  <div class="min-w-0 flex-1 text-sm">
    <p class="m-0 font-medium {TONE_TEXT[tone]}">{title}</p>
    <ul class="m-0 mt-2 list-inside list-disc space-y-1 pl-1 text-foreground-muted">
      {#each issues.slice(0, MAX_LISTED) as issue (issue.key)}
        <li>
          {#if issue.tweakName}<span class="font-medium">{issue.tweakName}</span>:{/if}
          {issue.message}
          {#if issue.rolledBack}<span class="text-xs text-warning">(rolled back)</span>{/if}
        </li>
      {/each}
      {#if issues.length > MAX_LISTED}
        <li class={TONE_TEXT[tone]}>…and {issues.length - MAX_LISTED} more</li>
      {/if}
    </ul>
  </div>
</Callout>
