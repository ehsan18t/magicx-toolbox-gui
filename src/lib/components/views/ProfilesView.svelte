<script lang="ts" module>
  const SKELETON_CARDS = 2;
</script>

<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { DROP_VEIL, HEADING, TONE_WASH } from "$lib/design";
  import { PageLayout, PageStats } from "$lib/components/layout";
  import { Icon } from "$lib/components/shared";
  import { Badge, Button, Callout, card, EmptyState, IconButton, IconTile, Skeleton } from "$lib/components/ui";
  import { PROFILE_EXT } from "$lib/config/app";
  import { confirmStore } from "$lib/stores/confirm.svelte";
  import { modalStore } from "$lib/stores/modal.svelte";
  import { PROFILE_FILE_REJECTED, profileStore } from "$lib/stores/profile.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { listenFileDrop } from "$lib/utils/fileDrop";
  import { SEP } from "$lib/utils/format";
  import { fade, pop, reflow } from "$lib/utils/motion";
  import { formatDate } from "$lib/utils/time";
  import { onMount } from "svelte";

  const profiles = $derived(profileStore.savedProfiles);
  const currentProfileDir = $derived(profileStore.profileDir);
  let deletingProfile = $state<string | null>(null);
  let isDragOver = $state(false);

  async function handleDelete(name: string) {
    if (deletingProfile) return;
    const confirmed = await confirmStore.ask({
      title: `Delete ${name}?`,
      message: "The profile file is removed from this PC. This cannot be undone.",
      confirmText: "Delete profile",
      variant: "danger",
    });
    if (!confirmed || deletingProfile) return;

    deletingProfile = name;
    if (await profileStore.deleteProfile(name)) toastStore.success(`Profile "${name}" deleted`);
    else toastStore.error(profileStore.deleteError ?? "Could not delete the profile");
    deletingProfile = null;
  }

  async function openImport(importing: Promise<boolean>) {
    if (await importing) modalStore.open("profileImport");
    else if (profileStore.importError) toastStore.error(profileStore.importError);
  }

  async function handleOpenFolder() {
    const dir = await profileStore.chooseFolder();
    if (dir) toastStore.success(`Showing profiles from ${dir}`);
  }

  function handleResetFolder() {
    profileStore.setProfileDir(null);
    toastStore.info("Showing profiles from the default folder");
  }

  // The import modal owns drops while it is open.
  const importModalOpen = () => modalStore.current === "profileImport";

  onMount(() =>
    listenFileDrop({
      extension: `.${PROFILE_EXT}`,
      onOver: () => (isDragOver = !importModalOpen()),
      onLeave: () => (isDragOver = false),
      onDrop: (path) => {
        if (!importModalOpen()) void openImport(profileStore.importProfileFromPath(path));
      },
      onReject: () => {
        if (!importModalOpen()) toastStore.error(PROFILE_FILE_REJECTED);
      },
    }),
  );
</script>

<div class="relative h-full">
  <PageLayout title="Profiles" description="Saved configuration profiles you can apply on this or another PC.">
    {#snippet aside()}
      <PageStats items={[{ value: profiles.length, label: "saved" }]} />
      {#if currentProfileDir}
        <p
          class="m-0 flex min-w-0 flex-1 items-center gap-1.5 text-xs text-foreground-muted"
          use:tooltip={currentProfileDir}
        >
          <Icon icon="mdi:folder-open" size="xs" class="shrink-0" />
          <span class="truncate font-mono">{currentProfileDir}</span>
        </p>
      {/if}
      <div class="ml-auto flex flex-wrap gap-2">
        {#if currentProfileDir}
          <Button icon="mdi:refresh" tooltip="Reset to the default AppData folder" onclick={handleResetFolder}>
            Reset folder
          </Button>
        {/if}
        <Button icon="mdi:folder-open" tooltip="Select a folder to view profiles" onclick={handleOpenFolder}>
          Open folder
        </Button>
        <Button variant="primary" icon="mdi:plus" tooltip="Unavailable while profiles are rebuilt" disabled>
          New profile
        </Button>
      </div>
    {/snippet}

    <Callout tone="warning" icon="mdi:hammer-wrench" role="status">
      <p class="m-0 text-ui">
        <span class="font-semibold">Profiles are being rebuilt.</span>
        <span class="text-foreground-muted">
          Exporting, importing and applying profiles are unavailable for now. Your tweaks and snapshots are unaffected.
        </span>
      </p>
    </Callout>

    {#if profileStore.isLoadingSavedProfiles}
      <div class="space-y-2">
        {#each { length: SKELETON_CARDS }, i (i)}
          <Skeleton class="h-24 rounded-lg" />
        {/each}
      </div>
    {:else if profileStore.savedProfilesError}
      <Callout tone="error" icon="mdi:alert-circle" role="alert">
        <p class="m-0 text-ui">{profileStore.savedProfilesError}</p>
      </Callout>
    {:else if profiles.length === 0}
      <EmptyState
        icon="mdi:folder-outline"
        title="No saved profiles"
        description="Profiles you export are kept here for quick access."
      />
    {:else}
      <h2 class="sr-only">Saved profiles</h2>
      <div class="grid animate-fade-in grid-cols-cards gap-2">
        <!-- The card() look, not Card: animate: needs an element as the each block's only child. -->
        {#each profiles as profile (profile.name + profile.created_at)}
          <div
            class={card({ class: "flex min-w-0 flex-col gap-3 p-3.5 transition-colors hover:border-border-hover" })}
            out:pop
            animate:reflow
          >
            <div class="flex items-start gap-2.5">
              <IconTile icon="mdi:file-cog" size="sm" />
              <div class="min-w-0 flex-1">
                <h3 class={["m-0 wrap-break-word", HEADING.item]}>{profile.name}</h3>
                <p class="m-0 mt-0.5 line-clamp-2 text-xs text-foreground-muted">
                  {profile.description || "No description"}
                </p>
              </div>
              <Badge class="shrink-0">v{profile.app_version}</Badge>
            </div>
            <div class="mt-auto flex flex-wrap items-center justify-between gap-2 border-t border-border pt-3">
              <span class="text-xs text-foreground-muted">
                Windows {profile.source_windows_version}{SEP}{formatDate(profile.created_at)}
              </span>
              <div class="flex gap-1.5">
                <IconButton
                  icon="mdi:delete"
                  size="sm"
                  tooltip="Delete {profile.name}"
                  class={TONE_WASH.error}
                  loading={deletingProfile === profile.name}
                  onclick={() => handleDelete(profile.name)}
                />
                <Button
                  size="sm"
                  variant="primary"
                  icon="mdi:play"
                  onclick={() => openImport(profileStore.importSaved(profile.name))}
                >
                  Apply
                </Button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </PageLayout>

  {#if isDragOver}
    <div
      class="absolute inset-0 z-scrim flex flex-col items-center justify-center {DROP_VEIL}"
      transition:fade={{ speed: "fast" }}
    >
      <IconTile icon="mdi:file-import" size="4xl" />
      <h2 class={["mt-6", HEADING.prompt]}>Drop to import profile</h2>
      <p class="mt-1 text-sm text-foreground-muted">Release the file to start importing</p>
    </div>
  {/if}
</div>
