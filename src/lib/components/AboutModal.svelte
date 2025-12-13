<script lang="ts">
  import { APP_CONFIG } from "$lib/config/app";
  import { closeModal, modalStore } from "$lib/stores/modal.svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { onMount } from "svelte";
  import ExternalLink from "./ExternalLink.svelte";
  import Icon from "./Icon.svelte";
  import { IconButton, Modal, ModalBody } from "./ui";

  let appVersion = $state("1.0.0");

  const isOpen = $derived(modalStore.current === "about");

  onMount(async () => {
    try {
      appVersion = await getVersion();
    } catch (error) {
      console.error("Failed to get app version:", error);
    }
  });

  const developer = {
    name: "Ehsan Khan",
    email: "ehsan18t@gmail.com",
    github: "https://github.com/ehsan18t",
    website: "https://ehsankhan.me",
  };

  const links = {
    repository: APP_CONFIG.githubRepo,
    issues: `${APP_CONFIG.githubRepo}/issues`,
    releases: `${APP_CONFIG.githubRepo}/releases`,
    license: `${APP_CONFIG.githubRepo}/blob/main/LICENSE`,
  };
</script>

<Modal open={isOpen} onclose={closeModal} size="sm">
  <!-- Custom Header with gradient -->
  <div class="relative bg-linear-to-br from-accent/20 via-accent/10 to-transparent px-6 pt-6 pb-4">
    <IconButton
      icon="mdi:close"
      size={18}
      class="absolute top-3 right-3 bg-black/10 hover:bg-black/20"
      onclick={closeModal}
      aria-label="Close"
    />

    <div class="flex flex-col items-center text-center">
      <div class="mb-3 flex h-16 w-16 items-center justify-center rounded-2xl bg-accent/20 shadow-lg">
        <Icon icon="mdi:magic-staff" width="36" class="text-accent" />
      </div>
      <h2 class="m-0 text-xl font-bold text-foreground">
        {APP_CONFIG.appName}
      </h2>
      <div class="mt-1 flex items-center gap-2">
        <span class="rounded-full bg-accent/15 px-2.5 py-0.5 text-xs font-semibold text-accent">
          v{appVersion}
        </span>
      </div>
    </div>
  </div>

  <ModalBody class="space-y-4 px-6 py-5">
    <p class="m-0 text-center text-sm leading-relaxed text-foreground-muted">
      A powerful Windows system optimization and tweaking application for privacy, performance, and customization.
    </p>

    <div class="grid grid-cols-2 gap-2">
      <ExternalLink
        href={links.repository}
        class="flex items-center justify-center gap-2 rounded-lg border border-border bg-surface px-3 py-2.5 text-sm text-foreground transition-colors hover:bg-[hsl(var(--muted))]"
      >
        <Icon icon="mdi:github" width="18" />
        <span>Source</span>
      </ExternalLink>
      <ExternalLink
        href={links.releases}
        class="flex items-center justify-center gap-2 rounded-lg border border-border bg-surface px-3 py-2.5 text-sm text-foreground transition-colors hover:bg-[hsl(var(--muted))]"
      >
        <Icon icon="mdi:download" width="18" />
        <span>Releases</span>
      </ExternalLink>
      <ExternalLink
        href={links.issues}
        class="flex items-center justify-center gap-2 rounded-lg border border-border bg-surface px-3 py-2.5 text-sm text-foreground transition-colors hover:bg-[hsl(var(--muted))]"
      >
        <Icon icon="mdi:bug" width="18" />
        <span>Report Bug</span>
      </ExternalLink>
      <ExternalLink
        href={links.license}
        class="flex items-center justify-center gap-2 rounded-lg border border-border bg-surface px-3 py-2.5 text-sm text-foreground transition-colors hover:bg-[hsl(var(--muted))]"
      >
        <Icon icon="mdi:license" width="18" />
        <span>MIT License</span>
      </ExternalLink>
    </div>

    <div class="rounded-lg border border-border bg-surface/50 p-4">
      <div class="flex items-center gap-3">
        <div class="flex h-10 w-10 items-center justify-center rounded-full bg-accent/15">
          <Icon icon="mdi:account" width="20" class="text-accent" />
        </div>
        <div class="flex-1">
          <span class="block text-sm font-semibold text-foreground">{developer.name}</span>
          <span class="text-xs text-foreground-muted">Developer</span>
        </div>
        <div class="flex gap-1">
          <ExternalLink
            href={developer.github}
            class="flex h-8 w-8 items-center justify-center rounded-lg text-foreground-muted transition-colors hover:bg-accent/10 hover:text-accent"
            title="GitHub"
          >
            <Icon icon="mdi:github" width="18" />
          </ExternalLink>
          <ExternalLink
            href={developer.website}
            class="flex h-8 w-8 items-center justify-center rounded-lg text-foreground-muted transition-colors hover:bg-accent/10 hover:text-accent"
            title="Website"
          >
            <Icon icon="mdi:web" width="18" />
          </ExternalLink>
          <a
            href="mailto:{developer.email}"
            class="flex h-8 w-8 items-center justify-center rounded-lg text-foreground-muted transition-colors hover:bg-accent/10 hover:text-accent"
            title="Email"
          >
            <Icon icon="mdi:email" width="18" />
          </a>
        </div>
      </div>
    </div>

    <p class="m-0 text-center text-[11px] text-foreground-subtle">
      © 2025 {developer.name}. Made with ❤️ for Windows users.
    </p>
  </ModalBody>
</Modal>
