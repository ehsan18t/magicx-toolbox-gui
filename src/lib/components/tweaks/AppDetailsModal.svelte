<script lang="ts">
  import { Icon, MarkdownText } from "$lib/components/shared";
  import { Callout, ICON_SIZE, Modal, ModalBody, ModalHeader } from "$lib/components/ui";
  import { appDetailsModalStore } from "$lib/stores/appDetailsModal.svelte";
  import { appsStore } from "$lib/stores/apps.svelte";
  import { isPermanent } from "$lib/utils/appPresentation";

  const app = $derived(appDetailsModalStore.shownId ? appsStore.app(appDetailsModalStore.shownId) : undefined);
  const permanent = $derived(app ? isPermanent(appsStore.status(app.id)) : false);
</script>

<Modal open={appDetailsModalStore.appId !== null} onclose={appDetailsModalStore.close} size="lg">
  {#if app}
    <ModalHeader title={app.name} size="lg" onclose={appDetailsModalStore.close}>
      <p class="m-0 mt-1 text-sm text-foreground-muted">{app.description}</p>
    </ModalHeader>
    <ModalBody class="flex flex-col gap-4">
      {#if app.warning}
        <!-- The icon is a child: Callout sizes its own icon per density, and this one is between sizes. -->
        <Callout tone="warning" class="flex items-start gap-2 py-2 text-sm">
          <Icon icon="mdi:alert" width={ICON_SIZE.md} class="mt-0.5 shrink-0 text-warning" />
          <span>{app.warning}</span>
        </Callout>
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
  {/if}
</Modal>
