<script lang="ts">
  import { ConfirmDialog } from "$lib/components/modals";
  import { Icon } from "$lib/components/shared";
  import { PageLayout } from "$lib/components/layout";
  import { Badge, Button, EmptyState } from "$lib/components/ui";
  import { modalStore } from "$lib/stores/modal.svelte";
  import { profileStore } from "$lib/stores/profile.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { fade, pop, reflow } from "$lib/utils/motion";
  import { appDataDir, join } from "@tauri-apps/api/path";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";

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
    // No loadSavedProfiles(): it errors until the profile backend returns (docs/spec/profile-v1.md).
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
          isDragOver = false;
        }
      })
      .then((fn) => {
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

<div class="relative h-full">
  <PageLayout title="Profiles" description="Saved configuration profiles you can apply on this or another PC.">
    {#snippet aside()}
      <p class="m-0 text-xs text-foreground-muted">
        <span class="font-semibold text-foreground tabular-nums">{profiles.length}</span> saved
      </p>
    {/snippet}

    {#snippet toolbar()}
      {#if currentProfileDir}
        <p class="m-0 flex min-w-0 flex-1 items-center gap-1.5 text-xs text-foreground-muted" title={currentProfileDir}>
          <Icon icon="mdi:folder-open" width="14" class="shrink-0" />
          <span class="truncate font-mono">{currentProfileDir}</span>
        </p>
      {/if}
      <div class="ml-auto flex flex-wrap gap-2">
        {#if currentProfileDir}
          <Button variant="secondary" onclick={handleResetFolder} title="Reset to the default AppData folder">
            <Icon icon="mdi:refresh" width="16" />
            Reset folder
          </Button>
        {/if}
        <Button variant="secondary" onclick={handleOpenFolder} title="Select a folder to view profiles">
          <Icon icon="mdi:folder-open" width="16" />
          Open folder
        </Button>
        <Button variant="primary" disabled title="Unavailable while profiles are rebuilt">
          <Icon icon="mdi:plus" width="16" />
          New profile
        </Button>
      </div>
    {/snippet}

    <!-- The profile backend is being rebuilt; this notice goes when it returns (docs/spec/profile-v1.md). -->
    <div class="flex items-start gap-3 rounded-lg border border-warning/30 bg-warning/8 px-3 py-2.5" role="status">
      <Icon icon="mdi:hammer-wrench" width="18" class="mt-0.5 shrink-0 text-warning" />
      <p class="m-0 text-[13px]">
        <span class="font-semibold">Profiles are being rebuilt.</span>
        <span class="text-foreground-muted">
          Exporting, importing and applying profiles are unavailable for now. Your tweaks and snapshots are unaffected.
        </span>
      </p>
    </div>

    {#if isLoading}
      <div class="space-y-2">
        <div class="h-24 animate-pulse rounded-lg bg-muted"></div>
        <div class="h-24 animate-pulse rounded-lg bg-muted"></div>
      </div>
    {:else if profiles.length === 0}
      <EmptyState
        icon="mdi:folder-outline"
        title="No saved profiles"
        description="Profiles you export are kept here for quick access."
      />
    {:else}
      <div class="grid animate-fade-in grid-cols-[repeat(auto-fill,minmax(260px,1fr))] gap-2">
        {#each profiles as profile (profile.name + profile.created_at)}
          <div
            class="flex min-w-0 flex-col gap-3 rounded-lg border border-border bg-card p-3.5 transition-colors hover:border-border-hover"
            out:pop
            animate:reflow
          >
            <div class="flex items-start gap-2.5">
              <span class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-accent/12 text-accent">
                <Icon icon="mdi:file-cog" width="18" />
              </span>
              <div class="min-w-0 flex-1">
                <h3 class="m-0 text-sm font-semibold wrap-break-word">{profile.name}</h3>
                <p class="m-0 mt-0.5 line-clamp-2 text-xs text-foreground-muted">
                  {profile.description || "No description"}
                </p>
              </div>
              <Badge class="shrink-0 text-xs">v{profile.app_version}</Badge>
            </div>
            <div class="mt-auto flex flex-wrap items-center justify-between gap-2 border-t border-border pt-3">
              <span class="text-xs text-foreground-muted">
                Windows {profile.source_windows_version} · {new Date(profile.created_at).toLocaleDateString()}
              </span>
              <div class="flex gap-1.5">
                <Button
                  size="sm"
                  variant="secondary"
                  aria-label="Delete {profile.name}"
                  onclick={() => (profileToDelete = profile.name)}
                  disabled={deletingProfile === profile.name}
                >
                  <Icon
                    icon={deletingProfile === profile.name ? "mdi:loading" : "mdi:delete"}
                    width="16"
                    class={deletingProfile === profile.name ? "animate-spin" : ""}
                  />
                </Button>
                <Button size="sm" variant="primary" onclick={() => handleApplySaved(profile.name)}>
                  <Icon icon="mdi:play" width="16" />
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
      <div class="flex h-24 w-24 items-center justify-center rounded-2xl bg-accent/15">
        <Icon icon="mdi:file-import" width="48" class="text-accent" />
      </div>
      <h2 class="mt-6 text-xl font-semibold">Drop to import profile</h2>
      <p class="mt-1 text-sm text-foreground-muted">Release the file to start importing</p>
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
