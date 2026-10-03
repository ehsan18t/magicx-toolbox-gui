<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { Icon } from "$lib/components/shared";
  import { IconButton } from "$lib/components/ui";
  import { APP_CONFIG } from "$lib/config/app";
  import { appInfoStore } from "$lib/stores/appInfo.svelte";
  import { elevationStore } from "$lib/stores/elevation.svelte";
  import { LOGS_PANEL_ID, LOGS_TOGGLE_ID, logsStore } from "$lib/stores/logs.svelte";
  import { sidebarStore } from "$lib/stores/sidebar.svelte";
  import { systemStore } from "$lib/stores/system.svelte";
  import ThemeToggle from "./ThemeToggle.svelte";
  import TitleBarSearch from "./TitleBarSearch.svelte";
  import WindowControls from "./WindowControls.svelte";

  let iconFailed = $state(false);

  const isAdmin = $derived(systemStore.info?.is_admin ?? null);
  const navLabel = $derived(sidebarStore.isOpen ? "Collapse navigation" : "Expand navigation");
</script>

<header class="flex h-titlebar shrink-0 items-stretch bg-background text-foreground select-none drag-enable">
  <div class="flex min-w-0 shrink items-center gap-1 pl-1">
    <button
      type="button"
      class="flex h-9 w-10 shrink-0 cursor-pointer items-center justify-center rounded-md text-foreground drag-disable hover:bg-muted"
      aria-label={navLabel}
      aria-expanded={sidebarStore.isOpen}
      onclick={() => sidebarStore.toggle()}
      use:tooltip={navLabel}
    >
      <Icon icon="fluent:navigation-20-regular" size="lg" />
    </button>

    <div class="flex min-w-0 items-center gap-2.5 pl-1.5">
      {#if !iconFailed}
        <img src={APP_CONFIG.appIcon} alt="" class="h-4 w-4 shrink-0" onerror={() => (iconFailed = true)} />
      {/if}
      <span class="hidden truncate text-xs text-foreground titlebar-name:inline">{appInfoStore.name}</span>
      {#if appInfoStore.version}
        <span class="hidden shrink-0 text-xs text-foreground-subtle titlebar-version:inline"
          >{appInfoStore.version}</span
        >
      {/if}
      {#if isAdmin !== null}
        <span
          class="inline-flex shrink-0 animate-fade-in items-center gap-1 rounded px-1.5 py-0.5 text-caption font-semibold {isAdmin
            ? 'bg-success/15 text-success'
            : 'bg-warning/15 text-warning'}"
          use:tooltip={isAdmin
            ? "Running as administrator"
            : "Running as a standard user: some tweaks need administrator"}
        >
          <Icon icon={isAdmin ? "fluent:shield-checkmark-16-filled" : "fluent:shield-error-16-filled"} size="3xs" />
          {isAdmin ? "Admin" : "Standard user"}
        </span>
      {/if}
    </div>
  </div>

  <div class="flex min-w-36 flex-1 items-center justify-center px-3">
    <TitleBarSearch />
  </div>

  <div class="flex shrink-0 items-center gap-0.5 pr-0 drag-disable">
    {#if isAdmin === false}
      <button
        type="button"
        class="flex h-8 animate-fade-in cursor-pointer items-center gap-1.5 rounded-md px-2 text-xs font-medium text-warning hover:bg-muted disabled:cursor-wait disabled:opacity-60"
        onclick={() => elevationStore.restartAsAdmin()}
        disabled={elevationStore.isRestarting}
        use:tooltip={"Restart as administrator"}
        aria-label="Restart as administrator"
      >
        <Icon
          icon={elevationStore.isRestarting ? "mdi:loading" : "fluent:shield-keyhole-16-regular"}
          size="md"
          class={elevationStore.isRestarting ? "animate-spin" : ""}
        />
        <span class="hidden titlebar-admin:inline">Restart as admin</span>
      </button>
    {/if}

    <IconButton
      id={LOGS_TOGGLE_ID}
      icon="fluent:bug-20-regular"
      tooltip="Logs"
      aria-expanded={logsStore.isPanelOpen}
      aria-controls={LOGS_PANEL_ID}
      class={logsStore.isPanelOpen ? "text-accent enabled:hover:text-accent" : undefined}
      onclick={() => logsStore.togglePanel()}
    />

    <ThemeToggle />

    <WindowControls />
  </div>
</header>
