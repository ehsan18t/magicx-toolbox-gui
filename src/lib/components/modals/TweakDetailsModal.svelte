<script lang="ts">
  import { discardSnapshotEntry, listSnapshotEntries } from "$lib/api/tweaks";
  import { Icon } from "$lib/components/shared";
  import { Badge, IconButton, Modal, ModalBody, ModalHeader } from "$lib/components/ui";
  import { closeTweakDetailsModal, tweakDetailsModalStore } from "$lib/stores/tweakDetailsModal.svelte";
  import { pendingChangesStore, revertTweak, tweaksStore } from "$lib/stores/tweaks.svelte";
  import type { EntrySummary } from "$lib/types";
  import { PERMISSION_INFO, permissionFromElevation, RISK_INFO } from "$lib/types";

  const isOpen = $derived(tweakDetailsModalStore.isOpen);

  const tweak = $derived.by(() => {
    const state = tweakDetailsModalStore.state;
    if (!state) return null;
    return tweaksStore.list.find((t) => t.definition.id === state.tweakId) ?? null;
  });

  const def = $derived(tweak?.definition ?? null);
  const status = $derived(tweak?.status ?? null);

  const pendingChange = $derived.by(() => {
    const t = tweak;
    if (!t) return undefined;
    return pendingChangesStore.get(t.definition.id);
  });

  const riskInfo = $derived(def ? RISK_INFO[def.risk_level] : null);
  const permission = $derived(def ? permissionFromElevation(def.elevation) : "none");
  const permissionInfo = $derived(permission !== "none" ? PERMISSION_INFO[permission] : null);

  const stateLabel = $derived.by(() => {
    if (!status) return "";
    switch (status.state) {
      case "active":
        return `Active · ${status.activeOption}`;
      case "system_default":
        return "System Default";
      case "unavailable":
        return "Unavailable";
      case "unknown":
        return "Unknown";
      default:
        return "Checking…";
    }
  });

  // Snapshot entries (the discard affordance) — loaded when the modal opens for a tweak
  // that has restorable history.
  let entries = $state<EntrySummary[]>([]);
  let entriesLoading = $state(false);
  let busySeq = $state<number | null>(null);

  $effect(() => {
    const t = tweak;
    const open = isOpen;
    let cancelled = false;

    if (open && t?.status.has_backup) {
      entriesLoading = true;
      listSnapshotEntries(t.definition.id)
        .then((e) => {
          if (!cancelled) {
            entries = e;
            entriesLoading = false;
          }
        })
        .catch(() => {
          if (!cancelled) {
            entries = [];
            entriesLoading = false;
          }
        });
    } else {
      entries = [];
    }

    return () => {
      cancelled = true;
    };
  });

  function entryValidity(entry: EntrySummary): string {
    return entry.validity === "Valid" ? "Valid" : `Invalid · ${entry.validity.Invalid}`;
  }

  async function discardEntry(seq: number) {
    const t = tweak;
    if (!t) return;
    busySeq = seq;
    try {
      await discardSnapshotEntry(t.definition.id, seq);
      const remaining = await listSnapshotEntries(t.definition.id);
      entries = remaining;
      if (remaining.length === 0) {
        tweaksStore.patchStatus(t.definition.id, { has_backup: false });
      }
    } catch (e) {
      console.error("Failed to discard snapshot entry:", e);
    } finally {
      busySeq = null;
    }
  }

  async function handleRestore() {
    const t = tweak;
    if (!t) return;
    await revertTweak(t.definition.id, { showToast: true, tweakName: t.definition.name });
  }
</script>

