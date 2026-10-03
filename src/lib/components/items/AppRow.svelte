<script lang="ts">
  import { ActivityBar, Button, MetaItem } from "$lib/components/ui";
  import { appsStore } from "$lib/stores/apps.svelte";
  import { confirmStore } from "$lib/stores/confirm.svelte";
  import { appDetailsModalStore } from "$lib/stores/detailsModal.svelte";
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
  import { expand } from "$lib/utils/motion";
  import { elapsedClock, SECOND_MS } from "$lib/utils/time";
  import { permissionFact, riskFact, rowDomId, toRiskLevel } from "$lib/utils/tweakPresentation";
  import type { Snippet } from "svelte";
  import ItemRow from "./ItemRow.svelte";
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
  info={app.info}
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
      <Button
        class="justify-self-start"
        tone={ui.tone}
        icon={ui.icon}
        loading={busy}
        disabled={action.disabledReason !== null}
        tooltip={action.disabledReason}
        onclick={handleAction}
        aria-label="{ui.aria} {app.name}"
      >
        {ui.label}
      </Button>
    {/if}
  {/snippet}

  {#snippet notices()}
    {#if operation}
      <div class="flex items-center gap-3 text-xs text-foreground-muted" role="status" transition:expand>
        <ActivityBar class="flex-1" />
        <!-- The clock stays out of the live region, which would otherwise re-announce every second. -->
        <span class="tabular-nums"
          >{APP_OPERATION_LABEL[operation.kind]}…<span aria-hidden="true">
            {elapsedClock(operation.startedAt, now)}</span
          ></span
        >
      </div>
    {/if}

    <WarningNotice id={warningId} text={app.warning} open={warningOpen} />
  {/snippet}

  {#snippet meta()}
    <MetaItem {...chip} />
    <MetaItem {...riskFact(toRiskLevel(app.risk))} />
    {#if app.warning}
      <WarningToggle open={warningOpen} controls={warningId} ontoggle={() => (warningOpen = !warningOpen)} />
    {/if}
    {#if action?.kind === "remove"}<MetaItem {...permissionFact("Admin")} />{/if}
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
      aria-label="Open details for {app.name}"
      tooltip="Open details"
      collapses
      onclick={() => appDetailsModalStore.open(app.id)}
    />
  {/snippet}
</ItemRow>
