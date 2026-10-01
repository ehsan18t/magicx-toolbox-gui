<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { ConfirmDialog } from "$lib/components/modals";
  import { Icon, MarkdownText } from "$lib/components/shared";
  import { Button, IconButton, Modal, ModalBody, ModalHeader, StatusBadge } from "$lib/components/ui";
  import { appsStore } from "$lib/stores/apps.svelte";
  import { searchStore } from "$lib/stores/search.svelte";
  import type { AppView, RiskLevel } from "$lib/types";
  import { permissionInfoFor, RISK_INFO } from "$lib/types";
  import type { Snippet } from "svelte";

  interface Props {
    app: AppView;
    titleSlot?: Snippet;
    descriptionSlot?: Snippet;
  }

  let { app, titleSlot, descriptionSlot }: Props = $props();

  const status = $derived(appsStore.status(app.id));
  const presence = $derived(status?.presence);
  const busy = $derived(appsStore.isBusy(app.id));
  const appError = $derived(appsStore.error(app.id));
  const permanent = $derived(status?.install_route === "none" && presence?.state !== "absent");

  const riskLevel = $derived(app.risk.toLowerCase() as RiskLevel);
  const riskInfo = $derived(RISK_INFO[riskLevel]);
  const riskConfig: Record<RiskLevel, { icon: string; variant: "success" | "warning" | "orange" | "error" }> = {
    low: { icon: "mdi:check-circle", variant: "success" },
    medium: { icon: "mdi:alert", variant: "warning" },
    high: { icon: "mdi:alert-circle", variant: "orange" },
    critical: { icon: "mdi:alert-octagon", variant: "error" },
  };

  const chip = $derived.by((): { label: string; tip: string; icon: string; tone: string; spin?: boolean } => {
    const muted = "bg-muted/50 text-foreground-muted";
    const warn = "bg-warning/10 text-warning";
    if (!presence) {
      return appsStore.scanError
        ? { label: "Unknown", tip: appsStore.scanError, icon: "mdi:help-circle-outline", tone: warn }
        : {
            label: "Checking",
            tip: "Checking whether this app is installed…",
            icon: "mdi:loading",
            tone: muted,
            spin: true,
          };
    }
    if (presence.state === "installed") {
      return presence.provisioned_only
        ? {
            label: "Provisioned only",
            tip: "Not installed for any account yet, but Windows installs it for every new account.",
            icon: "mdi:package-variant",
            tone: "bg-info/10 text-info",
          }
        : {
            label: "Installed",
            tip: "Installed on this PC",
            icon: "mdi:check-circle",
            tone: "bg-success/10 text-success",
          };
    }
    if (presence.state === "absent") {
      return { label: "Not installed", tip: "Not installed on this PC", icon: "mdi:circle-outline", tone: muted };
    }
    return {
      label: presence.needs_elevation ? "Unknown · needs elevation" : "Unknown",
      tip: presence.needs_elevation ? `${presence.reason} Restart as administrator to resolve.` : presence.reason,
      icon: "mdi:help-circle-outline",
      tone: warn,
    };
  });

  // One action per card; null only for an absent app with no install route, which is hidden anyway.
  const action = $derived.by((): { kind: "remove" | "install" | "store"; disabledReason: string | null } | null => {
    if (!presence) return { kind: "remove", disabledReason: chip.tip };
    if (presence.state === "unknown") return { kind: "remove", disabledReason: presence.reason };
    if (presence.state === "installed") {
      const a = app.remove_availability;
      return { kind: "remove", disabledReason: a.state === "available" ? null : a.reason };
    }
    if (status?.install_route === "winget") {
      const a = app.install_availability;
      return { kind: "install", disabledReason: a.state === "available" ? null : a.reason };
    }
    if (status?.install_route === "store_page") return { kind: "store", disabledReason: null };
    return null;
  });

  const permissionInfo = $derived(action?.kind === "remove" ? permissionInfoFor("Admin") : null);

  const actionConfig = {
    remove: { label: "Remove", icon: "mdi:delete-outline", aria: "Remove", tone: "text-error hover:bg-error/10" },
    install: { label: "Install", icon: "mdi:download", aria: "Install", tone: "text-accent hover:bg-accent/10" },
    store: { label: "Get in Store", icon: "mdi:open-in-new", aria: "Open the Microsoft Store page for", tone: "" },
  } as const;

  let showRemoveConfirm = $state(false);
  let showDetails = $state(false);

  const scope = $derived(app.source === "appx" ? "for every account on this PC" : "for your account");
  const confirmMessage = $derived(
    (permanent
      ? `${app.name} is removed ${scope} and has no install source on this PC, so this cannot be undone.`
      : `${app.name} is removed ${scope}. You can reinstall it later from this card.`) +
      (app.warning ? ` ${app.warning}` : ""),
  );

  function handleAction() {
    if (!action || action.disabledReason !== null) return;
    if (action.kind === "remove") showRemoveConfirm = true;
    else if (action.kind === "install") void appsStore.install(app.id);
    else void appsStore.openStorePage(app.id);
  }

  function confirmRemove() {
    showRemoveConfirm = false;
    void appsStore.remove(app.id);
  }

  const isHighlighting = $derived(searchStore.highlightTweakId === app.id);

  $effect(() => {
    if (!isHighlighting) return;
    const timer = setTimeout(() => searchStore.clearHighlight(), 1500);
    return () => clearTimeout(timer);
  });
</script>

