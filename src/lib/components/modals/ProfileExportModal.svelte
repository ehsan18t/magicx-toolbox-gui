<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import {
    Badge,
    Button,
    Checkbox,
    IconButton,
    Modal,
    ModalBody,
    ModalFooter,
    ModalHeader,
    Switch,
  } from "$lib/components/ui";
  import { closeModal, modalStore } from "$lib/stores/modal.svelte";
  import { profileStore } from "$lib/stores/profile.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { categoriesStore, tweaksStore } from "$lib/stores/tweaks.svelte";
  import { SvelteSet } from "svelte/reactivity";

  const isOpen = $derived(modalStore.current === "profileExport");

  // Wizard step state
  let step = $state<1 | 2>(1);

  // Step 1: Tweak selection - use const since we only mutate, never reassign
  const selectedTweakIds = new SvelteSet<string>();
  let selectAllApplied = $state(true);

  // Step 2: Profile details
  let profileName = $state("");
  let profileDescription = $state("");
  let includeSystemState = $state(false);

  // Get applied tweaks grouped by category
  const appliedTweaks = $derived(tweaksStore.list.filter((t) => t.status.is_applied));

  const tweaksByCategory = $derived.by(() => {
    const byCategory: Record<string, typeof appliedTweaks> = {};
    for (const tweak of appliedTweaks) {
      const catId = tweak.definition.category_id;
      if (!byCategory[catId]) byCategory[catId] = [];
      byCategory[catId].push(tweak);
    }
    return byCategory;
  });

  const selectedCount = $derived(selectedTweakIds.size);
  const canExport = $derived(selectedCount > 0 && profileName.trim().length > 0);

  // Helper to reset tweak selection to all applied tweaks
  function resetSelection() {
    selectedTweakIds.clear();
    for (const tweak of appliedTweaks) {
      selectedTweakIds.add(tweak.definition.id);
    }
  }

  // Reset state when modal opens
  $effect(() => {
    if (isOpen) {
      step = 1;
      resetSelection();
      selectAllApplied = true;
      profileName = "";
      profileDescription = "";
      includeSystemState = false;
    }
  });

  function handleSelectAll(checked: boolean) {
    selectAllApplied = checked;
    selectedTweakIds.clear();
    if (checked) {
      for (const tweak of appliedTweaks) {
        selectedTweakIds.add(tweak.definition.id);
      }
    }
  }

  function toggleTweak(tweakId: string) {
    if (selectedTweakIds.has(tweakId)) {
      selectedTweakIds.delete(tweakId);
    } else {
      selectedTweakIds.add(tweakId);
    }
    selectAllApplied = selectedTweakIds.size === appliedTweaks.length;
  }

  function handleBack() {
    step = 1;
  }

  function handleNext() {
    if (step === 1 && selectedCount > 0) {
      step = 2;
    }
  }

  async function handleExport() {
    const success = await profileStore.exportProfile(profileName.trim(), Array.from(selectedTweakIds), {
      description: profileDescription.trim() || undefined,
      includeSystemState,
    });

    if (success) {
      toastStore.show("success", `Profile "${profileName}" exported successfully`);
      closeModal();
    } else if (profileStore.exportError) {
      toastStore.show("error", profileStore.exportError);
    }
  }

  function getCategoryIcon(categoryId: string): string {
    return categoriesStore.getIcon(categoryId);
  }

  function getCategoryName(categoryId: string): string {
    return categoriesStore.getName(categoryId);
  }
</script>

