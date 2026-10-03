<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import { field } from "./variants";

  interface Props extends Omit<HTMLInputAttributes, "class" | "value"> {
    value?: string | number | null;
    class?: string;
  }

  let { value = $bindable(), type = "text", class: className, ...rest }: Props = $props();
</script>

<input {type} bind:value class={field({ class: className })} {...rest} />