<article
  id="app-{app.id}"
  class="flex min-w-0 flex-col rounded-lg border border-border bg-card px-3 pt-2.5 pb-2 transition-all duration-200 hover:border-border-hover {isHighlighting
    ? 'tweak-highlight'
    : ''}"
  aria-busy={busy}
>
  <div class="flex items-start justify-between gap-4">
    <div class="min-w-0 flex-1">
      <h3 class="m-0 flex flex-wrap items-center gap-2 text-[13px] leading-tight font-semibold text-foreground">
        {#if titleSlot}
          {@render titleSlot()}
        {:else}
          {app.name}
        {/if}
        <span
          class="inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-medium tracking-wide {chip.tone}"
          use:tooltip={chip.tip}
        >
          <Icon icon={chip.icon} width="10" class={chip.spin ? "animate-spin" : ""} />
          {chip.label}
        </span>
      </h3>
      <p class="m-0 mt-1.5 mb-1.5 text-[12px] leading-relaxed text-foreground-muted/80">
        {#if descriptionSlot}
          {@render descriptionSlot()}
        {:else}
          {app.description}
        {/if}
      </p>
    </div>

    {#if action}
      {@const config = actionConfig[action.kind]}
      <div class="shrink-0 pt-0.5" use:tooltip={action.disabledReason}>
        <Button
          variant="outline"
          size="xs"
          class={config.tone}
          loading={busy}
          disabled={action.disabledReason !== null}
          onclick={handleAction}
          aria-label="{config.aria} {app.name}"
        >
          {#if !busy}
            <Icon icon={config.icon} width="14" />
          {/if}
          {config.label}
        </Button>
      </div>
    {/if}
  </div>

  {#if appError}
    <div
      class="mt-2 flex items-start gap-2 rounded-lg border border-error/20 bg-error/5 px-3 py-2 text-xs leading-relaxed text-error"
      role="alert"
    >
      <Icon icon="mdi:alert-circle" width="16" class="mt-0.5 shrink-0" />
      <span class="flex-1 wrap-break-word">{appError}</span>
      <button
        type="button"
        class="flex shrink-0 cursor-pointer items-center justify-center rounded border-0 bg-transparent p-0.5 text-error/70 transition-colors duration-150 hover:bg-error/10 hover:text-error focus-visible:ring-2 focus-visible:ring-error/40 focus-visible:outline-none"
        onclick={() => appsStore.clearError(app.id)}
        aria-label="Dismiss error"
      >
        <Icon icon="mdi:close" width="16" />
      </button>
    </div>
  {/if}

  <div class="mt-auto flex flex-wrap items-center gap-x-4 gap-y-2 border-t border-border/40 pt-2.5">
    <div class="flex min-w-0 flex-1 flex-wrap items-center gap-2">
      <StatusBadge
        variant={riskConfig[riskLevel].variant}
        icon={riskConfig[riskLevel].icon}
        label={riskInfo.name}
        tooltip={riskInfo.description}
      />
      {#if permissionInfo}
        <StatusBadge
          variant="muted"
          icon={permissionInfo.icon}
          label={permissionInfo.name}
          tooltip={permissionInfo.description}
        />
      {/if}
      {#if permanent}
        <StatusBadge
          variant="warning"
          icon="mdi:alert"
          label="Permanent"
          tooltip="No install source on this PC: once removed, it cannot be reinstalled from here"
        />
      {/if}
    </div>
    <button
      type="button"
      class="hover:bg-muted/50 inline-flex cursor-pointer items-center gap-1.5 rounded-md border-0 bg-transparent px-2 py-1 text-[11px] font-medium text-foreground-muted transition-all duration-150 hover:text-foreground focus-visible:ring-2 focus-visible:ring-accent/40 focus-visible:outline-none"
      onclick={() => (showDetails = true)}
      aria-label="Open details for {app.name}"
    >
      Details
      <Icon icon="mdi:chevron-right" width="18" />
    </button>
  </div>
</article>

<ConfirmDialog
  open={showRemoveConfirm}
  title="Remove {app.name}?"
  message={confirmMessage}
  confirmText="Remove"
  variant="danger"
  onconfirm={confirmRemove}
  oncancel={() => (showRemoveConfirm = false)}
/>

<Modal open={showDetails} onclose={() => (showDetails = false)} size="lg" labelledBy="app-details-{app.id}">
  <ModalHeader id="app-details-{app.id}">
    <div class="min-w-0">
      <h2 class="m-0 truncate text-lg font-bold text-foreground">{app.name}</h2>
      <p class="m-0 mt-1 text-sm text-foreground-muted">{app.description}</p>
    </div>
    <IconButton icon="mdi:close" onclick={() => (showDetails = false)} aria-label="Close" />
  </ModalHeader>
  <ModalBody scrollable class="flex max-h-[calc(100dvh-2.5rem-6rem)] flex-col gap-4">
    {#if app.warning}
      <div
        class="flex items-start gap-2 rounded-lg border border-warning/30 bg-warning/10 px-3 py-2 text-sm text-warning"
      >
        <Icon icon="mdi:alert" width="16" class="mt-0.5 shrink-0" />
        <span>{app.warning}</span>
      </div>
    {/if}
    {#if permanent}
      <p class="m-0 text-sm text-foreground-muted">
        There is no install source for this app on this PC, so removing it cannot be undone from here.
      </p>
    {/if}
    {#if app.info}
      <MarkdownText content={app.info} />
    {/if}
  </ModalBody>
</Modal>
