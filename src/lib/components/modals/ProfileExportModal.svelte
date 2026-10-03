<script lang="ts" module>
  const FIELD =
    "w-full rounded-lg border border-border bg-surface px-3 py-2.5 text-sm text-foreground placeholder:text-foreground-muted focus:border-accent focus:ring-1 focus:ring-accent focus:outline-none";
</script>

<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import {
    Badge,
    Button,
    Callout,
    Checkbox,
    ICON_SIZE,
    Modal,
    ModalBody,
    ModalFooter,
    Switch,
  } from "$lib/components/ui";
  import { PROFILE_EXT } from "$lib/config/app";
  import { modalStore } from "$lib/stores/modal.svelte";
  import { profileStore } from "$lib/stores/profile.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { categoriesStore, tweaksStore } from "$lib/stores/tweaksData.svelte";
  import { plural } from "$lib/utils/format";
  import { untrack } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import WizardHeader from "./WizardHeader.svelte";

  const isOpen = $derived(modalStore.current === "profileExport");

  let step = $state<1 | 2>(1);
  // Exclusions over the live applied list, so a tweak reported applied after open is exported too.
  const excludedIds = new SvelteSet<string>();
  let profileName = $state("");
  let profileDescription = $state("");
  let includeSystemState = $state(false);

  const appliedTweaks = $derived(tweaksStore.list.filter((t) => t.status.state === "active"));
  const selectedIds = $derived(appliedTweaks.map((t) => t.definition.id).filter((id) => !excludedIds.has(id)));
  const selectAllApplied = $derived(selectedIds.length === appliedTweaks.length);
  const canExport = $derived(selectedIds.length > 0 && profileName.trim().length > 0);

  const tweaksByCategory = $derived.by(() => {
    const byCategory: Record<string, typeof appliedTweaks> = {};
    for (const tweak of appliedTweaks) (byCategory[tweak.definition.categoryId] ??= []).push(tweak);
    return byCategory;
  });

  // Open edge only: tracking the form fields here would wipe them as they are typed.
  $effect(() => {
    if (!isOpen) return;
    untrack(() => {
      step = 1;
      excludedIds.clear();
      profileName = "";
      profileDescription = "";
      includeSystemState = false;
    });
  });

  function toggleAll() {
    if (selectAllApplied) for (const tweak of appliedTweaks) excludedIds.add(tweak.definition.id);
    else excludedIds.clear();
  }

  function toggleTweak(id: string) {
    if (!excludedIds.delete(id)) excludedIds.add(id);
  }

  async function handleExport() {
    const name = profileName.trim();
    const success = await profileStore.exportProfile(name, selectedIds, {
      description: profileDescription.trim() || undefined,
      includeSystemState,
    });

    if (success) {
      toastStore.success(`Profile "${name}" exported successfully`);
      modalStore.close();
    } else if (profileStore.exportError) {
      toastStore.error(profileStore.exportError);
    }
  }
</script>

