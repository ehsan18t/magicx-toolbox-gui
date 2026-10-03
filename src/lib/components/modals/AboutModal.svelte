<script lang="ts" module>
  import { APP_CONFIG } from "$lib/config/app";

  const { githubRepo: repo, author } = APP_CONFIG;

  const PROJECT_LINKS = [
    { label: "Source code", href: repo },
    { label: "Releases", href: `${repo}/releases` },
    { label: "Report a problem", href: `${repo}/issues/new` },
  ];

  const AUTHOR_LINKS: { icon: IconName; label: string; href: string }[] = [
    { icon: "mdi:github", label: `${author.name} on GitHub`, href: author.github },
    { icon: "mdi:web", label: new URL(author.website).host, href: author.website },
    { icon: "mdi:email", label: `Email ${author.name}`, href: `mailto:${author.email}` },
  ];
</script>

<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { ExternalLink, Icon, type IconName } from "$lib/components/shared";
  import { IconButton, Modal, textLink } from "$lib/components/ui";
  import { button } from "$lib/components/ui/variants";
  import { appInfoStore } from "$lib/stores/appInfo.svelte";
  import { modalStore } from "$lib/stores/modal.svelte";
  import { systemStore } from "$lib/stores/system.svelte";
  import { copyText } from "$lib/utils/clipboard";
  import { delay } from "$lib/utils/motion";
  import { onMount } from "svelte";

  const titleId = $props.id();
  let copied = $state(false);
  let copiedTimer: ReturnType<typeof setTimeout> | undefined;

  const isOpen = $derived(modalStore.current === "about");
  const info = $derived(systemStore.info);
  const appVersion = $derived(appInfoStore.version);

  onMount(() => {
    void appInfoStore.load();
    return () => clearTimeout(copiedTimer);
  });

  const facts = $derived(
    [
      { label: "Windows", value: info ? `${info.windows.product_name} ${info.windows.display_version}` : null },
      { label: "Build", value: info?.windows.build_number ?? null },
      { label: "Running as", value: info ? (info.is_admin ? "Administrator" : "Standard user") : null },
      { label: "Engine", value: appInfoStore.tauriVersion ? `Tauri ${appInfoStore.tauriVersion}` : null },
    ].filter((f): f is { label: string; value: string } => f.value !== null),
  );

  async function copyDetails() {
    const text = [`${APP_CONFIG.appName} ${appVersion}`, ...facts.map((f) => `${f.label}: ${f.value}`)].join("\n");
    if (!(await copyText(text, "Could not copy to the clipboard"))) return;
    copied = true;
    clearTimeout(copiedTimer);
    copiedTimer = setTimeout(() => (copied = false), delay("feedback"));
  }
</script>

<Modal open={isOpen} onclose={modalStore.close} size="md" labelledBy={titleId}>
  <div class="relative overflow-y-auto px-7 pt-7 pb-6">
    <IconButton icon="mdi:close" label="Close" class="absolute top-3 right-3" onclick={modalStore.close} />

    <img src={APP_CONFIG.appIcon} alt="" width="44" height="44" class="block" />
    <h2 id={titleId} class="m-0 mt-4 font-display text-hero leading-none font-semibold tracking-display">
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
        <dl class="m-0 grid grid-cols-label-value gap-x-6 gap-y-1.5 text-ui">
          {#each facts as fact (fact.label)}
            <dt class="text-foreground-muted">{fact.label}</dt>
            <dd class="m-0 select-text">{fact.value}</dd>
          {/each}
        </dl>
        <button
          type="button"
          class={button({ variant: "outline", class: "shrink-0" })}
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
      {#each PROJECT_LINKS as link (link.label)}
        <ExternalLink href={link.href} class="text-foreground {textLink}">{link.label}</ExternalLink>
      {/each}
    </nav>

    <footer class="mt-5 flex flex-wrap items-center justify-between gap-3 border-t border-border pt-4 text-xs">
      <p class="m-0 text-foreground-muted">
        Made by {author.name}. Free and open source under the
        <ExternalLink
          href="{repo}/blob/main/LICENSE"
          class="text-foreground underline underline-offset-2 hover:text-accent"
        >
          MIT License</ExternalLink
        >.
      </p>
      <div class="flex gap-0.5">
        {#each AUTHOR_LINKS as link (link.href)}
          <ExternalLink
            href={link.href}
            class="flex h-7 w-7 items-center justify-center rounded-md text-foreground-muted hover:bg-muted hover:text-foreground"
            aria-label={link.label}
          >
            <Icon icon={link.icon} width="16" />
          </ExternalLink>
        {/each}
      </div>
    </footer>
  </div>
</Modal>
