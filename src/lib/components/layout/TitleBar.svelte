<script lang="ts">
  import { Icon } from "$lib/components/shared";
  import { Badge, Button, IconButton } from "$lib/components/ui";
  import { APP_CONFIG } from "$lib/config/app";
  import { appInfoStore } from "$lib/stores/appInfo.svelte";
  import { elevationStore } from "$lib/stores/elevation.svelte";
  import { LOGS_PANEL_ID, LOGS_TOGGLE_ID, logsStore } from "$lib/stores/logs.svelte";
  import { sidebarStore } from "$lib/stores/sidebar.svelte";
  import ThemeToggle from "./ThemeToggle.svelte";
  import TitleBarSearch from "./TitleBarSearch.svelte";
  import WindowControls from "./WindowControls.svelte";

  let iconFailed = $state(false);

  const isAdmin = $derived(elevationStore.isAdmin);
  const navLabel = $derived(sidebarStore.isOpen ? "Collapse navigation" : "Expand navigation");
</script>

<header class="flex h-titlebar shrink-0 items-stretch bg-background text-foreground select-none drag-enable">
  <div class="flex min-w-0 shrink items-center gap-1 pl-1">
    <IconButton
      icon="fluent:navigation-20-regular"
      tooltip={navLabel}
      aria-expanded={sidebarStore.isOpen}
      class="drag-disable"
      onclick={() => sidebarStore.toggle()}
    />

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
        <Badge
          tone={isAdmin ? "success" : "warning"}
          size="caption"
          case="none"
          class="shrink-0 animate-fade-in"
          tooltip={isAdmin ? "Running as administrator" : "Running as a standard user: some tweaks need administrator"}
        >
          <Icon icon={isAdmin ? "fluent:shield-checkmark-16-filled" : "fluent:shield-error-16-filled"} size="3xs" />
          {isAdmin ? "Admin" : "Standard user"}
        </Badge>
      {/if}
    </div>
  </div>

  <div class="flex min-w-36 flex-1 items-center justify-center px-3">
    <TitleBarSearch />
  </div>

  <div class="flex shrink-0 items-center gap-0.5 pr-0 drag-disable">
    {#if isAdmin === false}
      <Button
        variant="ghost"
        size="sm"
        tone="warning"
        icon="fluent:shield-keyhole-16-regular"
        loading={elevationStore.isRestarting}
        tooltip="Restart as administrator"
        aria-label="Restart as administrator"
        class="animate-fade-in"
        onclick={() => elevationStore.restartAsAdmin()}
      >
        <span class="hidden titlebar-admin:inline">Restart as admin</span>
      </Button>
    {/if}

    <IconButton
      id={LOGS_TOGGLE_ID}
      icon="fluent:bug-20-regular"
      tooltip="Logs"
      active={logsStore.isPanelOpen}
      aria-expanded={logsStore.isPanelOpen}
      aria-controls={LOGS_PANEL_ID}
      onclick={() => logsStore.togglePanel()}
    />

    <ThemeToggle />

    <WindowControls />
  </div>
</header>
