<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Badge } from "$lib/components/ui";
  import type { RegistryChange } from "$lib/types";

  interface Props {
    change: RegistryChange;
    currentWindowsVersion: number | null;
  }

  const { change, currentWindowsVersion }: Props = $props();

  function formatRegistryPath(c: RegistryChange): string {
    return `${c.hive}\\${c.key}`;
  }

  function formatRegistryValue(value: unknown): string {
    if (value === null || value === undefined) return "(delete)";
    if (typeof value === "number") return `0x${value.toString(16).toUpperCase()} (${value})`;
    if (typeof value === "string") return value === "" ? '""' : `"${value}"`;
    return JSON.stringify(value);
  }

  function windowsApplies(windowsVersions: number[] | undefined): boolean {
    if (!windowsVersions || windowsVersions.length === 0) return true;
    if (!currentWindowsVersion) return true;
    return windowsVersions.includes(currentWindowsVersion);
  }

  function isKeyAction(action: RegistryChange["action"]): boolean {
    return action === "delete_key" || action === "create_key";
  }

  function actionLabel(action: RegistryChange["action"]): string {
    switch (action) {
      case "set":
        return "set";
      case "delete_value":
        return "delete value";
      case "delete_key":
        return "delete key";
      case "create_key":
        return "create key";
    }
  }
</script>

<div class="overflow-hidden rounded-lg border border-border/60 bg-background">
  <div class="bg-muted/30 flex flex-wrap items-center justify-between gap-2 border-b border-border/40 px-3 py-2">
    <div class="flex min-w-0 items-center gap-2">
      <Icon icon="mdi:key-variant" width="12" class="text-foreground-muted" />
      <code class="bg-transparent p-0 font-mono text-[10px] break-all text-primary">
        {formatRegistryPath(change)}
      </code>
    </div>
    <div class="flex items-center gap-2">
      <Badge size="sm" variant="default">action: {actionLabel(change.action)}</Badge>
      {#if change.windows_versions && change.windows_versions.length > 0}
        <Badge size="sm" variant="default">Win {change.windows_versions.join(",")}</Badge>
      {/if}
      {#if !windowsApplies(change.windows_versions)}
        <Badge size="sm" variant="warning">not active</Badge>
      {/if}
      {#if change.skip_validation}
        <Badge size="sm" variant="default">skip_validation</Badge>
      {/if}
    </div>
  </div>
  <div class="px-3 py-2">
    {#if isKeyAction(change.action)}
      <div class="flex items-center gap-2 text-xs">
        <Badge size="sm" variant="info">Key</Badge>
        <code class="bg-transparent p-0 font-mono text-[10px] text-foreground/80">
          {change.action === "delete_key" ? "(delete key)" : "(create key)"}
        </code>
      </div>
    {:else}
      <div class="mb-1.5 flex flex-wrap items-center gap-2">
        <span class="font-mono text-xs font-semibold text-foreground">
          {change.value_name || "(Default)"}
        </span>
        {#if change.action === "set"}
          <Badge size="sm" variant="default">{change.value_type ?? "unknown"}</Badge>
        {/if}
      </div>
      <div class="flex items-center gap-2 text-xs">
        <Badge size="sm" variant="info">{change.action === "delete_value" ? "Delete" : "Value"}</Badge>
        <code class="bg-transparent p-0 font-mono text-[10px] text-foreground/80">
          {formatRegistryValue(change.value)}
        </code>
      </div>
    {/if}
  </div>
</div>
