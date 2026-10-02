<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon, MarkdownText } from "$lib/components/shared";
  import { Button, HighlightedText, IconButton, Modal, ModalBody, ModalHeader } from "$lib/components/ui";
  import { appsStore } from "$lib/stores/apps.svelte";
  import { confirm } from "$lib/stores/confirm.svelte";
  import { pageFilterStore } from "$lib/stores/pageFilter.svelte";
  import type { AppView, RiskLevel } from "$lib/types";
  import { permissionInfoFor, RISK_INFO } from "$lib/types";
  import { searchHighlight } from "$lib/utils/searchHighlight.svelte";
  import { RISK_TONE, TONE_TEXT } from "$lib/utils/tweakPresentation";
  import type { Snippet } from "svelte";

  interface Props {
    app: AppView;
    titleSlot?: Snippet;
    descriptionSlot?: Snippet;
    /** Extra meta-line content, e.g. the category in search results. */
    context?: Snippet;
  }

  let { app, titleSlot, descriptionSlot, context }: Props = $props();

  const status = $derived(appsStore.status(app.id));
  const filterMatch = $derived(titleSlot ? null : pageFilterStore.match(app.id));
  const presence = $derived(status?.presence);
  const busy = $derived(appsStore.isBusy(app.id));
  const appError = $derived(appsStore.error(app.id));
  const permanent = $derived(status?.install_route === "none" && presence?.state !== "absent");

  const riskLevel = $derived(app.risk.toLowerCase() as RiskLevel);
  const riskInfo = $derived(RISK_INFO[riskLevel]);

  const chip = $derived.by((): { label: string; tip: string; icon: string; tone: string; spin?: boolean } => {
    const muted = "text-foreground-muted";
    const warn = "text-warning";
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
            tone: "text-info",
          }
        : {
            label: "Installed",
            tip: "Installed on this PC",
            icon: "mdi:check-circle",
            tone: "text-success",
          };
    }
    if (presence.state === "absent") {
      return { label: "Not installed", tip: "Not installed on this PC", icon: "mdi:circle-outline", tone: muted };
    }
    return {
      label: presence.needs_elevation ? "Unknown, needs admin" : "Unknown",
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
    remove: { label: "Remove", icon: "mdi:delete-outline", aria: "Remove", tone: "text-error" },
    install: { label: "Install", icon: "mdi:download", aria: "Install", tone: "text-accent" },
    store: { label: "Get in Store", icon: "mdi:open-in-new", aria: "Open the Microsoft Store page for", tone: "" },
  } as const;

  let showDetails = $state(false);
  let warningOpen = $state(false);

  const scope = $derived(app.source === "appx" ? "for every account on this PC" : "for your account");
  const confirmMessage = $derived(
    (permanent
      ? `${app.name} is removed ${scope} and has no install source on this PC, so this cannot be undone.`
      : `${app.name} is removed ${scope}. You can reinstall it later from here.`) +
      (app.warning ? ` ${app.warning}` : ""),
  );

  async function handleAction() {
    if (!action || action.disabledReason !== null) return;
    if (action.kind === "install") void appsStore.install(app.id);
    else if (action.kind === "store") void appsStore.openStorePage(app.id);
    else if (
      await confirm({ title: `Remove ${app.name}?`, message: confirmMessage, confirmText: "Remove", variant: "danger" })
    )
      void appsStore.remove(app.id);
  }

  let rowEl = $state<HTMLElement | null>(null);
  const highlight = searchHighlight(
    () => app.id,
    () => rowEl,
  );
</script>

<article
  id="app-{app.id}"
  bind:this={rowEl}
  class="@container relative flex min-w-0 flex-col overflow-hidden rounded-lg border border-border bg-card hover:border-border-hover {highlight.active
    ? 'tweak-highlight'
    : ''}"
  aria-busy={busy}
