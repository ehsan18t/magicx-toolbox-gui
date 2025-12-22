<script lang="ts">
  import { ConfirmDialog } from "$lib/components/modals";
  import { Icon } from "$lib/components/shared";
  import { ActionButton, Badge, Button, EmptyState } from "$lib/components/ui";
  import { modalStore } from "$lib/stores/modal.svelte";
  import { profileStore } from "$lib/stores/profile.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { appDataDir, join } from "@tauri-apps/api/path";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";

  // Use derived state from store
  const profiles = $derived(profileStore.savedProfiles);
  const isLoading = $derived(profileStore.loadingSavedProfiles);
  const currentProfileDir = $derived(profileStore.currentProfileDir);
  let profileToDelete = $state<string | null>(null);
  let deletingProfile = $state<string | null>(null);

  async function handleDelete(name: string) {
    if (!name || deletingProfile) return;
    deletingProfile = name;

    const success = await profileStore.deleteProfile(name);
    profileToDelete = null;

    if (success) {
      toastStore.show("success", `Profile "${name}" deleted`);
    } else {
      toastStore.show("error", profileStore.deleteError ?? "Failed to delete profile");
    }

    deletingProfile = null;
  }

  async function handleApplySaved(name: string) {
    try {
      let profilesDir: string;

      if (currentProfileDir) {
        profilesDir = currentProfileDir;
      } else {
        const appData = await appDataDir();
        profilesDir = await join(appData, "profiles");
      }

      const safeName = name.replace(/[^a-z0-9\-_]/gi, "");
      const path = await join(profilesDir, `${safeName}.mgx`);

      const success = await profileStore.importProfileFromPath(path);
      if (success) {
        modalStore.open("profileImport");
      } else if (profileStore.importError) {
        toastStore.show("error", profileStore.importError);
      }
    } catch (e) {
      console.error("Failed to prepare import:", e);
      toastStore.show("error", "Failed to load profile for import");
    }
  }

  async function handleOpenFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Profile Folder",
      });

      if (selected && typeof selected === "string") {
        profileStore.setProfileDir(selected);
        toastStore.success(`Loaded profiles from: ${selected}`);
      }
    } catch (e) {
      console.error("Failed to open folder:", e);
      toastStore.error("Failed to open folder dialog");
    }
  }

  function handleResetFolder() {
    profileStore.setProfileDir(null);
    toastStore.info("Reset to default profile directory");
  }

  // Drag state
  let isDragOver = $state(false);

  async function handleDroppedFile(path: string) {
    if (!path.endsWith(".mgx")) {
      toastStore.error("Invalid file type. Please select a .mgx profile file.");
      return;
    }

    const success = await profileStore.importProfileFromPath(path);
    if (success) {
      modalStore.open("profileImport");
    } else if (profileStore.importError) {
      toastStore.error(profileStore.importError);
    }
  }

  onMount(() => {
    profileStore.loadSavedProfiles();

    // Set up native drag-drop listener
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === "over") {
          isDragOver = true;
        } else if (event.payload.type === "drop") {
          isDragOver = false;
          const paths = event.payload.paths;
          if (paths && paths.length > 0) {
            handleDroppedFile(paths[0]);
          }
        } else {
          // cancelled
          isDragOver = false;
        }
      })
      .then((fn) => {
        // If cleanup was called before promise resolved, immediately clean up
        if (cancelled) {
          fn();
        } else {
          unlisten = fn;
        }
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  });
</script>

