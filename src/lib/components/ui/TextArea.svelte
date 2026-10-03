<script lang="ts">
  import type { HTMLTextareaAttributes } from "svelte/elements";
  import { field } from "./variants";

  interface Props extends Omit<HTMLTextareaAttributes, "class" | "value"> {
    value?: string | null;
    class?: string;
  }

  let { value = $bindable(), rows = 3, class: className, ...rest }: Props = $props();
</script>

<textarea bind:value {rows} class={field({ multiline: true, class: className })} {...rest}></textarea>