<Modal open={isOpen} onclose={closeModal} size="lg" labelledBy="export-modal-title">
  <ModalHeader id="export-modal-title">
    <div class="flex items-center gap-3">
      <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-accent/15">
        <Icon icon="mdi:export" width="24" class="text-accent" />
      </div>
      <div>
        <h2 class="m-0 text-lg font-bold text-foreground">Export Profile</h2>
        <p class="m-0 text-sm text-foreground-muted">
          {#if step === 1}
            Select tweaks to include
          {:else}
            Enter profile details
          {/if}
        </p>
      </div>
    </div>
    <div class="flex items-center gap-2">
      <!-- Step indicator -->
      <div class="bg-muted flex items-center gap-1.5 rounded-full px-3 py-1.5">
        <span
          class="flex h-5 w-5 items-center justify-center rounded-full text-xs font-bold {step === 1
            ? 'bg-accent text-white'
            : 'bg-success text-white'}"
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
      <IconButton icon="mdi:close" onclick={closeModal} aria-label="Close" />
    </div>
  </ModalHeader>

  <ModalBody scrollable maxHeight="calc(100dvh - 14rem)">
    {#if step === 1}
      <!-- Step 1: Select Tweaks -->
      <div class="space-y-4">
        <!-- Select all toggle -->
        <button
          type="button"
          class="hover:bg-muted/30 flex w-full items-center justify-between rounded-lg border border-border bg-surface p-3 transition-colors focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-background focus-visible:outline-none"
          onclick={() => handleSelectAll(!selectAllApplied)}
          aria-label="Select all applied tweaks"
        >
          <div class="flex items-center gap-3">
            <Checkbox checked={selectAllApplied} ariaLabel="Select all" />
            <span class="font-medium text-foreground">Select All Applied Tweaks</span>
          </div>
          <Badge variant="default">{appliedTweaks.length} tweaks</Badge>
        </button>

        {#if appliedTweaks.length === 0}
          <div class="flex flex-col items-center justify-center gap-3 py-12 text-center">
            <Icon icon="mdi:information-outline" width="48" class="text-foreground-muted" />
            <p class="text-foreground-muted">No tweaks have been applied yet.</p>
            <p class="text-sm text-foreground-muted">Apply some tweaks first, then export them as a profile.</p>
          </div>
        {:else}
          <!-- Tweaks by category -->
          <div class="space-y-3">
            {#each Object.entries(tweaksByCategory) as [categoryId, categoryTweaks] (categoryId)}
              {@const categorySelected = categoryTweaks.filter((t) => selectedTweakIds.has(t.definition.id)).length}
              <div class="rounded-lg border border-border">
                <!-- Category header -->
                <div class="bg-muted/30 flex items-center gap-2 border-b border-border px-3 py-2">
                  <Icon icon={getCategoryIcon(categoryId)} width="18" class="text-accent" />
                  <span class="flex-1 text-sm font-semibold text-foreground">{getCategoryName(categoryId)}</span>
                  <span class="text-xs text-foreground-muted">{categorySelected}/{categoryTweaks.length}</span>
                </div>

                <!-- Tweaks list -->
                <div class="divide-y divide-border">
                  {#each categoryTweaks as tweak (tweak.definition.id)}
                    {@const isSelected = selectedTweakIds.has(tweak.definition.id)}
                    {@const currentOption = tweak.definition.options[tweak.status.current_option_index ?? 0]}
                    <button
                      type="button"
                      class="hover:bg-muted/50 focus-visible:bg-muted/50 flex w-full items-center gap-3 px-3 py-2.5 text-left transition-colors focus-visible:outline-none"
                      onclick={() => toggleTweak(tweak.definition.id)}
                      aria-label="Toggle {tweak.definition.name}"
                    >
                      <Checkbox checked={isSelected} ariaLabel="{tweak.definition.name} selection" />
                      <div class="min-w-0 flex-1">
                        <span class="block truncate text-sm font-medium text-foreground">{tweak.definition.name}</span>
                      </div>
                      <Badge variant="default" class="shrink-0">{currentOption?.label ?? "Applied"}</Badge>
                    </button>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {:else}
      <!-- Step 2: Profile Details -->
      <div class="space-y-5">
        <!-- Summary -->
        <div class="flex items-center gap-3 rounded-lg border border-border bg-success/10 p-3">
          <Icon icon="mdi:check-circle" width="20" class="text-success" />
          <span class="text-sm text-foreground">{selectedCount} tweaks selected for export</span>
        </div>

        <!-- Profile name -->
        <div class="space-y-2">
          <label for="profile-name" class="block text-sm font-medium text-foreground">
            Profile Name <span class="text-error">*</span>
          </label>
          <input
            id="profile-name"
            type="text"
            bind:value={profileName}
            placeholder="e.g., My Gaming Setup"
            class="w-full rounded-lg border border-border bg-surface px-3 py-2.5 text-sm text-foreground placeholder:text-foreground-muted focus:border-accent focus:ring-1 focus:ring-accent focus:outline-none"
          />
        </div>

        <!-- Description -->
        <div class="space-y-2">
          <label for="profile-desc" class="block text-sm font-medium text-foreground"> Description (optional) </label>
          <textarea
            id="profile-desc"
            bind:value={profileDescription}
            placeholder="Describe what this profile is for..."
            rows="3"
            class="w-full resize-none rounded-lg border border-border bg-surface px-3 py-2.5 text-sm text-foreground placeholder:text-foreground-muted focus:border-accent focus:ring-1 focus:ring-accent focus:outline-none"
          ></textarea>
        </div>

        <!-- System state toggle -->
        <div class="flex items-center justify-between rounded-lg border border-border bg-surface p-4">
          <div class="flex-1">
            <div class="flex items-center gap-2">
              <Icon icon="mdi:database" width="18" class="text-accent" />
              <span class="font-medium text-foreground">Include Baseline System State</span>
            </div>
            <p class="mt-1 text-sm text-foreground-muted">
              Records current system settings (registry, services, tasks) to detect conflicts when importing on another
              machine.
            </p>
          </div>
          <Switch checked={includeSystemState} onchange={(checked) => (includeSystemState = checked)} />
        </div>

        <!-- Info box -->
        <div class="flex items-start gap-3 rounded-lg border border-border/50 bg-surface/50 p-3">
          <Icon icon="mdi:information" width="18" class="mt-0.5 shrink-0 text-accent" />
          <p class="m-0 text-xs leading-relaxed text-foreground-muted">
            Profiles are saved as <code class="bg-muted rounded px-1">.mgx</code> files that can be imported on other machines
            or after reinstalling Windows. They only contain tweak IDs and settings—actual system changes come from the app's
            tweak definitions.
          </p>
        </div>
      </div>
    {/if}
  </ModalBody>

  <ModalFooter>
    {#if step === 1}
      <Button variant="secondary" onclick={closeModal}>Cancel</Button>
      <Button variant="primary" onclick={handleNext} disabled={selectedCount === 0}>
        Continue
        <Icon icon="mdi:arrow-right" width="18" />
      </Button>
    {:else}
      <Button variant="secondary" onclick={handleBack}>
        <Icon icon="mdi:arrow-left" width="18" />
        Back
      </Button>
      <Button variant="primary" onclick={handleExport} disabled={!canExport} loading={profileStore.isExporting}>
        <Icon icon="mdi:export" width="18" />
        Export Profile
      </Button>
    {/if}
  </ModalFooter>
</Modal>
