<script lang="ts">
  import { Callout } from "$lib/components/ui";
  import { expand } from "$lib/utils/motion";

  interface Props {
    id: string;
    text?: string | null;
    open: boolean;
  }

  let { id, text, open }: Props = $props();
</script>

{#if text && open}
  <!-- The wrapper carries the transition: a component cannot. -->
  <div {id} transition:expand>
    <Callout tone="warning" density="compact" icon="mdi:alert" class="border-0 leading-relaxed">
      <span class="min-w-0">{text}</span>
    </Callout>
  </div>
{/if}
