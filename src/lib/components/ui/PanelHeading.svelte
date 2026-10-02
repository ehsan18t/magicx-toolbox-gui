<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { cn } from "$lib/utils/cn";
  import { TONE_TEXT, type Tone } from "$lib/utils/tweakPresentation";
  import type { Snippet } from "svelte";

  interface Props {
    icon: string;
    tone?: Tone;
    id?: string;
    class?: string;
    children: Snippet;
  }

  let { icon, tone = "neutral", id, class: className, children }: Props = $props();
</script>

<h3 {id} class={cn("m-0 mb-2.5 flex items-center gap-2 text-ui font-semibold", className)}>
  <Icon {icon} width="16" class={TONE_TEXT[tone]} />
  {@render children()}
</h3>
