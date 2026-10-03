<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { TONE_TEXT } from "$lib/design";
  import { Icon } from "$lib/components/shared";
  import { Button } from "$lib/components/ui";
  import { appDetailsModalStore } from "$lib/stores/appDetailsModal.svelte";
  import { appsStore } from "$lib/stores/apps.svelte";
  import { confirmStore } from "$lib/stores/confirm.svelte";
  import type { SearchResult } from "$lib/stores/search.svelte";
  import type { AppView } from "$lib/types";
  import {
    APP_ACTION_UI,
    APP_OPERATION_LABEL,
    appAction,
    appPresenceChip,
    isPermanent,
    removeConfirmMessage,
  } from "$lib/utils/appPresentation";
  import { elapsedClock, SECOND_MS } from "$lib/utils/time";
  import { expand } from "$lib/utils/motion";
  import { permissionInfoFor, RISK_INFO, RISK_TONE, rowDomId, toRiskLevel } from "$lib/utils/tweakPresentation";
  import type { Snippet } from "svelte";
  import ItemRow from "./ItemRow.svelte";
  import MetaItem from "./MetaItem.svelte";
  import RowAction from "./RowAction.svelte";
  import WarningNotice from "./WarningNotice.svelte";
  import WarningToggle from "./WarningToggle.svelte";

  interface Props {
    app: AppView;
    match?: SearchResult;
    context?: Snippet;
  }

  let { app, match, context }: Props = $props();

  const status = $derived(appsStore.status(app.id));
  const operation = $derived(appsStore.operation(app.id));
  const busy = $derived(operation !== undefined);
  const permanent = $derived(isPermanent(status));
  const chip = $derived(appPresenceChip(status, appsStore.scanError));
  const action = $derived(appAction(app, status, appsStore.scanError));
  const permissionInfo = $derived(action?.kind === "remove" ? permissionInfoFor("Admin") : null);
  const riskLevel = $derived(toRiskLevel(app.risk));
  const riskInfo = $derived(RISK_INFO[riskLevel]);
  const warningId = $derived(`${rowDomId("app", app.id)}-warning`);

  let warningOpen = $state(false);

  async function handleAction() {
    if (!action || action.disabledReason !== null) return;
    switch (action.kind) {
      case "install":
        void appsStore.install(app.id);
        break;
      case "store":
        void appsStore.openStorePage(app.id);
        break;
      case "remove": {
        const confirmed = await confirmStore.ask({
          title: `Remove ${app.name}?`,
          message: removeConfirmMessage(app, permanent),
          confirmText: "Remove",
          variant: "danger",
        });
        if (confirmed) void appsStore.remove(app.id);
      }
    }
  }

  // winget reports no usable progress to a redirected script, so this shows activity and elapsed time.
  // Timed from the store's start, so a remounted row keeps counting.
  let now = $state(Date.now());
  $effect(() => {
    if (!operation) return;
    now = Date.now();
    const timer = setInterval(() => (now = Date.now()), SECOND_MS);
    return () => clearInterval(timer);
  });
</script>

<ItemRow
  kind="app"
  id={app.id}
  title={app.name}
  description={app.description}
  {match}
  stripe={status?.presence.state === "installed" ? "accent" : null}
  error={appsStore.error(app.id)}
  ondismisserror={() => appsStore.clearError(app.id)}
  {context}
  aria-busy={busy}
>
  {#snippet control()}
    {#if action}
      {@const ui = APP_ACTION_UI[action.kind]}
      <div class="justify-self-start" use:tooltip={action.disabledReason}>
        <Button
          class={ui.tone && TONE_TEXT[ui.tone]}
          loading={busy}
          disabled={action.disabledReason !== null}
          onclick={handleAction}
          aria-label="{ui.aria} {app.name}"
        >
          {#if !busy}<Icon icon={ui.icon} size="md" />{/if}
          {ui.label}
        </Button>
      </div>
    {/if}
  {/snippet}

  {#snippet notices()}
    {#if operation}
      <div class="flex items-center gap-3 text-xs text-foreground-muted" role="status" transition:expand>
        <div class="h-1 flex-1 overflow-hidden rounded-full bg-muted">
          <!-- Reduced motion: one pass would end off-track, so hold a static, dimmed full bar. -->
          <div
            class="h-full w-1/3 animate-activity rounded-full bg-accent motion-reduce:w-full motion-reduce:animate-none motion-reduce:opacity-50"
          ></div>
        </div>
        <span class="tabular-nums">{APP_OPERATION_LABEL[operation.kind]}… {elapsedClock(operation.startedAt, now)}</span
        >
      </div>
    {/if}

    <WarningNotice id={warningId} text={app.warning} open={warningOpen} />
  {/snippet}

  {#snippet meta()}
    <MetaItem icon={chip.icon} label={chip.label} tone={chip.tone} tooltip={chip.tip} spin={chip.spin} />
    <MetaItem
      icon="mdi:shield-half-full"
      label="{riskInfo.name} risk"
      tone={RISK_TONE[riskLevel]}
      tooltip={riskInfo.description}
    />
    {#if app.warning}
      <WarningToggle open={warningOpen} controls={warningId} ontoggle={() => (warningOpen = !warningOpen)} />
    {/if}
    {#if permissionInfo}
      <MetaItem
        icon={permissionInfo.icon}
        label={permissionInfo.name}
        tone="neutral"
        tooltip={permissionInfo.description}
      />
    {/if}
    {#if permanent}
      <MetaItem
        icon="mdi:alert"
        label="Permanent"
        tone="warning"
        tooltip="No install source on this PC: once removed, it cannot be reinstalled from here"
      />
    {/if}
  {/snippet}

  {#snippet actions()}
    <RowAction
      icon="mdi:chevron-right"
      label="Details"
      ariaLabel="Open details for {app.name}"
      onclick={() => appDetailsModalStore.open(app.id)}
    />
  {/snippet}
</ItemRow>
