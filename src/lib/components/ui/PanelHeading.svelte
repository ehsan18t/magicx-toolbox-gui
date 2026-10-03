<script lang="ts">
  import { Icon, type IconName } from "$lib/components/shared";
  import { cn } from "$lib/utils/cn";
  import type { Snippet } from "svelte";
  import { ICON_SIZE } from "./icon";
  import { TONE_TEXT, type TextTone } from "./tone";

  interface Props {
    icon: IconName;
    tone?: TextTone;
    id?: string;
    class?: string;
    children: Snippet;
  }

  let { icon, tone = "neutral", id, class: className, children }: Props = $props();
</script>

<h3 {id} class={cn("m-0 flex items-center gap-2 text-ui font-semibold", className)}>
  <Icon {icon} width={ICON_SIZE.md} class={TONE_TEXT[tone]} />
  {@render children()}
</h3>
