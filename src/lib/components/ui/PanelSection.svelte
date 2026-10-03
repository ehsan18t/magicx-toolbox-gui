<script lang="ts">
  import type { IconName } from "$lib/design";
  import type { Snippet } from "svelte";
  import PanelHeading from "./PanelHeading.svelte";

  interface Props {
    title: string;
    icon: IconName;
    class?: string | undefined;
    children: Snippet;
  }

  let { title, icon, class: className, children }: Props = $props();

  const headingId = $props.id();
</script>

<section aria-labelledby={headingId} class={className}>
  <PanelHeading id={headingId} {icon} class="mb-2.5">{title}</PanelHeading>
  {@render children()}
</section>