<Modal open={isOpen} onclose={modalStore.close} size="lg">
  <WizardHeader
    title="Export Profile"
    icon="mdi:export"
    subtitle={step === 1 ? "Select tweaks to include" : "Enter profile details"}
    onclose={modalStore.close}
  >
    {#snippet actions()}
      <div class="flex items-center gap-1.5 rounded-full bg-muted px-3 py-1.5">
        <span
          class="flex h-5 w-5 items-center justify-center rounded-full text-xs font-bold text-white {step === 1
            ? 'bg-accent'
            : 'bg-success'}"
        >
          {step === 1 ? "1" : "✓"}
        </span>
        <div class="h-0.5 w-4 {step === 2 ? 'bg-accent' : 'bg-border'}"></div>
        <span
          class="flex h-5 w-5 items-center justify-center rounded-full text-xs font-bold {step === 2
            ? 'bg-accent text-white'
            : 'bg-muted text-foreground-muted'}"
        >
          2
        </span>
      </div>
    {/snippet}
  </WizardHeader>

  <ModalBody>
    {#if step === 1}
      <div class="animate-fade-in space-y-4">
        <button
          type="button"
          class="flex w-full items-center justify-between rounded-lg border border-border bg-surface p-3 transition-colors hover:bg-muted/30 focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-background focus-visible:outline-none"
          onclick={toggleAll}
          aria-label="Select all applied tweaks"
        >
          <div class="flex items-center gap-3">
            <Checkbox checked={selectAllApplied} />
            <span class="font-medium text-foreground">Select All Applied Tweaks</span>
          </div>
          <Badge>{plural(appliedTweaks.length, "tweak")}</Badge>
        </button>

        {#if appliedTweaks.length === 0}
          <div class="flex flex-col items-center justify-center gap-3 py-12 text-center">
            <Icon icon="mdi:information-outline" width="48" class="text-foreground-muted" />
            <p class="text-foreground-muted">No tweaks have been applied yet.</p>
            <p class="text-sm text-foreground-muted">Apply some tweaks first, then export them as a profile.</p>
          </div>
        {:else}
          <div class="space-y-3">
            {#each Object.entries(tweaksByCategory) as [categoryId, categoryTweaks] (categoryId)}
              {@const categorySelected = categoryTweaks.filter((t) => !excludedIds.has(t.definition.id)).length}
              <div class="rounded-lg border border-border">
                <div class="flex items-center gap-2 border-b border-border bg-muted/30 px-3 py-2">
                  <Icon icon={categoriesStore.icon(categoryId)} width={ICON_SIZE.lg} class="text-accent" />
                  <span class="flex-1 text-sm font-semibold text-foreground">{categoriesStore.name(categoryId)}</span>
                  <span class="text-xs text-foreground-muted">{categorySelected}/{categoryTweaks.length}</span>
                </div>

                <div class="divide-y divide-border">
                  {#each categoryTweaks as tweak (tweak.definition.id)}
                    <button
                      type="button"
                      class="flex w-full items-center gap-3 px-3 py-2.5 text-left transition-colors hover:bg-muted/50 focus-visible:bg-muted/50 focus-visible:outline-none"
                      onclick={() => toggleTweak(tweak.definition.id)}
                      aria-label="Toggle {tweak.definition.name}"
                    >
                      <Checkbox checked={!excludedIds.has(tweak.definition.id)} />
                      <div class="min-w-0 flex-1">
                        <span class="block truncate text-sm font-medium text-foreground">{tweak.definition.name}</span>
                      </div>
                      <Badge class="shrink-0">{tweak.status.activeOption ?? "System Default"}</Badge>
                    </button>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {:else}
      <div class="animate-fade-in space-y-5">
        <Callout tone="success" density="panel" icon="mdi:check-circle">
          <span class="text-sm text-foreground">{plural(selectedIds.length, "tweak")} selected for export</span>
        </Callout>

        <div class="space-y-2">
          <label for="profile-name" class="block text-sm font-medium text-foreground">
            Profile Name <span class="text-error">*</span>
          </label>
          <input
            id="profile-name"
            type="text"
            bind:value={profileName}
            placeholder="e.g., My Gaming Setup"
            class={FIELD}
          />
        </div>

        <div class="space-y-2">
          <label for="profile-desc" class="block text-sm font-medium text-foreground">Description (optional)</label>
          <textarea
            id="profile-desc"
            bind:value={profileDescription}
            placeholder="Describe what this profile is for…"
            rows="3"
            class="{FIELD} resize-none"></textarea>
        </div>

        <div class="flex items-center justify-between rounded-lg border border-border bg-surface p-4">
          <div class="flex-1">
            <div class="flex items-center gap-2">
              <Icon icon="mdi:database" width={ICON_SIZE.lg} class="text-accent" />
              <span class="font-medium text-foreground">Include Baseline System State</span>
            </div>
            <p class="mt-1 text-sm text-foreground-muted">
              Records current system settings (registry, services, tasks) to detect conflicts when importing on another
              machine.
            </p>
          </div>
          <Switch
            checked={includeSystemState}
            label="Include baseline system state"
            onchange={(checked) => (includeSystemState = checked)}
          />
        </div>

        <Callout tone="neutral" density="panel" icon="mdi:information">
          <p class="m-0 text-xs leading-relaxed text-foreground-muted">
            Profiles are saved as <code class="rounded bg-muted px-1">.{PROFILE_EXT}</code> files that can be imported on
            other machines or after reinstalling Windows. They only store which tweaks you picked and the settings you chose.
            The actual changes to your system come from the app's built-in tweak definitions.
          </p>
        </Callout>
      </div>
    {/if}
  </ModalBody>

  <ModalFooter>
    {#if step === 1}
      <Button variant="secondary" onclick={modalStore.close}>Cancel</Button>
      <Button variant="primary" onclick={() => (step = 2)} disabled={selectedIds.length === 0}>
        Continue
        <Icon icon="mdi:arrow-right" width={ICON_SIZE.lg} />
      </Button>
    {:else}
      <Button variant="secondary" onclick={() => (step = 1)}>
        <Icon icon="mdi:arrow-left" width={ICON_SIZE.lg} />
        Back
      </Button>
      <Button variant="primary" onclick={handleExport} disabled={!canExport} loading={profileStore.isExporting}>
        <Icon icon="mdi:export" width={ICON_SIZE.lg} />
        Export Profile
      </Button>
    {/if}
  </ModalFooter>
</Modal>