>
  <span
    class="absolute top-3 bottom-3 left-0 w-0.75 rounded-r-full {presence?.state === 'installed'
      ? 'bg-accent'
      : 'bg-transparent'}"
    aria-hidden="true"
  ></span>

  <div class="flex flex-1 flex-col gap-2.5 py-3 pr-3 pl-4">
    <div
      class="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-6 gap-y-1 @max-[520px]:grid-cols-1 @max-[520px]:gap-y-2"
    >
      <h3 class="m-0 text-sm leading-snug font-semibold wrap-break-word text-foreground">
        {#if titleSlot}{@render titleSlot()}{:else if filterMatch}<HighlightedText
            text={app.name}
            ranges={filterMatch.nameRanges}
            highlightClass="rounded-sm bg-accent/25 text-foreground"
          />{:else}{app.name}{/if}
      </h3>

      {#if action}
        {@const config = actionConfig[action.kind]}
        <div class="justify-self-start" use:tooltip={action.disabledReason}>
          <Button
            variant="secondary"
            size="md"
            class={config.tone}
            loading={busy}
            disabled={action.disabledReason !== null}
            onclick={handleAction}
            aria-label="{config.aria} {app.name}"
          >
            {#if !busy}<Icon icon={config.icon} width="16" />{/if}
            {config.label}
          </Button>
        </div>
      {/if}

      <p class="col-span-full m-0 text-[13px] leading-snug text-foreground-muted">
        {#if descriptionSlot}{@render descriptionSlot()}{:else if filterMatch}<HighlightedText
            text={app.description}
            ranges={filterMatch.descriptionRanges}
            highlightClass="rounded-sm bg-accent/25 text-foreground"
          />{:else}{app.description}{/if}
      </p>
    </div>

    {#if app.warning && warningOpen}
      <div
        id="app-warning-{app.id}"
        class="flex gap-2 rounded-md bg-warning/8 px-2.5 py-2 text-xs leading-relaxed text-foreground"
      >
        <Icon icon="mdi:alert" width="14" class="mt-px shrink-0 text-warning" />
        <span class="min-w-0">{app.warning}</span>
      </div>
    {/if}

    {#if appError}
      <div
        class="flex items-start gap-2 rounded-md border border-error/30 bg-error/8 px-2.5 py-2 text-xs text-error"
        role="alert"
      >
        <Icon icon="mdi:alert-circle" width="14" class="mt-px shrink-0" />
        <span class="min-w-0 flex-1 wrap-break-word">{appError}</span>
        <button
          type="button"
          class="flex shrink-0 cursor-pointer rounded p-0.5 text-error/70 hover:bg-error/10 hover:text-error"
          onclick={() => appsStore.clearError(app.id)}
          aria-label="Dismiss error"
        >
          <Icon icon="mdi:close" width="14" />
        </button>
      </div>
    {/if}

    <div class="mt-auto flex flex-wrap items-center gap-x-3.5 gap-y-1 text-xs">
      <span class="inline-flex items-center gap-1 {chip.tone}" use:tooltip={chip.tip}>
        <Icon icon={chip.icon} width="13" class="shrink-0 {chip.spin ? 'animate-spin' : ''}" />
        {chip.label}
      </span>
      <span class="inline-flex items-center gap-1 {TONE_TEXT[RISK_TONE[riskLevel]]}" use:tooltip={riskInfo.description}>
        <Icon icon="mdi:shield-half-full" width="13" class="shrink-0" />
        {riskInfo.name} risk
      </span>
      {#if app.warning}
        <button
          type="button"
          class="inline-flex cursor-pointer items-center gap-1 rounded text-warning hover:underline"
          aria-expanded={warningOpen}
          aria-controls={warningOpen ? `app-warning-${app.id}` : undefined}
          use:tooltip={warningOpen ? "Hide warning" : "Show warning"}
          onclick={() => (warningOpen = !warningOpen)}
        >
          <Icon icon="mdi:alert" width="13" class="shrink-0" />
          Warning
        </button>
      {/if}
      {#if permissionInfo}
        <span class="inline-flex items-center gap-1 text-foreground-muted" use:tooltip={permissionInfo.description}>
          <Icon icon={permissionInfo.icon} width="13" class="shrink-0" />
          {permissionInfo.name}
        </span>
      {/if}
      {#if permanent}
        <span
          class="inline-flex items-center gap-1 text-warning"
          use:tooltip={"No install source on this PC: once removed, it cannot be reinstalled from here"}
        >
          <Icon icon="mdi:alert" width="13" class="shrink-0" />
          Permanent
        </span>
      {/if}
      {#if context}{@render context()}{/if}
      <button
        type="button"
        class="ml-auto inline-flex h-7 cursor-pointer items-center gap-1.5 rounded-md px-2 text-xs font-medium text-foreground-muted hover:bg-muted hover:text-foreground"
        onclick={() => (showDetails = true)}
        aria-label="Open details for {app.name}"
      >
        <Icon icon="mdi:chevron-right" width="15" />
        Details
      </button>
    </div>
  </div>
</article>

<Modal open={showDetails} onclose={() => (showDetails = false)} size="lg" labelledBy="app-details-{app.id}">
  <ModalHeader id="app-details-{app.id}">
    <div class="min-w-0">
      <h2 class="m-0 font-display text-lg font-semibold wrap-break-word text-foreground">{app.name}</h2>
      <p class="m-0 mt-1 text-sm text-foreground-muted">{app.description}</p>
    </div>
    <IconButton icon="mdi:close" onclick={() => (showDetails = false)} aria-label="Close" />
  </ModalHeader>
  <ModalBody class="flex flex-col gap-4">
    {#if app.warning}
      <div class="flex items-start gap-2 rounded-lg border border-warning/30 bg-warning/8 px-3 py-2 text-sm">
        <Icon icon="mdi:alert" width="16" class="mt-0.5 shrink-0 text-warning" />
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
