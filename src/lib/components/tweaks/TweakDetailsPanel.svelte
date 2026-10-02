<script lang="ts">
  import { discardSnapshotEntry, listSnapshotEntries } from "$lib/api/tweaks";
  import { Icon, MarkdownText } from "$lib/components/shared";
  import {
    FirewallChangeItem,
    HostsChangeItem,
    RegistryChangeItem,
    SchedulerChangeItem,
    ServiceChangeItem,
  } from "$lib/components/tweaks/details";
  import { Badge } from "$lib/components/ui";
  import { confirm } from "$lib/stores/confirm.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { closeTweakDetailsModal, tweakDetailsModalStore } from "$lib/stores/tweakDetailsModal.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import {
    elevationStore,
    keepCurrentState,
    loadingStore,
    pendingChangesStore,
    refreshTweakStatus,
    revertTweak,
    systemStore,
    tweaksStore,
  } from "$lib/stores/tweaks.svelte";
  import type { AttentionItem, EntrySummary, TweakEffectOption } from "$lib/types";
  import { attentionCause, permissionInfoFor, RISK_INFO } from "$lib/types";
  import { errorMessage, isAppExiting } from "$lib/utils/error";
  import { availabilityTitle, RISK_TONE, stateSummary, TONE_SOFT, TONE_TEXT } from "$lib/utils/tweakPresentation";

  interface Props {
    docked: boolean;
  }

  let { docked }: Props = $props();

  const tweak = $derived.by(() => {
    const id = tweakDetailsModalStore.tweakId;
    return id ? (tweaksStore.list.find((t) => t.definition.id === id) ?? null) : null;
  });

  const def = $derived(tweak?.definition ?? null);
  const status = $derived(tweak?.status ?? null);
  const pendingChange = $derived(def ? pendingChangesStore.get(def.id) : undefined);
  const riskInfo = $derived(def ? RISK_INFO[def.risk_level] : null);
  const isHighRisk = $derived(def?.risk_level === "high" || def?.risk_level === "critical");
  const isLoading = $derived(def ? loadingStore.isLoading(def.id) : false);
  const permissionInfo = $derived(def ? permissionInfoFor(def.required_level) : null);
  const summary = $derived(status ? stateSummary(status) : null);

  const currentWindowsVersion = $derived(systemStore.info ? (systemStore.info.windows.is_windows_11 ? 11 : 10) : null);

  let closeButton = $state<HTMLButtonElement | null>(null);
  let entries = $state<EntrySummary[]>([]);
  let entriesLoading = $state(false);
  let busySeq = $state<number | null>(null);
  let keeping = $state(false);

  let lastTab = navigationStore.activeTab;
  $effect(() => {
    const tab = navigationStore.activeTab;
    if (tab !== lastTab) {
      lastTab = tab;
      closeTweakDetailsModal();
    }
  });

  let panelEl = $state<HTMLElement | null>(null);
  let returnFocusTo: HTMLElement | null = null;

  let openedId: string | null = null;
  // Overlay is modal, so focus moves in. Focus returns to the opener in the effect body: a cleanup reads the pre-close store.
  $effect(() => {
    const id = def?.id ?? null;
    if (id && id !== openedId) {
      const active = document.activeElement;
      if (active instanceof HTMLElement && active !== document.body && !panelEl?.contains(active)) {
        returnFocusTo = active;
      }
    }
    openedId = id;
    if (id) {
      if (!docked) closeButton?.focus();
    } else if (returnFocusTo) {
      if (returnFocusTo.isConnected) returnFocusTo.focus();
      returnFocusTo = null;
    }
  });

  // Keyed on the whole tweak, so a restore or keep re-reads the list.
  $effect(() => {
    const id = tweak?.definition.id;
    let cancelled = false;

    if (id) {
      entriesLoading = true;
      listSnapshotEntries(id)
        .then((e) => {
          if (!cancelled) entries = e;
        })
        .catch(() => {
          if (!cancelled) entries = [];
        })
        .finally(() => {
          if (!cancelled) entriesLoading = false;
        });
    } else {
      entries = [];
    }

    return () => {
      cancelled = true;
    };
  });

  function handleKeydown(e: KeyboardEvent) {
    if (!def || e.defaultPrevented) return;
    const otherModal = [...document.querySelectorAll('[aria-modal="true"]')].some((el) => el !== panelEl);
    if (otherModal) return;
    if (e.key === "Escape") {
      closeTweakDetailsModal();
      return;
    }
    if (e.key !== "Tab" || docked || !panelEl) return;
    const focusables = [...panelEl.querySelectorAll<HTMLElement>("button:not([disabled]), a[href], [tabindex='0']")];
    const first = focusables[0];
    const last = focusables.at(-1);
    if (!first || !last) return;
    const active = document.activeElement;
    if (!panelEl.contains(active)) {
      e.preventDefault();
      first.focus();
    } else if (e.shiftKey && active === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && active === last) {
      e.preventDefault();
      first.focus();
    }
  }

  function attentionHint(item: AttentionItem): string {
    if (item.kind === "no_undo") return "(this one cannot be retried)";
    if (item.class === "busy") return "(retrying later may succeed)";
    if (item.class === "access_denied" && elevationStore.level === "User") return "(restart as administrator to retry)";
    return "";
  }

  function entryValidity(entry: EntrySummary): string {
    return entry.validity === "Valid" ? "Valid" : `Invalid · ${entry.validity.Invalid}`;
  }

  async function discardEntry(seq: number) {
    const t = tweak;
    if (!t) return;
    busySeq = seq;
    try {
      await discardSnapshotEntry(t.definition.id, seq);
    } catch (e) {
      const message = errorMessage(e);
      console.error("Failed to discard snapshot entry:", message);
      if (isAppExiting(e)) toastStore.warning(message);
      else toastStore.error(message);
      busySeq = null;
      return;
    }

    entries = entries.filter((entry) => entry.seq !== seq);
    try {
      entries = await listSnapshotEntries(t.definition.id);
    } catch (e) {
      toastStore.warning(`The entry was discarded, but the list could not be refreshed: ${errorMessage(e)}`);
    } finally {
      busySeq = null;
    }
    // The engine owns `has_backup` and its status arrives stamped, so an in-flight sweep cannot restore the badge.
    const read = await refreshTweakStatus(t.definition.id);
    if (!read.ok) {
      toastStore.warning(`The entry was discarded, but the tweak's state could not be re-read: ${read.message}`);
    }
  }

  async function requestDiscard(seq: number) {
    const ok = await confirm({
      title: `Discard snapshot entry #${seq}?`,
      message: "The state it recorded can no longer be restored for this tweak.",
      confirmText: "Discard",
      variant: "danger",
    });
    if (ok) await discardEntry(seq);
  }

  async function executeKeepCurrentState() {
    const t = tweak;
    if (!t) return;
    const ok = await confirm({
      title: "Keep the current state?",
      message:
        "This accepts the current state as-is and releases the saved snapshot, so the earlier state can no longer be restored for this tweak.",
      confirmText: "Keep current state",
      variant: "danger",
    });
    if (!ok) return;
    keeping = true;
    try {
      if (await keepCurrentState(t.definition.id, { showToast: true, tweakName: t.definition.name })) {
        entries = [];
        return;
      }
      // Nothing was released; re-read so the list does not claim the entries are gone.
      entries = await listSnapshotEntries(t.definition.id);
    } catch (e) {
      toastStore.warning(`The snapshot entries could not be re-read: ${errorMessage(e)}`);
    } finally {
      keeping = false;
    }
  }

  async function handleRestoreClick() {
    const t = tweak;
    if (!t || isLoading) return;
    if (
      isHighRisk &&
      !(await confirm({
        title: `Restore ${t.definition.name}?`,
        message: `This ${t.definition.risk_level}-risk tweak steps back to the state saved before its last change.`,
        confirmText: "Restore",
        variant: "warning",
      }))
    )
      return;
    // A second restore while one is in flight would walk back to the next-older snapshot.
    if (loadingStore.isLoading(t.definition.id)) return;
    await revertTweak(t.definition.id, { showToast: true, tweakName: t.definition.name });
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#snippet sectionTitle(icon: string, title: string)}
  <h3 class="m-0 mb-2.5 flex items-center gap-2 text-[13px] font-semibold text-foreground">
    <Icon {icon} width="16" class="text-foreground-muted" />
    {title}
  </h3>
{/snippet}

{#snippet fact(label: string, value: string, tone: string, hint?: string)}
  <div class="flex items-baseline justify-between gap-4 px-3 py-2">
    <dt class="shrink-0 text-xs text-foreground-muted">{label}</dt>
    <dd class="m-0 min-w-0 text-right">
      <span class="text-[13px] font-medium wrap-break-word {tone}">{value}</span>
      {#if hint}<span class="block text-xs text-foreground-subtle">{hint}</span>{/if}
    </dd>
  </div>
{/snippet}

{#snippet changeLabel(icon: string, title: string, count: number)}
  <h4 class="m-0 flex items-center gap-1.5 text-[11px] font-semibold tracking-wide text-foreground-muted uppercase">
    <Icon {icon} width="13" />
    {title}
    <span class="font-normal opacity-60">{count}</span>
  </h4>
{/snippet}

{#snippet changeList(o: TweakEffectOption)}
  {@const changeCount =
    o.registry_changes.length +
    o.service_changes.length +
    o.scheduler_changes.length +
    o.hosts_changes.length +
    o.firewall_changes.length +
    o.commands.length}
  {#if changeCount > 0}
    <div class="space-y-3 border-t border-border px-3 py-3">
      {#if o.registry_changes.length > 0}
        <div class="space-y-1.5">
          {@render changeLabel("mdi:database", "Registry", o.registry_changes.length)}
          {#each o.registry_changes as change, idx (idx)}
            <RegistryChangeItem {change} {currentWindowsVersion} />
          {/each}
        </div>
      {/if}
      {#if o.service_changes.length > 0}
        <div class="space-y-1.5">
          {@render changeLabel("mdi:server", "Services", o.service_changes.length)}
          {#each o.service_changes as change, idx (idx)}
            <ServiceChangeItem {change} />
          {/each}
        </div>
      {/if}
      {#if o.scheduler_changes.length > 0}
        <div class="space-y-1.5">
          {@render changeLabel("mdi:calendar", "Scheduled Tasks", o.scheduler_changes.length)}
          {#each o.scheduler_changes as change, idx (idx)}
            <SchedulerChangeItem {change} />
          {/each}
        </div>
      {/if}
      {#if o.hosts_changes.length > 0}
        <div class="space-y-1.5">
          {@render changeLabel("mdi:file-document-outline", "Hosts File", o.hosts_changes.length)}
          {#each o.hosts_changes as change, idx (idx)}
            <HostsChangeItem {change} />
          {/each}
        </div>
      {/if}
      {#if o.firewall_changes.length > 0}
        <div class="space-y-1.5">
          {@render changeLabel("mdi:shield-outline", "Firewall", o.firewall_changes.length)}
          {#each o.firewall_changes as change, idx (idx)}
            <FirewallChangeItem {change} />
          {/each}
        </div>
      {/if}
      {#if o.commands.length > 0}
        <div class="space-y-1.5">
          {@render changeLabel("mdi:console", "Commands", o.commands.length)}
          {#each o.commands as cmd, idx (idx)}
            <code
              class="block rounded-md border border-border bg-background px-3 py-2 font-mono text-[11px] break-all whitespace-pre-wrap text-foreground/85 select-text"
              >{cmd}</code
            >
          {/each}
        </div>
      {/if}
    </div>
  {:else}
    <div class="border-t border-border px-3 py-2.5 text-xs text-foreground-muted italic">
      No system changes. This is the stock Windows default.
    </div>
  {/if}
{/snippet}

{#if tweak && def && status && summary}
  {#if !docked}
    <button
      type="button"
      class="absolute inset-0 z-30 animate-fade-in cursor-default bg-black/30"
      aria-hidden="true"
      tabindex="-1"
      onclick={closeTweakDetailsModal}
    ></button>
  {/if}

  <aside
    class="flex flex-col {docked
      ? 'relative w-[clamp(360px,30%,440px)] shrink-0 border-l border-border bg-surface'
      : 'absolute inset-y-0 right-0 z-40 w-full max-w-115 animate-slide-in-right border-l border-border bg-elevated shadow-dialog'}"
    bind:this={panelEl}
    role={docked ? "complementary" : "dialog"}
    aria-modal={docked ? undefined : "true"}
    aria-labelledby="tweak-details-title"
  >
    <header class="flex shrink-0 items-start gap-3 border-b border-border px-5 pt-4 pb-3">
      <div class="min-w-0 flex-1">
        <h2 id="tweak-details-title" class="m-0 font-display text-lg leading-snug font-semibold wrap-break-word">
          {def.name}
        </h2>
        <p class="m-0 mt-1 text-[13px] leading-relaxed text-foreground-muted">{def.description}</p>
      </div>
      <button
        bind:this={closeButton}
        type="button"
        class="flex h-8 w-8 shrink-0 cursor-pointer items-center justify-center rounded-md text-foreground-muted hover:bg-muted hover:text-foreground"
        aria-label="Close details"
        onclick={closeTweakDetailsModal}
      >
        <Icon icon="mdi:close" width="18" />
      </button>
    </header>

    <div class="min-h-0 flex-1 space-y-5 overflow-y-auto px-5 py-4 select-text">
      <section>
        <dl class="m-0 divide-y divide-border rounded-lg border border-border bg-card">
          {@render fact(
            "Current state",
            summary.label,
            TONE_TEXT[summary.tone],
            status.state === "unavailable" ? (status.unavailableReason ?? undefined) : undefined,
          )}
          {#if pendingChange}
            {@render fact("Pending", `→ ${pendingChange.optionLabel}`, "text-warning")}
          {/if}
          {#if riskInfo}
            {@render fact("Risk", riskInfo.name, TONE_TEXT[RISK_TONE[def.risk_level]], riskInfo.description)}
          {/if}
          {@render fact(
            "Runs as",
            permissionInfo?.name ?? "Standard user",
            "text-foreground",
            permissionInfo?.description,
          )}
          {@render fact(
            "Restart",
            def.requires_reboot ? "Needed after apply or restore" : "Not needed",
            "text-foreground",
          )}
          {@render fact(
            "Reversible",
            def.reversible ? "Yes" : "No",
            def.reversible ? "text-foreground" : "text-warning",
          )}
          {@render fact(
            "Snapshot",
            status.has_backup ? "Saved, can restore" : "None",
            status.has_backup ? "text-accent" : "text-foreground-muted",
          )}
        </dl>

        {#if status.has_backup}
          <button
            type="button"
            class="mt-3 inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-secondary px-3 text-[13px] font-medium hover:bg-secondary-hover disabled:cursor-not-allowed disabled:opacity-50"
            onclick={handleRestoreClick}
            disabled={isLoading || def.availability.state !== "available"}
          >
            <Icon icon="mdi:history" width="16" />
            {status.attention?.reason === "restore_failed" ? "Retry restore" : "Restore previous state"}
          </button>
        {/if}
      </section>

      {#if def.warning}
        <div class="flex gap-3 rounded-lg border border-warning/30 bg-warning/8 p-3">
          <Icon icon="mdi:alert" width="18" class="mt-0.5 shrink-0 text-warning" />
          <p class="m-0 text-[13px] leading-relaxed text-foreground">{def.warning}</p>
        </div>
      {/if}

      {#if def.availability.state !== "available"}
        <div class="flex gap-3 rounded-lg border border-warning/30 bg-warning/8 p-3">
          <Icon icon="mdi:shield-lock-outline" width="18" class="mt-0.5 shrink-0 text-warning" />
          <p class="m-0 text-[13px] leading-relaxed">
            <span class="font-semibold">{availabilityTitle(def.availability)}.</span>
            <span class="text-foreground-muted">{def.availability.reason}</span>
          </p>
        </div>
      {/if}

      {#if status.state === "unknown" && status.unknownReasons.length > 0}
        <div class="rounded-lg border border-warning/30 bg-warning/8 p-3">
          <p class="m-0 mb-1.5 flex items-center gap-2 text-[13px] font-semibold">
            <Icon icon="mdi:help-circle-outline" width="16" class="text-warning" />
            Could not determine state
          </p>
          <ul class="m-0 list-none space-y-1 p-0 pl-6">
            {#each status.unknownReasons as reason, i (`${reason.effect}-${i}`)}
              <li class="text-xs text-foreground-muted">
                <span class="font-mono break-all text-foreground">{reason.effect}</span>
                ({reason.cause}{reason.needs_elevation ? ", restart as admin to resolve" : ""})
              </li>
            {/each}
          </ul>
        </div>
      {/if}

      {#if status.attention}
        <div class="rounded-lg border border-error/35 bg-error/8 p-3">
          <p class="m-0 flex gap-2 text-[13px] leading-relaxed">
            <Icon icon="mdi:alert-circle" width="18" class="mt-0.5 shrink-0 text-error" />
            <span>
              <span class="font-semibold">Needs attention.</span>
              <span class="text-foreground-muted">
                {attentionCause(status.attention.reason)}{status.has_backup
                  ? ", so the snapshot was kept."
                  : ". There is no snapshot left to restore."}
              </span>
            </span>
          </p>
          {#if status.attention.items.length}
            <ul class="m-0 mt-2 list-none space-y-1 p-0 pl-6.5">
              {#each status.attention.items as item, i (`${item.effect}-${i}`)}
                <li class="text-xs text-foreground-muted">
                  {#if item.effect}<span class="font-mono break-all text-foreground">{item.effect}</span>:{/if}
                  {item.message}
                  {attentionHint(item)}
                </li>
              {/each}
            </ul>
          {/if}
          <!-- Consent stays reachable whenever a record exists, entries left or not (ADR-0002). -->
          <button
            type="button"
            class="mt-3 ml-6.5 inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-secondary px-3 text-[13px] font-medium hover:bg-secondary-hover disabled:cursor-not-allowed disabled:opacity-50"
            onclick={executeKeepCurrentState}
            disabled={keeping || isLoading}
          >
            <Icon icon={keeping ? "mdi:loading" : "mdi:check"} width="16" class={keeping ? "animate-spin" : ""} />
            Keep current state
          </button>
        </div>
      {/if}

      {#if status.residues.length > 0 || status.heldShared.length > 0}
        <div class="space-y-2 rounded-lg border border-border bg-card p-3 text-[13px] text-foreground-muted">
          {#if status.residues.length > 0}
            <p class="m-0 flex gap-2">
              <Icon icon="mdi:information-outline" width="16" class="mt-0.5 shrink-0 text-info" />
              <span>Residual settings remain outside the active option: {status.residues.join(", ")}</span>
            </p>
          {/if}
          {#if status.heldShared.length > 0}
            <p class="m-0 flex gap-2">
              <Icon icon="mdi:link-variant" width="16" class="mt-0.5 shrink-0" />
              <span>
                Shared settings held: {status.heldShared.map((h) => `${h.shared} (${h.holders.join(", ")})`).join("; ")}
              </span>
            </p>
          {/if}
        </div>
      {/if}

      {#if def.info}
        <section>
          {@render sectionTitle("mdi:information-outline", "Details")}
          <MarkdownText content={def.info} />
        </section>
      {/if}

      <section>
        {@render sectionTitle("mdi:tune-variant", "Options")}
        <div class="space-y-2.5">
          {#if status.observed}
            {@const obs = status.observed}
            <div class="overflow-hidden rounded-lg border border-warning/40 bg-card">
              <div class="flex items-center justify-between gap-3 bg-warning/6 px-3 py-2.5">
                <div class="min-w-0">
                  <span class="block text-[13px] font-semibold wrap-break-word">{obs.changes.label}</span>
                  <span class="text-xs text-foreground-muted">Matches none of the options below</span>
                </div>
                <Badge variant="warning" size="sm">Current</Badge>
              </div>
              {@render changeList(obs.changes)}
              <ul class="m-0 list-none space-y-1 border-t border-border px-3 py-2.5">
                {#each obs.agreement as a (a.effect)}
                  <li class="text-xs">
                    <span class="font-mono break-all text-foreground">{a.name}</span>
                    {#if a.wanted_by.length > 0}
                      <span class="text-foreground-muted">agrees with {a.wanted_by.join(" and ")}</span>
                    {:else}
                      <span class="text-warning">agrees with no option</span>
                    {/if}
                  </li>
                {/each}
              </ul>
            </div>
          {/if}
          {#each def.options as option, i (option.label)}
            {@const isCurrent = status.activeOption === option.label}
            {@const isPending = pendingChange?.optionLabel === option.label}
            {@const unavailable = status.unavailableOptions.find((u) => u.label === option.label)}
            <div
              class="overflow-hidden rounded-lg border bg-card {isCurrent
                ? 'border-accent/50'
                : isPending
                  ? 'border-warning/50'
                  : 'border-border'}"
            >
              <div class="flex items-center justify-between gap-3 px-3 py-2.5">
                <div class="flex min-w-0 items-center gap-2.5">
                  <span
                    class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-xs font-bold {isCurrent
                      ? TONE_SOFT.accent
                      : TONE_SOFT.neutral}"
                  >
                    {i + 1}
                  </span>
                  <div class="min-w-0">
                    <span class="block text-[13px] font-semibold wrap-break-word">{option.label}</span>
                    {#if unavailable}<span class="block text-xs text-warning">{unavailable.reason}</span>{/if}
                  </div>
                </div>
                <div class="flex shrink-0 flex-wrap justify-end gap-1.5">
                  {#if isCurrent}<Badge variant="accent" size="sm">Current</Badge>{/if}
                  {#if isPending}<Badge variant="warning" size="sm">Pending</Badge>{/if}
                  {#if unavailable}<Badge variant="warning" size="sm">Unavailable</Badge>{/if}
                </div>
              </div>
              {@render changeList(option)}
            </div>
          {/each}
        </div>
      </section>

      <!-- Gated on entries, not `has_backup`: an all-invalid history still needs a discard path (ADR-0002). -->
      {#if status.has_backup || entriesLoading || entries.length > 0}
        <section>
          {@render sectionTitle("mdi:history", "Snapshot entries")}
          {#if entriesLoading}
            <div class="flex items-center gap-2 text-[13px] text-foreground-muted">
              <Icon icon="mdi:loading" width="16" class="animate-spin" />
              Loading…
            </div>
          {:else if entries.length === 0}
            <p class="m-0 text-[13px] text-foreground-muted italic">No snapshot entries.</p>
          {:else}
            <div class="space-y-1.5">
              {#each entries as entry (entry.seq)}
                <div class="flex items-center justify-between gap-3 rounded-md border border-border bg-card px-3 py-2">
                  <div class="min-w-0 text-xs wrap-break-word">
                    <span class="font-semibold">#{entry.seq}</span>
                    <span class="text-foreground-muted"> · {entryValidity(entry)}</span>
                    {#if entry.timestamp}<span class="text-foreground-muted"> · {entry.timestamp}</span>{/if}
                  </div>
                  <button
                    type="button"
                    class="inline-flex shrink-0 cursor-pointer items-center gap-1 rounded-md px-2 py-1 text-xs font-medium text-foreground-muted hover:bg-error/10 hover:text-error disabled:cursor-not-allowed disabled:opacity-50"
                    onclick={() => requestDiscard(entry.seq)}
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
        </section>
      {/if}
    </div>
  </aside>
{/if}
