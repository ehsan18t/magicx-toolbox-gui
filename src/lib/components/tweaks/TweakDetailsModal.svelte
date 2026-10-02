<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { discardSnapshotEntry, listSnapshotEntries } from "$lib/api/tweaks";
  import { Icon, MarkdownText } from "$lib/components/shared";
  import { Modal, ModalBody } from "$lib/components/ui";
  import { confirm } from "$lib/stores/confirm.svelte";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { closeTweakDetailsModal, tweakDetailsModalStore } from "$lib/stores/tweakDetailsModal.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import {
    elevationStore,
    loadingStore,
    pendingChangesStore,
    refreshTweakStatus,
    tweaksStore,
  } from "$lib/stores/tweaks.svelte";
  import type { AttentionItem, EntrySummary } from "$lib/types";
  import { attentionCause, permissionInfoFor, RISK_INFO } from "$lib/types";
  import { buildMatrix, optionsMatchingNow, type ChangeKind, type MatrixCell } from "$lib/utils/changeMatrix";
  import { errorMessage, isAppExiting } from "$lib/utils/error";
  import { expand } from "$lib/utils/motion";
  import { keepWithConfirm, restoreTweak } from "$lib/utils/tweakActions";
  import { availabilityTitle, isHighRisk, RISK_TONE, stateSummary, TONE_TEXT } from "$lib/utils/tweakPresentation";
  import TweakControl from "./TweakControl.svelte";

  // Held through the exit animation, after the store has cleared the id.
  let lastId: string | null = null;
  const shownId = $derived.by(() => (lastId = tweakDetailsModalStore.tweakId ?? lastId));
  const tweak = $derived(shownId ? (tweaksStore.getById(shownId) ?? null) : null);

  const def = $derived(tweak?.definition ?? null);
  const status = $derived(tweak?.status ?? null);
  const pendingChange = $derived(def ? pendingChangesStore.get(def.id) : undefined);
  const isLoading = $derived(def ? loadingStore.isLoading(def.id) : false);
  const summary = $derived(status ? stateSummary(status) : null);
  const permissionInfo = $derived(def ? permissionInfoFor(def.required_level) : null);

  const matrix = $derived(def && status ? buildMatrix(def.options, status.observed?.changes ?? null) : []);
  const showNow = $derived(matrix.some((r) => r.now !== null));
  const scripted = $derived(def ? def.options.filter((o) => o.commands.length > 0) : []);

  const KIND_LABEL: Record<ChangeKind, { label: string; icon: string }> = {
    registry: { label: "Registry", icon: "mdi:database" },
    service: { label: "Services", icon: "mdi:server" },
    task: { label: "Scheduled tasks", icon: "mdi:calendar" },
    hosts: { label: "Hosts file", icon: "mdi:file-document-outline" },
    firewall: { label: "Firewall", icon: "mdi:shield-outline" },
  };

  let entries = $state<EntrySummary[]>([]);
  let entriesLoading = $state(false);
  let busySeq = $state<number | null>(null);
  let keeping = $state(false);
  // Gated on entries, not `has_backup`: an all-invalid history still needs a discard path (ADR-0002).
  const hasHistory = $derived(!!status?.has_backup || entriesLoading || entries.length > 0);

  let lastTab = navigationStore.activeTab;
  $effect(() => {
    const tab = navigationStore.activeTab;
    if (tab !== lastTab) {
      lastTab = tab;
      closeTweakDetailsModal();
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

  function attentionHint(item: AttentionItem): string {
    if (item.kind === "no_undo") return "(this one cannot be retried)";
    if (item.class === "busy") return "(retrying later may succeed)";
    if (item.class === "access_denied" && elevationStore.level === "User") return "(restart as administrator to retry)";
    return "";
  }

  const holderName = (id: string) => tweaksStore.getById(id)?.definition.name ?? id;

  function entryTime(stamp: string): string {
    const date = new Date(stamp);
    return Number.isNaN(date.getTime()) ? stamp : date.toLocaleString();
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

  async function keepCurrent() {
    const t = tweak;
    if (!t) return;
    keeping = true;
    try {
      if (await keepWithConfirm(t.definition)) {
        entries = [];
        return;
      }
      // Declined or refused: re-read so the list does not claim the entries are gone.
      entries = await listSnapshotEntries(t.definition.id);
    } catch (e) {
      toastStore.warning(`The snapshot entries could not be re-read: ${errorMessage(e)}`);
    } finally {
      keeping = false;
    }
  }
</script>

{#snippet metaItem(icon: string, label: string, tone: string, tip?: string | null)}
  <span class="inline-flex items-center gap-1 {tone}" use:tooltip={tip ?? null}>
    <Icon {icon} width="14" class="shrink-0" />
    {label}
  </span>
{/snippet}

{#snippet cellText(cell: MatrixCell | null)}
  {#if cell}
    <span class={cell.removal ? "text-foreground-muted italic" : "font-medium"}>
      {cell.text}
    </span>
    {#if cell.note}<span class="ml-1 text-caption text-foreground-subtle">{cell.note}</span>{/if}
  {:else}
    <span class="text-foreground-subtle" use:tooltip={"Left as it is by this option"}>—</span>
  {/if}
{/snippet}

<Modal
  open={tweakDetailsModalStore.tweakId !== null}
  onclose={closeTweakDetailsModal}
  size="full"
  labelledBy="tweak-details-title"
>
  {#if tweak && def && status && summary}
    {@const isFavorite = favoritesStore.isFavorite(def.id)}
    <header class="shrink-0 border-b border-border px-6 pt-5 pb-4">
      <div class="flex items-start gap-4">
        <div class="min-w-0 flex-1">
          <h2 id="tweak-details-title" class="m-0 font-display text-xl leading-snug font-semibold wrap-break-word">
            {def.name}
          </h2>
          <p class="m-0 mt-1 text-ui leading-relaxed text-foreground-muted">{def.description}</p>
        </div>
        <div class="flex shrink-0 items-center gap-0.5">
          <button
            type="button"
            class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-md hover:bg-muted {isFavorite
              ? 'text-warning'
              : 'text-foreground-muted hover:text-foreground'}"
            aria-label={isFavorite ? "Remove from favorites" : "Add to favorites"}
            aria-pressed={isFavorite}
            use:tooltip={isFavorite ? "Remove from favorites" : "Add to favorites"}
            onclick={() => favoritesStore.toggle(def.id)}
          >
            <Icon icon={isFavorite ? "mdi:star" : "mdi:star-outline"} width="18" />
          </button>
          <button
            type="button"
            class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-md text-foreground-muted hover:bg-muted hover:text-foreground"
            aria-label="Close details"
            onclick={closeTweakDetailsModal}
          >
            <Icon icon="mdi:close" width="18" />
          </button>
        </div>
      </div>

      <div class="mt-3 flex flex-wrap items-center justify-between gap-x-6 gap-y-3">
        <div class="flex flex-wrap items-center gap-x-3.5 gap-y-1 text-xs">
          {@render metaItem(summary.icon, summary.label, TONE_TEXT[summary.tone], "Current state")}
          {#if pendingChange}
            {@render metaItem("mdi:arrow-right", `${pendingChange.optionLabel} pending`, "text-warning")}
          {/if}
          {@render metaItem(
            "mdi:shield-half-full",
            `${RISK_INFO[def.risk_level].name} risk`,
            TONE_TEXT[RISK_TONE[def.risk_level]],
            RISK_INFO[def.risk_level].description,
          )}
          {@render metaItem(
            permissionInfo?.icon ?? "mdi:account",
            permissionInfo?.name ?? "Standard user",
            "text-foreground-muted",
            permissionInfo?.description,
          )}
          {#if def.requires_reboot}
            {@render metaItem("mdi:restart", "Restart needed", "text-foreground-muted", "After applying or restoring")}
          {/if}
          {#if !def.reversible}
            {@render metaItem("mdi:undo-variant", "Not reversible", "text-warning")}
          {/if}
          {@render metaItem(
            "mdi:history",
            status.has_backup ? "Snapshot saved" : "No snapshot",
            status.has_backup ? "text-foreground-muted" : "text-foreground-subtle",
          )}
        </div>
        <div class="flex flex-wrap items-center gap-2">
          {#if status.has_backup && !status.attention}
            <button
              type="button"
              class="inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-secondary px-3 text-ui font-medium hover:bg-secondary-hover disabled:cursor-not-allowed disabled:opacity-50"
              disabled={isLoading || def.availability.state !== "available"}
              use:tooltip={"Restore the state saved before the last change"}
              onclick={() => restoreTweak(def, isHighRisk(def.risk_level))}
            >
              <Icon icon="mdi:history" width="16" />
              Restore
            </button>
          {/if}
          <TweakControl {tweak} class="max-w-md" />
        </div>
      </div>
    </header>

    <ModalBody class="space-y-5 select-text">
      {#if def.warning}
        <div class="flex gap-3 rounded-lg border border-warning/30 bg-warning/8 p-3">
          <Icon icon="mdi:alert" width="18" class="mt-0.5 shrink-0 text-warning" />
          <p class="m-0 text-ui leading-relaxed text-foreground">{def.warning}</p>
        </div>
      {/if}

      {#if def.availability.state !== "available"}
        <div class="flex gap-3 rounded-lg border border-warning/30 bg-warning/8 p-3">
          <Icon icon="mdi:shield-lock-outline" width="18" class="mt-0.5 shrink-0 text-warning" />
          <p class="m-0 text-ui leading-relaxed">
            <span class="font-semibold">{availabilityTitle(def.availability)}.</span>
            <span class="text-foreground-muted">{def.availability.reason}</span>
          </p>
        </div>
      {/if}

      {#if status.state === "unavailable" && status.unavailableReason}
        <div class="flex gap-3 rounded-lg border border-border bg-card p-3">
          <Icon icon="mdi:cancel" width="18" class="mt-0.5 shrink-0 text-foreground-muted" />
          <p class="m-0 text-ui text-foreground-muted">{status.unavailableReason}</p>
        </div>
      {/if}

      {#if status.state === "unknown" && status.unknownReasons.length > 0}
        <div class="rounded-lg border border-warning/30 bg-warning/8 p-3">
          <p class="m-0 mb-1.5 flex items-center gap-2 text-ui font-semibold">
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
          <p class="m-0 flex gap-2 text-ui leading-relaxed">
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
          <div class="mt-3 ml-6.5 flex flex-wrap gap-2">
            {#if status.has_backup}
              <button
                type="button"
                class="inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-secondary px-3 text-ui font-medium hover:bg-secondary-hover disabled:cursor-not-allowed disabled:opacity-50"
                disabled={isLoading || def.availability.state !== "available"}
                onclick={() => restoreTweak(def, isHighRisk(def.risk_level))}
              >
                <Icon icon="mdi:history" width="16" />
                {status.attention.reason === "restore_failed" ? "Retry restore" : "Restore"}
              </button>
            {/if}
            <!-- Consent stays reachable whenever a record exists, entries left or not (ADR-0002). -->
            <button
              type="button"
              class="inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-secondary px-3 text-ui font-medium hover:bg-secondary-hover disabled:cursor-not-allowed disabled:opacity-50"
              onclick={keepCurrent}
              disabled={keeping || isLoading}
            >
              <Icon icon={keeping ? "mdi:loading" : "mdi:check"} width="16" class={keeping ? "animate-spin" : ""} />
              Keep current state
            </button>
          </div>
        </div>
      {/if}

      {#if status.residues.length > 0 || status.heldShared.length > 0}
        <div class="space-y-2 rounded-lg border border-border bg-card p-3 text-ui text-foreground-muted">
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
                Shared settings held: {status.heldShared
                  .map((h) => `${h.shared} (${h.holders.map(holderName).join(", ")})`)
                  .join("; ")}
              </span>
            </p>
          {/if}
        </div>
      {/if}

      <section aria-labelledby="tweak-changes-title">
        <h3 id="tweak-changes-title" class="m-0 mb-2.5 flex items-center gap-2 text-ui font-semibold">
          <Icon icon="mdi:tune-variant" width="16" class="text-foreground-muted" />
          What each option sets
        </h3>
        {#if matrix.length === 0}
          <p class="m-0 text-ui text-foreground-muted">
            {scripted.length > 0 ? "This tweak works through the scripts below." : "No system settings are listed."}
          </p>
        {:else}
          <div class="overflow-x-auto rounded-lg border border-border">
            <table class="w-full border-collapse text-left text-ui">
              <thead>
                <tr class="border-b border-border bg-muted/40 text-xs">
                  <th
                    scope="col"
                    class="sticky left-0 z-raised min-w-56 bg-card px-3 py-2 font-medium text-foreground-muted"
                  >
                    Setting
                  </th>
                  {#if showNow}
                    <th scope="col" class="min-w-36 bg-warning/8 px-3 py-2 font-semibold">
                      This PC now
                      <span class="block text-caption font-normal text-foreground-muted">no single option matches</span>
                    </th>
                  {/if}
                  {#each def.options as option (option.label)}
                    {@const current = status.activeOption === option.label}
                    {@const pending = pendingChange?.optionLabel === option.label}
                    {@const unavailable = status.unavailableOptions.find((u) => u.label === option.label)}
                    <th
                      scope="col"
                      class="min-w-36 px-3 py-2 font-semibold {current
                        ? 'bg-accent/12'
                        : pending
                          ? 'bg-warning/10'
                          : ''}"
                    >
                      {option.label}
                      {#if current}
                        <span class="block text-caption font-medium text-accent">Current</span>
                      {:else if pending}
                        <span class="block text-caption font-medium text-warning">Pending</span>
                      {:else if unavailable}
                        <span class="block text-caption font-normal text-warning" use:tooltip={unavailable.reason}>
                          Unavailable here
                        </span>
                      {/if}
                    </th>
                  {/each}
                </tr>
              </thead>
              <tbody>
                {#each matrix as row, r (`${row.kind}-${row.location}-${row.title}-${r}`)}
                  {#if r === 0 || matrix[r - 1].kind !== row.kind}
                    <tr class="border-b border-border">
                      <th
                        scope="rowgroup"
                        colspan={def.options.length + (showNow ? 2 : 1)}
                        class="px-3 pt-3 pb-1.5 text-caption font-semibold tracking-wide text-foreground-muted uppercase"
                      >
                        <span class="sticky left-3 inline-flex items-center gap-1.5">
                          <Icon icon={KIND_LABEL[row.kind].icon} width="13" />
                          {KIND_LABEL[row.kind].label}
                        </span>
                      </th>
                    </tr>
                  {/if}
                  <tr class="border-b border-border last:border-b-0">
                    <th scope="row" class="sticky left-0 z-raised max-w-80 bg-card px-3 py-2 align-top font-normal">
                      <span class="flex flex-wrap items-baseline gap-x-2">
                        <span class="font-mono text-code font-semibold break-all">{row.title}</span>
                        {#if row.type}<span class="text-caption text-foreground-subtle">{row.type}</span>{/if}
                      </span>
                      <span
                        class="mt-0.5 block truncate font-mono text-caption text-foreground-muted"
                        use:tooltip={row.location}
                      >
                        {row.location}
                      </span>
                    </th>
                    {#if showNow}
                      {@const agrees = optionsMatchingNow(row, def.optionLabels, status.observed?.agreement ?? [])}
                      <td class="bg-warning/5 px-3 py-2 align-top break-all">
                        {@render cellText(row.now)}
                        {#if row.now}
                          <span class="mt-0.5 block text-caption {agrees.length ? 'text-success' : 'text-warning'}">
                            {agrees.length ? `✓ ${agrees.join(", ")}` : "No option"}
                          </span>
                        {/if}
                      </td>
                    {/if}
                    {#each def.options as option, i (option.label)}
                      <td
                        class="px-3 py-2 align-top break-all {status.activeOption === option.label
                          ? 'bg-accent/8'
                          : pendingChange?.optionLabel === option.label
                            ? 'bg-warning/6'
                            : ''}"
                      >
                        {@render cellText(row.cells[i])}
                      </td>
                    {/each}
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </section>

      {#if scripted.length > 0}
        <section aria-labelledby="tweak-scripts-title">
          <h3 id="tweak-scripts-title" class="m-0 mb-2.5 flex items-center gap-2 text-ui font-semibold">
            <Icon icon="mdi:console" width="16" class="text-foreground-muted" />
            Scripts
          </h3>
          <div class="space-y-1.5">
            {#each scripted as option (option.label)}
              <details class="group rounded-lg border border-border bg-card">
                <summary class="flex cursor-pointer list-none items-center gap-2 px-3 py-2 text-ui">
                  <Icon
                    icon="mdi:chevron-right"
                    width="16"
                    class="shrink-0 transition-transform duration-normal group-open:rotate-90"
                  />
                  <span class="font-medium">{option.label}</span>
                  <span class="text-xs text-foreground-muted">
                    runs {option.commands.length === 1 ? "1 script" : `${option.commands.length} scripts`}
                  </span>
                </summary>
                <div class="space-y-1.5 border-t border-border p-3">
                  {#each option.commands as cmd, idx (idx)}
                    <code
                      class="block rounded-md border border-border bg-background px-3 py-2 font-mono text-caption break-all whitespace-pre-wrap text-foreground/85"
                      >{cmd}</code
                    >
                  {/each}
                </div>
              </details>
            {/each}
          </div>
        </section>
      {/if}

      <div class="@container">
        <div class="grid items-start gap-5 @3xl:grid-cols-main-aside">
          {#if def.info}
            <section aria-labelledby="tweak-about-title" class={hasHistory ? "" : "@3xl:col-span-2"}>
              <h3 id="tweak-about-title" class="m-0 mb-2.5 flex items-center gap-2 text-ui font-semibold">
                <Icon icon="mdi:information-outline" width="16" class="text-foreground-muted" />
                About
              </h3>
              <MarkdownText content={def.info} />
            </section>
          {/if}

          {#if hasHistory}
            <section aria-labelledby="tweak-history-title" class={def.info ? "" : "@3xl:col-span-2"}>
              <h3 id="tweak-history-title" class="m-0 mb-2.5 flex items-center gap-2 text-ui font-semibold">
                <Icon icon="mdi:history" width="16" class="text-foreground-muted" />
                Snapshot history
              </h3>
              {#if entriesLoading}
                <div class="flex items-center gap-2 text-ui text-foreground-muted">
                  <Icon icon="mdi:loading" width="16" class="animate-spin" />
                  Loading…
                </div>
              {:else if entries.length === 0}
                <p class="m-0 text-ui text-foreground-muted italic">No snapshot entries.</p>
              {:else}
                <div class="animate-fade-in space-y-1.5">
                  {#each entries as entry (entry.seq)}
                    <div
                      class="flex items-center justify-between gap-3 rounded-md border border-border bg-card px-3 py-2"
                      transition:expand
                    >
                      <div class="min-w-0 text-xs wrap-break-word">
                        <span class="font-semibold">#{entry.seq}</span>
                        <span class="text-foreground-muted"> · {entryValidity(entry)}</span>
                        {#if entry.timestamp}<span class="text-foreground-muted">
                            · {entryTime(entry.timestamp)}</span
                          >{/if}
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
      </div>
    </ModalBody>
  {/if}
</Modal>
