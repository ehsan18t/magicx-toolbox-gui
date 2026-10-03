<script lang="ts">
  import type { IconName } from "$lib/design";
  import { IconTile, ModalHeader } from "$lib/components/ui";

  interface Props {
    title: string;
    icon: IconName;
    /** The current step's name. */
    step: string;
    /** Omitted while a step cannot be left. */
    onclose?: (() => void) | undefined;
  }

  let { title, icon, step, onclose }: Props = $props();
</script>

<ModalHeader {title} size="lg" {onclose}>
  {#snippet leading()}<IconTile {icon} />{/snippet}
  <p class="m-0 text-sm text-foreground-muted">{step}</p>
</ModalHeader>