<div class="flex h-full flex-col gap-5 overflow-hidden p-6">
  <!-- Header -->
  <header class="flex flex-wrap items-center justify-between gap-6">
    <div class="flex items-center gap-4">
      <div class="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-accent/15 text-accent">
        <Icon icon="mdi:file-multiple" width="28" />
      </div>
      <div>
        <h1 class="m-0 text-2xl font-bold tracking-tight text-foreground">Profile Library</h1>
        <p class="mt-1 mb-0 text-sm text-foreground-muted">
          {#if currentProfileDir}
            <span class="flex items-center gap-1.5" title={currentProfileDir}>
              <Icon icon="mdi:folder-open" width="14" />
              Custom Folder: <span class="font-mono text-xs">{currentProfileDir}</span>
            </span>
          {:else}
            Manage your saved configuration profiles
          {/if}
        </p>
      </div>
    </div>

    <div class="flex items-center gap-4 rounded-xl border border-border bg-card px-5 py-3">
      <div class="flex items-center gap-2.5">
        <div class="flex h-9 w-9 items-center justify-center rounded-full bg-accent/15">
          <Icon icon="mdi:folder" width="18" class="text-accent" />
        </div>
        <div class="flex flex-col items-center justify-center gap-0.5">
          <span class="text-base font-bold text-foreground">{profiles.length}</span>
          <span class="text-xs text-foreground-muted">Saved Profiles</span>
        </div>
      </div>
    </div>
  </header>

  <!-- Toolbar -->
  <div class="flex flex-wrap items-center gap-3">
    <div class="flex-1"></div>
    {#if currentProfileDir}
      <ActionButton
        intent="default"
        icon="mdi:refresh"
        onclick={handleResetFolder}
        tooltip="Reset to default AppData folder"
      >
        Reset Default
      </ActionButton>
    {/if}
    <ActionButton
      intent="default"
      icon="mdi:folder-open"
      onclick={handleOpenFolder}
      tooltip="Select a folder to view profiles"
    >
      Open Folder
    </ActionButton>
    <ActionButton intent="accent" icon="mdi:plus" onclick={() => modalStore.open("profileExport")}>
      New Profile
    </ActionButton>
  </div>

  <!-- Profiles List -->
  <div class="-mr-2 min-h-0 flex-1 overflow-y-auto pr-2">
    {#if isLoading}
      <div class="space-y-3">
        <div class="animate-pulse bg-muted/50 h-24 w-full rounded-lg"></div>
        <div class="animate-pulse bg-muted/50 h-24 w-full rounded-lg"></div>
      </div>
    {:else if profiles.length === 0}
      <EmptyState
        icon="mdi:folder-outline"
        title="No Saved Profiles"
        description="Profiles you export can be saved here for quick access."
        actionText="Create Profile"
        onaction={() => modalStore.open("profileExport")}
      />
    {:else}
      <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {#each profiles as profile (profile.name + profile.created_at)}
          <div
            class="group relative flex flex-col justify-between gap-4 rounded-xl border border-border bg-surface p-5 transition-all hover:border-accent hover:shadow-sm"
          >
            <div>
              <div class="flex items-start justify-between gap-2">
                <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-accent/10 text-accent">
                  <Icon icon="mdi:file-cog" width="20" />
                </div>
                <Badge class="text-xs">v{profile.app_version}</Badge>
              </div>
              <h3 class="mt-3 mb-1 text-base font-semibold text-foreground transition-colors group-hover:text-accent">
                {profile.name}
              </h3>
              <p class="line-clamp-2 min-h-[2.5em] text-sm text-foreground-muted">
                {profile.description || "No description"}
              </p>
            </div>

            <div class="flex items-center justify-between border-t border-border/50 pt-4">
              <div class="flex flex-col text-xs text-foreground-muted">
                <span class="font-medium">Win {profile.source_windows_version}</span>
                <span>{new Date(profile.created_at).toLocaleDateString()}</span>
              </div>
              <div class="flex gap-1">
                <Button
                  size="sm"
                  variant="secondary"
                  class="h-8 px-2"
                  onclick={() => (profileToDelete = profile.name)}
                  disabled={deletingProfile === profile.name}
                >
                  {#if deletingProfile === profile.name}
                    <Icon icon="mdi:loading" width="16" class="animate-spin" />
                  {:else}
                    <Icon icon="mdi:delete" width="16" />
                  {/if}
                </Button>
                <Button size="sm" variant="primary" class="h-8 px-3" onclick={() => handleApplySaved(profile.name)}>
                  <Icon icon="mdi:play" width="16" class="mr-1.5" />
                  Apply
                </Button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Drag Overlay -->
  {#if isDragOver}
    <div
      class="absolute inset-0 z-50 flex flex-col items-center justify-center bg-background/80 backdrop-blur-sm transition-all"
    >
      <div class="animate-bounce flex h-32 w-32 items-center justify-center rounded-3xl bg-accent/20">
        <Icon icon="mdi:file-import" width="64" class="text-accent" />
      </div>
      <h2 class="mt-8 text-2xl font-bold tracking-tight text-foreground">Drop to Import Profile</h2>
      <p class="mt-2 text-lg text-foreground-muted">Release the file to start importing</p>
    </div>
  {/if}
</div>

<ConfirmDialog
  open={!!profileToDelete}
  title="Delete Profile"
  message="Are you sure you want to delete '{profileToDelete}'? This action cannot be undone."
  confirmText="Delete"
  variant="danger"
  onconfirm={() => profileToDelete && handleDelete(profileToDelete)}
  oncancel={() => (profileToDelete = null)}
/>
