<script lang="ts" module>
  import { APP_CONFIG } from "$lib/config/app";
  import type { IconName } from "$lib/design";

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
  import { ExternalLink, Icon } from "$lib/components/shared";
  import { Button, IconButton, Modal, ModalTitle, textLink } from "$lib/components/ui";
  import { appInfoStore } from "$lib/stores/appInfo.svelte";
  import { modalStore } from "$lib/stores/modal.svelte";
  import { copyText } from "$lib/utils/clipboard";
  import { diagnosticsFacts, diagnosticsHeader, versionLabel } from "$lib/utils/diagnostics";
  import { delay } from "$lib/utils/motion";
  import { onMount } from "svelte";

  let copied = $state(false);
  let copiedTimer: ReturnType<typeof setTimeout> | undefined;

  const isOpen = $derived(modalStore.current === "about");
  const facts = $derived(diagnosticsFacts());

  onMount(() => {
    void appInfoStore.load();
    return () => clearTimeout(copiedTimer);
  });

  async function copyDetails() {
    if (!(await copyText(diagnosticsHeader(), "Could not copy to the clipboard"))) return;
    copied = true;
    clearTimeout(copiedTimer);
    copiedTimer = setTimeout(() => (copied = false), delay("feedback"));
  }
</script>

<Modal open={isOpen} onclose={modalStore.close} size="md">
  <div class="relative overflow-y-auto px-7 pt-7 pb-6">
    <IconButton icon="mdi:close" label="Close" class="absolute top-3 right-3" onclick={modalStore.close} />

    <img src={APP_CONFIG.appIcon} alt="" width="44" height="44" class="block" />
    <ModalTitle size="hero" class="mt-4">{APP_CONFIG.appName}</ModalTitle>
    <p class="m-0 mt-2 text-sm text-foreground-muted">{versionLabel()}</p>

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
        <Button
          variant="outline"
          class="shrink-0"
          tooltip="Copy the version and system details for a bug report"
          onclick={copyDetails}
        >
          <Icon
            icon={copied ? "mdi:check" : "mdi:content-copy"}
            size="sm"
            class={copied ? "animate-pop-in text-success" : ""}
          />
          {copied ? "Copied" : "Copy details"}
        </Button>
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
        <ExternalLink href="{repo}/blob/main/LICENSE" class="text-foreground {textLink}">MIT License</ExternalLink>.
      </p>
      <div class="flex gap-0.5">
        {#each AUTHOR_LINKS as link (link.href)}
          <ExternalLink
            href={link.href}
            class="flex h-7 w-7 items-center justify-center rounded-md text-foreground-muted hover:bg-muted hover:text-foreground"
            aria-label={link.label}
            tooltip={link.label}
          >
            <Icon icon={link.icon} size="md" />
          </ExternalLink>
        {/each}
      </div>
    </footer>
  </div>
</Modal>
