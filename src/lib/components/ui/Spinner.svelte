<script lang="ts" module>
  import type { IconSize } from "$lib/design";

  const GLYPH = { sm: "md", md: "3xl", lg: "6xl" } as const satisfies Record<string, IconSize>;
</script>

<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { cn } from "$lib/utils/cn";

  interface Props {
    size?: keyof typeof GLYPH;
    /** Announces the spinner as a status; without it the spinner is decorative. */
    label?: string;
    class?: string;
  }

  let { size = "md", label, class: className }: Props = $props();
</script>

<span
  role={label ? "status" : undefined}
  aria-label={label}
  aria-hidden={label ? undefined : "true"}
  class={cn("inline-flex text-accent", className)}
>
  <Icon icon="mdi:loading" size={GLYPH[size]} class="animate-spin" />
</span>
