<script lang="ts">
  import { MarkdownText } from "$lib/components/shared";
  import { Callout, Modal, ModalBody, ModalHeader } from "$lib/components/ui";
  import { appsStore } from "$lib/stores/apps.svelte";
  import { appDetailsModalStore } from "$lib/stores/detailsModal.svelte";
  import { isPermanent, PERMANENT_REMOVAL } from "$lib/utils/appPresentation";

  const app = $derived(appDetailsModalStore.shownId ? appsStore.app(appDetailsModalStore.shownId) : undefined);
  const permanent = $derived(app ? isPermanent(appsStore.status(app.id)) : false);
</script>

<Modal open={appDetailsModalStore.openId !== null} onclose={appDetailsModalStore.close} size="lg">
  {#if app}
    <ModalHeader title={app.name} size="lg" onclose={appDetailsModalStore.close}>
      <p class="m-0 mt-1 text-sm text-foreground-muted">{app.description}</p>
    </ModalHeader>
    <ModalBody class="flex flex-col gap-4">
      {#if app.warning}
        <Callout tone="warning" density="note" icon="mdi:alert">
          <span>{app.warning}</span>
        </Callout>
      {/if}
      {#if permanent}
        <p class="m-0 text-sm text-foreground-muted">{PERMANENT_REMOVAL}</p>
      {/if}
      {#if app.info}
        <MarkdownText content={app.info} />
      {/if}
    </ModalBody>
  {/if}
</Modal>