<Modal open={isOpen && !!tweak} onclose={closeTweakDetailsModal} size="lg" labelledBy="tweak-details-title">
  {#if tweak && def && status}
    <ModalHeader id="tweak-details-title">
      <div class="min-w-0">
        <h2 class="m-0 truncate text-lg font-bold text-foreground">{def.name}</h2>
        <p class="m-0 mt-1 text-sm text-foreground-muted">{def.description}</p>
      </div>
      <IconButton icon="mdi:close" onclick={closeTweakDetailsModal} aria-label="Close" />
    </ModalHeader>

    <ModalBody scrollable class="max-h-[calc(100dvh-2.5rem-6rem)]">
      <!-- Status Overview -->
      <div class="rounded-xl border border-border bg-surface/50 p-4">
        <div class="flex flex-wrap items-center gap-2">
          <!-- Detected state -->
          <div
            class="flex items-center gap-2 rounded-lg px-3 py-1.5 {status.is_applied
              ? 'bg-success/10 text-success'
              : status.state === 'unknown' || status.state === 'unavailable'
                ? 'bg-warning/10 text-warning'
                : 'bg-muted text-foreground-muted'}"
          >
            <Icon
              icon={status.is_applied
                ? "mdi:check-circle"
                : status.state === "unknown"
                  ? "mdi:help-circle-outline"
                  : status.state === "unavailable"
                    ? "mdi:cancel"
                    : "mdi:circle-outline"}
              width="16"
            />
            <span class="text-sm font-medium">{stateLabel}</span>
          </div>

          {#if riskInfo}
            <div class="bg-muted flex items-center gap-1.5 rounded-lg px-3 py-1.5">
              <Icon icon="mdi:shield-alert-outline" width="14" class="text-foreground-muted" />
              <span class="text-sm text-foreground-muted">{riskInfo.name} Risk</span>
            </div>
          {/if}

          {#if permissionInfo}
            <div class="bg-muted flex items-center gap-1.5 rounded-lg px-3 py-1.5">
              <Icon icon={permissionInfo.icon} width="14" class="text-foreground-muted" />
              <span class="text-sm text-foreground-muted">{permissionInfo.name}</span>
            </div>
          {/if}

          {#if def.requires_reboot}
            <div class="flex items-center gap-1.5 rounded-lg bg-info/10 px-3 py-1.5 text-info">
              <Icon icon="mdi:restart" width="14" />
              <span class="text-sm">Reboot Required</span>
            </div>
          {/if}

          {#if status.has_backup}
            <div class="flex items-center gap-1.5 rounded-lg bg-accent/10 px-3 py-1.5 text-accent">
              <Icon icon="mdi:history" width="14" />
              <span class="text-sm">Snapshot Available</span>
            </div>
          {/if}

          {#if pendingChange}
            <div class="flex items-center gap-1.5 rounded-lg bg-warning/10 px-3 py-1.5 text-warning">
              <Icon icon="mdi:clock-outline" width="14" />
              <span class="text-sm">Pending · {pendingChange.optionLabel}</span>
            </div>
          {/if}
        </div>

        <!-- Restore action -->
        {#if status.has_backup}
          <div class="mt-3 border-t border-border/50 pt-3">
            <button
              type="button"
              class="inline-flex cursor-pointer items-center gap-1.5 rounded-lg border border-accent/40 bg-accent/5 px-3 py-1.5 text-sm font-medium text-accent transition-colors hover:bg-accent/10"
              onclick={handleRestore}
              aria-label="Restore to original state"
            >
              <Icon icon="mdi:history" width="16" />
              {status.needs_attention ? "Retry restore" : "Restore to original state"}
            </button>
          </div>
        {/if}
      </div>

      <!-- Availability notice -->
      {#if def.availability.state !== "available"}
        <div class="mt-4 flex items-start gap-3 rounded-xl border border-warning/30 bg-warning/5 p-4">
          <Icon icon="mdi:shield-lock-outline" width="18" class="mt-0.5 shrink-0 text-warning" />
          <div class="text-sm">
            <span class="font-medium text-foreground">
              {def.availability.state === "sid_mismatch" ? "Over-the-shoulder guard" : "Elevation required"}
            </span>
            <span class="text-foreground-muted"> — {def.availability.reason}</span>
          </div>
        </div>
      {/if}

      <!-- Unknown detail -->
      {#if status.state === "unknown" && status.unknownReasons.length > 0}
        <div class="mt-4 rounded-xl border border-border p-4">
          <h3 class="m-0 mb-2 flex items-center gap-2 text-sm font-semibold text-foreground">
            <Icon icon="mdi:help-circle-outline" width="16" class="text-warning" />
            Could not determine state
          </h3>
          <ul class="m-0 list-none space-y-1 p-0">
            {#each status.unknownReasons as reason, i (`${reason.effect}-${i}`)}
              <li class="flex items-center gap-2 text-xs text-foreground-muted">
                <Icon icon="mdi:circle-small" width="14" />
                <span class="font-mono text-foreground">{reason.effect}</span>
                <span>— {reason.cause}{reason.needs_elevation ? " (restart as admin to resolve)" : ""}</span>
              </li>
            {/each}
          </ul>
        </div>
      {/if}

      <!-- Needs Attention detail -->
      {#if status.needs_attention}
        <div class="mt-4 flex items-start gap-3 rounded-xl border border-error/30 bg-error/5 p-4">
          <Icon icon="mdi:alert-circle" width="18" class="mt-0.5 shrink-0 text-error" />
          <div class="text-sm">
            <span class="font-medium text-foreground">Needs attention</span>
            <span class="text-foreground-muted">
              — the last restore didn't fully complete; the snapshot is kept.
              {#if status.unrestorable_resources.length}
                Unrecoverable: {status.unrestorable_resources.join("; ")}.
              {/if}
            </span>
          </div>
        </div>
      {/if}

      <!-- Residues / shared disclosures -->
      {#if status.residues.length > 0 || status.heldShared.length > 0}
        <div class="mt-4 rounded-xl border border-border p-4 text-sm">
          {#if status.residues.length > 0}
            <div class="flex items-start gap-2 text-foreground-muted">
              <Icon icon="mdi:information-outline" width="16" class="mt-0.5 shrink-0 text-info" />
              <span>Residual settings remain outside the active option: {status.residues.join(", ")}</span>
            </div>
          {/if}
          {#if status.heldShared.length > 0}
            <div class="mt-2 flex items-start gap-2 text-foreground-muted">
              <Icon icon="mdi:link-variant" width="16" class="mt-0.5 shrink-0 text-foreground-muted" />
              <span>
                Shared settings held: {status.heldShared.map((h) => `${h.shared} (${h.holders.join(", ")})`).join("; ")}
              </span>
            </div>
          {/if}
        </div>
      {/if}

      <!-- Options -->
      <div class="mt-6">
        <h3 class="mb-3 flex items-center gap-2 text-sm font-semibold text-foreground">
          <Icon icon="mdi:tune-variant" width="16" class="text-foreground-muted" />
          Options
        </h3>
        <div class="space-y-2">
          {#each def.optionLabels as label, i (label)}
            {@const isCurrent = status.activeOption === label}
            {@const isPending = pendingChange?.optionLabel === label}
            {@const unavailable = status.unavailableOptions.find((u) => u.label === label)}
            <div
              class="flex items-center justify-between gap-3 rounded-xl border px-4 py-3 {isCurrent
                ? 'border-accent/40 bg-accent/3'
                : isPending
                  ? 'border-warning/40 bg-warning/3'
                  : 'border-border bg-background'}"
            >
              <div class="flex min-w-0 items-center gap-3">
                <div
                  class="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg text-xs font-bold {isCurrent
                    ? 'bg-accent/15 text-accent'
                    : 'bg-muted text-foreground-muted'}"
                >
                  {i + 1}
                </div>
                <div class="min-w-0">
                  <span class="block truncate text-sm font-semibold text-foreground">{label}</span>
                  {#if unavailable}
                    <span class="text-xs text-warning">{unavailable.reason}</span>
                  {/if}
                </div>
              </div>
              <div class="flex shrink-0 items-center gap-2">
                {#if isCurrent}<Badge variant="accent" size="sm">Current</Badge>{/if}
                {#if isPending}<Badge variant="warning" size="sm">Pending</Badge>{/if}
                {#if unavailable}<Badge variant="warning" size="sm">Unavailable</Badge>{/if}
              </div>
            </div>
          {/each}
        </div>
      </div>

      <!-- Snapshot entries (discard affordance) -->
      {#if status.has_backup}
        <div class="mt-6">
          <h3 class="mb-3 flex items-center gap-2 text-sm font-semibold text-foreground">
            <Icon icon="mdi:history" width="16" class="text-foreground-muted" />
            Snapshot Entries
          </h3>
          {#if entriesLoading}
            <div class="flex items-center gap-2 text-sm text-foreground-muted">
              <Icon icon="mdi:loading" width="16" class="animate-spin" />
              Loading…
            </div>
          {:else if entries.length === 0}
            <p class="m-0 text-sm text-foreground-muted italic">No snapshot entries.</p>
          {:else}
            <div class="space-y-2">
              {#each entries as entry (entry.seq)}
                <div
                  class="flex items-center justify-between gap-3 rounded-lg border border-border bg-background px-3 py-2"
                >
                  <div class="min-w-0 text-xs">
                    <span class="font-medium text-foreground">#{entry.seq}</span>
                    <span class="text-foreground-muted"> · {entryValidity(entry)}</span>
                    {#if entry.timestamp}
                      <span class="text-foreground-muted"> · {entry.timestamp}</span>
                    {/if}
                  </div>
                  <button
                    type="button"
                    class="inline-flex shrink-0 cursor-pointer items-center gap-1 rounded-md border border-border bg-transparent px-2 py-1 text-[11px] font-medium text-foreground-muted transition-colors hover:border-error/40 hover:bg-error/5 hover:text-error disabled:cursor-not-allowed disabled:opacity-50"
                    onclick={() => discardEntry(entry.seq)}
                    disabled={busySeq === entry.seq}
                    aria-label="Discard snapshot entry {entry.seq}"
                  >
                    <Icon
                      icon={busySeq === entry.seq ? "mdi:loading" : "mdi:delete-outline"}
                      width="14"
                      class={busySeq === entry.seq ? "animate-spin" : ""}
                    />
                    Discard
                  </button>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </ModalBody>
  {/if}
</Modal>
