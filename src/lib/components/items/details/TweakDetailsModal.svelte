<script lang="ts">
  import { MarkdownText } from "$lib/components/shared";
  import { Modal, ModalBody, PanelHeading } from "$lib/components/ui";
  import { tweakDetailsModalStore } from "$lib/stores/detailsModal.svelte";
  import { createSnapshotHistory } from "$lib/stores/snapshotHistory.svelte";
  import { tweaksStore } from "$lib/stores/tweaksData.svelte";
  import ChangeMatrix from "./ChangeMatrix.svelte";
  import TweakDetailsHeader from "./TweakDetailsHeader.svelte";
  import ScriptList from "./ScriptList.svelte";
  import SnapshotHistory from "./SnapshotHistory.svelte";
  import StatusCallouts from "./StatusCallouts.svelte";

  const aboutId = $props.id();
  const open = $derived(tweakDetailsModalStore.openId !== null);
  const tweak = $derived.by(() => {
    const id = tweakDetailsModalStore.shownId;
    return id ? (tweaksStore.tweak(id) ?? null) : null;
  });
  const scripted = $derived(tweak?.definition.options.filter((o) => o.commands.length > 0) ?? []);
  const history = createSnapshotHistory(
    () => tweak,
    () => open,
  );
</script>

<Modal {open} onclose={tweakDetailsModalStore.close} size="full">
  {#if tweak}
    {@const def = tweak.definition}
    <TweakDetailsHeader {tweak} onclose={tweakDetailsModalStore.close} />

    <ModalBody class="space-y-5 select-text">
      <StatusCallouts {tweak} />
      <ChangeMatrix {tweak} scripted={scripted.length > 0} />
      {#if scripted.length > 0}<ScriptList options={scripted} />{/if}

      <div class="@container">
        <div class="grid items-start gap-5 @3xl:grid-cols-main-aside">
          {#if def.info}
            <section aria-labelledby={aboutId} class={history.visible ? "" : "@3xl:col-span-2"}>
              <PanelHeading id={aboutId} icon="mdi:information-outline" class="mb-2.5">About</PanelHeading>
              <MarkdownText content={def.info} />
            </section>
          {/if}
          {#if history.visible}
            <SnapshotHistory {history} class={def.info ? "" : "@3xl:col-span-2"} />
          {/if}
        </div>
      </div>
    </ModalBody>
  {/if}
</Modal>
