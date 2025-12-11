<script lang="ts">
  import { closeModal, modalStore } from "$lib/stores/modal";
  import { getName, getVersion } from "@tauri-apps/api/app";
  import { onMount } from "svelte";
  import ExternalLink from "./ExternalLink.svelte";
  import Icon from "./Icon.svelte";

  let appName = $state("MagicX Toolbox");
  let appVersion = $state("1.0.0");

  const isOpen = $derived($modalStore === "about");

  onMount(async () => {
    try {
      const [name, version] = await Promise.all([getName(), getVersion()]);
      appName = name;
      appVersion = version;
    } catch (error) {
      console.error("Failed to get app info:", error);
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
    repository: "https://github.com/ehsan18t/magicx-toolbox-gui",
    issues: "https://github.com/ehsan18t/magicx-toolbox-gui/issues",
    releases: "https://github.com/ehsan18t/magicx-toolbox-gui/releases",
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
      class="animate-in zoom-in-95 w-[min(90vw,480px)] rounded-xl border border-border bg-card shadow-xl duration-200"
      role="dialog"
      aria-modal="true"
      aria-labelledby="about-title"
    >
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-border px-5 py-4">
        <div class="flex items-center gap-3">
          <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-accent/15">
            <Icon icon="mdi:magic-staff" width="24" class="text-accent" />
          </div>
          <div>
            <h2 id="about-title" class="m-0 text-lg font-bold text-foreground">{appName}</h2>
            <span class="text-sm text-foreground-muted">v{appVersion}</span>
          </div>
        </div>
        <button
          class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-lg border-0 bg-transparent text-foreground-muted transition-colors hover:bg-[hsl(var(--muted))] hover:text-foreground"
          onclick={closeModal}
          aria-label="Close"
        >
          <Icon icon="mdi:close" width="20" />
        </button>
      </div>

      <!-- Body -->
      <div class="space-y-5 px-5 py-5">
        <!-- Description -->
        <p class="m-0 text-sm leading-relaxed text-foreground-muted">
          A powerful Windows system optimization and tweaking application. Customize your Windows
          experience with easy-to-use tweaks for privacy, performance, and UI enhancements.
        </p>

        <!-- Developer Section -->
        <div class="rounded-lg border border-border bg-surface p-4">
          <h3 class="mb-3 flex items-center gap-2 text-sm font-semibold text-foreground">
            <Icon icon="mdi:account-circle" width="18" class="text-accent" />
            Developer
          </h3>
          <div class="space-y-2">
            <div class="flex items-center gap-2">
              <Icon icon="mdi:account" width="16" class="text-foreground-muted" />
              <span class="text-sm text-foreground">{developer.name}</span>
            </div>
            <div class="flex items-center gap-2">
              <Icon icon="mdi:email" width="16" class="text-foreground-muted" />
              <a href="mailto:{developer.email}" class="text-sm text-accent hover:underline">
                {developer.email}
              </a>
            </div>
            <div class="flex items-center gap-2">
              <Icon icon="mdi:github" width="16" class="text-foreground-muted" />
              <ExternalLink href={developer.github} class="text-sm text-accent hover:underline">
                @ehsan18t
              </ExternalLink>
            </div>
            <div class="flex items-center gap-2">
              <Icon icon="mdi:web" width="16" class="text-foreground-muted" />
              <ExternalLink href={developer.website} class="text-sm text-accent hover:underline">
                ehsankhan.me
              </ExternalLink>
            </div>
          </div>
        </div>

        <!-- Links Section -->
        <div class="flex flex-wrap gap-2">
          <ExternalLink
            href={links.repository}
            class="flex items-center gap-2 rounded-lg border border-border bg-surface px-3 py-2 text-sm text-foreground transition-colors hover:bg-[hsl(var(--muted))]"
          >
            <Icon icon="mdi:github" width="16" />
            Repository
          </ExternalLink>
          <ExternalLink
            href={links.issues}
            class="flex items-center gap-2 rounded-lg border border-border bg-surface px-3 py-2 text-sm text-foreground transition-colors hover:bg-[hsl(var(--muted))]"
          >
            <Icon icon="mdi:bug" width="16" />
            Report Issue
          </ExternalLink>
          <ExternalLink
            href={links.releases}
            class="flex items-center gap-2 rounded-lg border border-border bg-surface px-3 py-2 text-sm text-foreground transition-colors hover:bg-[hsl(var(--muted))]"
          >
            <Icon icon="mdi:download" width="16" />
            Releases
          </ExternalLink>
        </div>

        <!-- License & Copyright -->
        <div class="border-t border-border pt-4 text-center">
          <p class="m-0 text-xs text-foreground-muted">Licensed under MIT License</p>
          <p class="m-0 mt-1 text-xs text-foreground-subtle">
            Copyright © 2025 Ehsan Khan. All rights reserved.
          </p>
        </div>
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
