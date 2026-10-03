<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { type IconName, TONE_SOFT } from "$lib/design";
  import { ModalHeader } from "$lib/components/ui";
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    icon: IconName;
    /** The current step's name. */
    subtitle: string;
    onclose?: () => void;
    actions?: Snippet;
  }

  let { title, icon, subtitle, onclose, actions }: Props = $props();
</script>

<ModalHeader {title} size="lg" {onclose} {actions}>
  {#snippet leading()}
    <div class="flex h-10 w-10 items-center justify-center rounded-lg {TONE_SOFT.accent}">
      <Icon {icon} size="3xl" />
    </div>
  {/snippet}
  <p class="m-0 text-sm text-foreground-muted">{subtitle}</p>
</ModalHeader>
