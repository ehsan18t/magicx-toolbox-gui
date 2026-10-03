<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import {
    Badge,
    Button,
    Callout,
    Card,
    EmptyState,
    Modal,
    ModalBody,
    ModalFooter,
    SectionCard,
    SettingRow,
    Switch,
    TextArea,
    TextField,
  } from "$lib/components/ui";
  import { modalStore } from "$lib/stores/modal.svelte";
  import { profileStore } from "$lib/stores/profile.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { categoriesStore, tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { plural } from "$lib/utils/format";
  import { groupByCategory, SYSTEM_DEFAULT_LABEL } from "$lib/utils/tweakPresentation";
  import { untrack } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import ProfileFileNote from "./ProfileFileNote.svelte";
  import SelectableRow from "./SelectableRow.svelte";
  import WizardHeader from "./WizardHeader.svelte";

  const formId = $props.id();
  const isOpen = $derived(modalStore.current === "profileExport");

  let step = $state<"select" | "details">("select");
  // Exclusions over the live applied list, so a tweak reported applied after open is exported too.
  const excludedIds = new SvelteSet<string>();
  let name = $state("");
  let description = $state("");
  let includeSystemState = $state(false);

  const applied = $derived(tweaksStore.list.filter((t) => t.status.state === "active"));
  const groups = $derived(groupByCategory(applied));
  const selectedIds = $derived(applied.map((t) => t.definition.id).filter((id) => !excludedIds.has(id)));
  const allSelected = $derived(selectedIds.length === applied.length);
  const canExport = $derived(selectedIds.length > 0 && name.trim().length > 0);

  // Open edge only: tracking the form fields here would wipe them as they are typed.
  $effect(() => {
    if (!isOpen) return;
    untrack(() => {
      step = "select";
      excludedIds.clear();
      name = "";
      description = "";
      includeSystemState = false;
    });
  });

  function toggleAll() {
    if (allSelected) for (const t of applied) excludedIds.add(t.definition.id);
    else excludedIds.clear();
  }

  function toggle(tweakId: string) {
    if (!excludedIds.delete(tweakId)) excludedIds.add(tweakId);
  }

  async function exportProfile() {
    const trimmed = name.trim();
    const exported = await profileStore.exportProfile(trimmed, selectedIds, {
      description: description.trim() || undefined,
      includeSystemState,
    });
    if (exported) {
      toastStore.success(`Exported profile "${trimmed}"`);
      modalStore.close();
    } else if (profileStore.exportError) {
      toastStore.error(profileStore.exportError);
    }
  }
</script>

<Modal open={isOpen} onclose={modalStore.close} size="lg">
  <WizardHeader
    title="Export profile"
    icon="mdi:export"
    step={step === "select" ? "Choose the tweaks to include" : "Name the profile"}
    onclose={modalStore.close}
  />

  {#if step === "select"}
    <ModalBody class="animate-fade-in space-y-3">
      {#if applied.length === 0}
        <EmptyState
          icon="mdi:information-outline"
          title="No applied tweaks"
          description="Apply some tweaks first, then export them as a profile."
        />
      {:else}
        <Card class="overflow-hidden">
          <SelectableRow checked={allSelected} label="Select all applied tweaks" onclick={toggleAll}>
            <span class="text-sm font-medium">Select all applied tweaks</span>
            {#snippet trailing()}<Badge>{plural(applied.length, "tweak")}</Badge>{/snippet}
          </SelectableRow>
        </Card>

        {#each groups as [categoryId, tweaks] (categoryId)}
          {@const selected = tweaks.filter((t) => !excludedIds.has(t.definition.id)).length}
          <SectionCard title={categoriesStore.name(categoryId)}>
            {#snippet actions()}<Badge class="mr-2">{selected}/{tweaks.length}</Badge>{/snippet}
            <div class="divide-y divide-border">
              {#each tweaks as tweak (tweak.definition.id)}
                <SelectableRow
                  checked={!excludedIds.has(tweak.definition.id)}
                  label={tweak.definition.name}
                  onclick={() => toggle(tweak.definition.id)}
                >
                  <span class="block truncate text-sm font-medium">{tweak.definition.name}</span>
                  {#snippet trailing()}
                    <Badge class="shrink-0">{tweak.status.activeOption ?? SYSTEM_DEFAULT_LABEL}</Badge>
                  {/snippet}
                </SelectableRow>
              {/each}
            </div>
          </SectionCard>
        {/each}
      {/if}
    </ModalBody>

    <ModalFooter>
      <Button variant="secondary" onclick={modalStore.close}>Cancel</Button>
      <Button variant="primary" disabled={selectedIds.length === 0} onclick={() => (step = "details")}>
        Continue
        <Icon icon="mdi:arrow-right" size="md" />
      </Button>
    </ModalFooter>
  {:else}
    <ModalBody class="animate-fade-in space-y-5">
      <Callout tone="success" density="panel" icon="mdi:check-circle">
        <span class="text-sm">{plural(selectedIds.length, "tweak")} selected</span>
      </Callout>

      <div class="space-y-1.5">
        <label for="{formId}-name" class="block text-sm font-medium">
          Profile name <span class="text-error" aria-hidden="true">*</span>
        </label>
        <TextField id="{formId}-name" bind:value={name} placeholder="For example, Gaming setup" required />
      </div>

      <div class="space-y-1.5">
        <label for="{formId}-description" class="block text-sm font-medium">Description (optional)</label>
        <TextArea id="{formId}-description" bind:value={description} placeholder="What this profile is for" />
      </div>

      <Card>
        <SettingRow
          title="Include baseline system state"
          description="Records the current registry, service and task settings, so an import on another PC can spot conflicts."
        >
          <Switch
            checked={includeSystemState}
            label="Include baseline system state"
            onchange={(on) => (includeSystemState = on)}
          />
        </SettingRow>
      </Card>

      <ProfileFileNote>
        stores which tweaks you picked and their options, not the changes themselves: those come from the app's built-in
        tweak definitions. Import it on another PC or after reinstalling Windows.
      </ProfileFileNote>
    </ModalBody>

    <ModalFooter>
      <Button variant="secondary" icon="mdi:arrow-left" onclick={() => (step = "select")}>Back</Button>
      <Button
        variant="primary"
        icon="mdi:export"
        disabled={!canExport}
        loading={profileStore.isExporting}
        onclick={exportProfile}
      >
        Export profile
      </Button>
    </ModalFooter>
  {/if}
</Modal>
