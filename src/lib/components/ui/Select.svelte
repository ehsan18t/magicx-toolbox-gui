<script lang="ts">
  import type { HTMLSelectAttributes } from "svelte/elements";

  interface Option {
    value: string | number;
    label: string;
    disabled?: boolean;
  }

  interface Props extends Omit<HTMLSelectAttributes, "class" | "onchange"> {
    value: string | number | null;
    options: Option[];
    placeholder?: string;
    pending?: boolean;
    class?: string;
    onchange?: (value: string | number) => void;
  }

  const {
    value,
    options,
    placeholder,
    pending = false,
    disabled = false,
    class: className = "",
    onchange,
    ...rest
  }: Props = $props();

  function handleChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    const newValue = target.value;
    // Try to preserve numeric type
    const numValue = Number(newValue);
    onchange?.(isNaN(numValue) ? newValue : numValue);
  }
</script>

<select
  class="bg-muted max-w-45 min-w-30 shrink-0 cursor-pointer appearance-none rounded-lg border border-border bg-[url('data:image/svg+xml,%3Csvg_xmlns=%27http://www.w3.org/2000/svg%27_width=%2712%27_height=%2712%27_viewBox=%270_0_24_24%27%3E%3Cpath_fill=%27%23888%27_d=%27M7_10l5_5_5-5z%27/%3E%3C/svg%3E')] bg-[right_8px_center] bg-no-repeat px-2.5 py-1.5 pr-7 text-xs font-medium text-foreground transition-all duration-200 hover:not-disabled:border-accent focus:border-accent focus:ring-2 focus:ring-accent/20 focus:outline-none disabled:cursor-not-allowed disabled:opacity-60
    {pending ? 'border-warning bg-warning/10' : ''}
    {className}"
  {disabled}
  value={value ?? ""}
  onchange={handleChange}
  {...rest}
>
  {#if placeholder && value === null}
    <option value="" disabled>{placeholder}</option>
  {/if}
  {#each options as opt (opt.value)}
    <option value={opt.value} disabled={opt.disabled}>{opt.label}</option>
  {/each}
</select>
