<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { ExternalLink, Icon } from "$lib/components/shared";
  import { Modal } from "$lib/components/ui";
  import { APP_CONFIG } from "$lib/config/app";
  import { closeModal, modalStore } from "$lib/stores/modal.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { systemStore } from "$lib/stores/tweaks.svelte";
  import { delay } from "$lib/utils/motion";
  import { getTauriVersion, getVersion } from "@tauri-apps/api/app";
  import { onMount } from "svelte";

  let appVersion = $state("");
  let tauriVersion = $state("");
  let copied = $state(false);

  const isOpen = $derived(modalStore.current === "about");
  const info = $derived(systemStore.info);

  onMount(async () => {
    const [app, tauri] = await Promise.allSettled([getVersion(), getTauriVersion()]);
    if (app.status === "fulfilled") appVersion = app.value;
    if (tauri.status === "fulfilled") tauriVersion = tauri.value;
  });

  const repo = APP_CONFIG.githubRepo;
  const links = [
    { label: "Source code", href: repo },
    { label: "Releases", href: `${repo}/releases` },
    { label: "Report a problem", href: `${repo}/issues/new` },
  ];

  const facts = $derived(
    [
      { label: "Windows", value: info ? `${info.windows.product_name} ${info.windows.display_version}` : null },
      { label: "Build", value: info?.windows.build_number ?? null },
      { label: "Running as", value: info ? (info.is_admin ? "Administrator" : "Standard user") : null },
      { label: "Engine", value: tauriVersion ? `Tauri ${tauriVersion}` : null },
    ].filter((f): f is { label: string; value: string } => f.value !== null),
  );

  async function copyDetails() {
    const text = [`${APP_CONFIG.appName} ${appVersion}`, ...facts.map((f) => `${f.label}: ${f.value}`)].join("\n");
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      setTimeout(() => (copied = false), delay("feedback"));
    } catch {
      toastStore.error("Could not copy to the clipboard");
    }
  }
</script>

<Modal open={isOpen} onclose={closeModal} size="md" labelledBy="about-title">
  <div class="relative overflow-y-auto px-7 pt-7 pb-6">
    <button
      type="button"
      class="absolute top-3 right-3 flex h-8 w-8 cursor-pointer items-center justify-center rounded-md text-foreground-muted hover:bg-muted hover:text-foreground"
      aria-label="Close"
      onclick={closeModal}
    >
      <Icon icon="mdi:close" width="18" />
    </button>

    <img src="/icons/Toolbox.ico" alt="" width="44" height="44" class="block" />
    <h2 id="about-title" class="m-0 mt-4 font-display text-hero leading-none font-semibold tracking-display">
      {APP_CONFIG.appName}
    </h2>
    <p class="m-0 mt-2 text-sm text-foreground-muted">
      {appVersion ? `Version ${appVersion}` : "Version unknown"}
    </p>

    <p class="m-0 mt-6 text-lead leading-relaxed">
      Curated Windows tweaks you can apply, check against the live system, and undo from snapshots.
    </p>

    <div class="mt-6 border-t border-border pt-4">
      <div class="flex items-start justify-between gap-4">
        <dl class="m-0 grid grid-cols-[auto_minmax(0,1fr)] gap-x-6 gap-y-1.5 text-ui">
          {#each facts as fact (fact.label)}
            <dt class="text-foreground-muted">{fact.label}</dt>
            <dd class="m-0 select-text">{fact.value}</dd>
          {/each}
        </dl>
        <button
          type="button"
          class="inline-flex h-8 shrink-0 cursor-pointer items-center gap-1.5 rounded-md border border-border px-3 text-ui font-medium hover:bg-muted"
          onclick={copyDetails}
          use:tooltip={"Copy the version and system details for a bug report"}
        >
          <Icon
            icon={copied ? "mdi:check" : "mdi:content-copy"}
            width="15"
            class={copied ? "animate-pop-in text-success" : ""}
          />
          {copied ? "Copied" : "Copy details"}
        </button>
      </div>
    </div>

    <nav class="mt-5 flex flex-wrap gap-x-6 gap-y-2 border-t border-border pt-4 text-ui" aria-label="Project links">
      {#each links as link (link.label)}
        <ExternalLink
          href={link.href}
          class="font-medium text-foreground underline decoration-foreground-subtle underline-offset-4 hover:text-accent hover:decoration-accent"
        >
          {link.label}
        </ExternalLink>
      {/each}
    </nav>

    <footer class="mt-5 flex flex-wrap items-center justify-between gap-3 border-t border-border pt-4 text-xs">
      <p class="m-0 text-foreground-muted">
        Made by Ehsan Khan. Free and open source under the
        <ExternalLink
          href="{repo}/blob/main/LICENSE"
          class="text-foreground underline underline-offset-2 hover:text-accent"
        >
          MIT License</ExternalLink
        >.
      </p>
      <div class="flex gap-0.5">
        {#each [{ icon: "mdi:github", label: "Ehsan Khan on GitHub", href: "https://github.com/ehsan18t" }, { icon: "mdi:web", label: "ehsankhan.me", href: "https://ehsankhan.me" }] as profile (profile.href)}
          <ExternalLink
            href={profile.href}
            class="flex h-7 w-7 items-center justify-center rounded-md text-foreground-muted hover:bg-muted hover:text-foreground"
            aria-label={profile.label}
          >
            <Icon icon={profile.icon} width="16" />
          </ExternalLink>
        {/each}
        <a
          href="mailto:ehsan18t@gmail.com"
          class="flex h-7 w-7 items-center justify-center rounded-md text-foreground-muted hover:bg-muted hover:text-foreground"
          aria-label="Email Ehsan Khan"
        >
          <Icon icon="mdi:email" width="16" />
        </a>
      </div>
    </footer>
  </div>
</Modal>
