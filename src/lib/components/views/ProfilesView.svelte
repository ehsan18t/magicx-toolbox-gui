<script lang="ts" module>
  const SKELETON_CARDS = 2;
</script>

<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { type IconName, TONE_SOFT } from "$lib/design";
  import { PageLayout, PageStats } from "$lib/components/layout";
  import { Icon } from "$lib/components/shared";
  import { Badge, Button, Callout, EmptyState } from "$lib/components/ui";
  import type { ButtonVariants } from "$lib/components/ui/variants";
  import { PROFILE_EXT } from "$lib/config/app";
  import { confirmStore } from "$lib/stores/confirm.svelte";
  import { modalStore } from "$lib/stores/modal.svelte";
  import { profileStore } from "$lib/stores/profile.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { listenFileDrop } from "$lib/utils/fileDrop";
  import { formatDate } from "$lib/utils/time";
  import { logError } from "$lib/utils/logger";
  import { fade, pop, reflow } from "$lib/utils/motion";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";

  const profiles = $derived(profileStore.savedProfiles);
  const currentProfileDir = $derived(profileStore.profileDir);
  let deletingProfile = $state<string | null>(null);
  let isDragOver = $state(false);

  async function handleDelete(name: string) {
    if (deletingProfile) return;
    const confirmed = await confirmStore.ask({
      title: "Delete Profile",
      message: `Are you sure you want to delete '${name}'? This action cannot be undone.`,
      confirmText: "Delete",
      variant: "danger",
    });
    if (!confirmed || deletingProfile) return;

    deletingProfile = name;
    if (await profileStore.deleteProfile(name)) toastStore.success(`Profile "${name}" deleted`);
    else toastStore.error(profileStore.deleteError ?? "Failed to delete profile");
    deletingProfile = null;
  }

  async function openImport(importing: Promise<boolean>) {
    if (await importing) modalStore.open("profileImport");
    else if (profileStore.importError) toastStore.error(profileStore.importError);
  }

  async function handleOpenFolder() {
    try {
      const selected = await open({ directory: true, multiple: false, title: "Select Profile Folder" });
      if (typeof selected === "string") {
        profileStore.setProfileDir(selected);
        toastStore.success(`Loaded profiles from: ${selected}`);
      }
    } catch (error) {
      logError("Failed to open folder", error);
      toastStore.error("Failed to open folder dialog");
    }
  }

  function handleResetFolder() {
    profileStore.setProfileDir(null);
    toastStore.info("Reset to default profile directory");
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
        if (!importModalOpen()) toastStore.error(`Invalid file type. Please select a .${PROFILE_EXT} profile file.`);
      },
    }),
  );
</script>

{#snippet toolbarButton(
  icon: IconName,
  label: string,
  tip: string,
  variant: ButtonVariants["variant"],
  onclick?: () => void,
)}
  <span class="inline-flex" use:tooltip={tip}>
    <Button {variant} {onclick} disabled={!onclick}>
      <Icon {icon} size="md" />
      {label}
    </Button>
  </span>
{/snippet}

<div class="relative h-full">
  <PageLayout title="Profiles" description="Saved configuration profiles you can apply on this or another PC.">
    {#snippet aside()}
      <PageStats items={[{ value: profiles.length, label: "saved" }]} />
    {/snippet}

    {#snippet toolbar()}
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
          {@render toolbarButton(
            "mdi:refresh",
            "Reset folder",
            "Reset to the default AppData folder",
            "secondary",
            handleResetFolder,
          )}
        {/if}
        {@render toolbarButton(
          "mdi:folder-open",
          "Open folder",
          "Select a folder to view profiles",
          "secondary",
          handleOpenFolder,
        )}
        {@render toolbarButton("mdi:plus", "New profile", "Unavailable while profiles are rebuilt", "primary")}
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
          <div class="h-24 animate-pulse rounded-lg bg-muted"></div>
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
      <div class="grid animate-fade-in grid-cols-cards gap-2">
        {#each profiles as profile (profile.name + profile.created_at)}
          {@const deleting = deletingProfile === profile.name}
          <div
            class="flex min-w-0 flex-col gap-3 rounded-lg border border-border bg-card p-3.5 transition-colors hover:border-border-hover"
            out:pop
            animate:reflow
          >
            <div class="flex items-start gap-2.5">
              <span class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md {TONE_SOFT.accent}">
                <Icon icon="mdi:file-cog" size="lg" />
              </span>
              <div class="min-w-0 flex-1">
                <h3 class="m-0 text-sm font-semibold wrap-break-word">{profile.name}</h3>
                <p class="m-0 mt-0.5 line-clamp-2 text-xs text-foreground-muted">
                  {profile.description || "No description"}
                </p>
              </div>
              <Badge class="shrink-0">v{profile.app_version}</Badge>
            </div>
            <div class="mt-auto flex flex-wrap items-center justify-between gap-2 border-t border-border pt-3">
              <span class="text-xs text-foreground-muted">
                Windows {profile.source_windows_version} · {formatDate(profile.created_at)}
              </span>
              <div class="flex gap-1.5">
                <Button
                  size="sm"
                  variant="secondary"
                  aria-label="Delete {profile.name}"
                  onclick={() => handleDelete(profile.name)}
                  loading={deleting}
                >
                  {#if !deleting}<Icon icon="mdi:delete" size="md" />{/if}
                </Button>
                <Button size="sm" variant="primary" onclick={() => openImport(profileStore.importSaved(profile.name))}>
                  <Icon icon="mdi:play" size="md" />
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
      class="absolute inset-0 z-scrim flex flex-col items-center justify-center bg-background/85"
      transition:fade={{ speed: "fast" }}
    >
      <div class="flex h-24 w-24 items-center justify-center rounded-xl {TONE_SOFT.accent}">
        <Icon icon="mdi:file-import" size="7xl" />
      </div>
      <h2 class="mt-6 text-xl font-semibold">Drop to import profile</h2>
      <p class="mt-1 text-sm text-foreground-muted">Release the file to start importing</p>
    </div>
  {/if}
</div>
