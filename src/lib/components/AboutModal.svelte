<script lang="ts">
  import { APP_CONFIG } from "$lib/config/app";
  import { closeModal, modalStore } from "$lib/stores/modal";
  import { getVersion } from "@tauri-apps/api/app";
  import { onMount } from "svelte";
  import ExternalLink from "./ExternalLink.svelte";
  import Icon from "./Icon.svelte";

  let appVersion = $state("1.0.0");

  const isOpen = $derived($modalStore === "about");

  onMount(async () => {
    try {
      appVersion = await getVersion();
    } catch (error) {
      console.error("Failed to get app version:", error);
    }
  });

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      closeModal();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && isOpen) {
      closeModal();
    }
  }

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

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
  <div
    class="fixed inset-0 z-1000 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    role="presentation"
    onclick={handleBackdropClick}
  >
    <div
      class="animate-in zoom-in-95 w-[min(90vw,420px)] overflow-hidden rounded-xl border border-border bg-card shadow-2xl duration-200"
      role="dialog"
      aria-modal="true"
      aria-labelledby="about-title"
    >
      <!-- Header with gradient accent -->
      <div
        class="relative bg-linear-to-br from-accent/20 via-accent/10 to-transparent px-6 pt-6 pb-4"
      >
        <!-- Close button -->
        <button
          class="absolute top-3 right-3 flex h-8 w-8 cursor-pointer items-center justify-center rounded-lg border-0 bg-black/10 text-foreground-muted transition-colors hover:bg-black/20 hover:text-foreground"
          onclick={closeModal}
          aria-label="Close"
        >
          <Icon icon="mdi:close" width="18" />
        </button>

        <!-- App branding -->
        <div class="flex flex-col items-center text-center">
          <div
            class="mb-3 flex h-16 w-16 items-center justify-center rounded-2xl bg-accent/20 shadow-lg"
          >
            <Icon icon="mdi:magic-staff" width="36" class="text-accent" />
          </div>
          <h2 id="about-title" class="m-0 text-xl font-bold text-foreground">
            {APP_CONFIG.appName}
          </h2>
          <div class="mt-1 flex items-center gap-2">
            <span class="rounded-full bg-accent/15 px-2.5 py-0.5 text-xs font-semibold text-accent">
              v{appVersion}
            </span>
          </div>
        </div>
      </div>

      <!-- Body -->
      <div class="space-y-4 px-6 py-5">
        <!-- Description -->
        <p class="m-0 text-center text-sm leading-relaxed text-foreground-muted">
          A powerful Windows system optimization and tweaking application for privacy, performance,
          and customization.
        </p>

        <!-- Quick Links -->
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

        <!-- Developer Card -->
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

        <!-- Footer -->
        <p class="m-0 text-center text-[11px] text-foreground-subtle">
          © 2025 {developer.name}. Made with ❤️ for Windows users.
        </p>
      </div>
    </div>
  </div>
{/if}

<style>
  @keyframes zoom-in-95 {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .animate-in {
    animation: zoom-in-95 0.2s ease-out;
  }
</style>
