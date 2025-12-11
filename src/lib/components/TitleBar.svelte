<script lang="ts">
  import { debugState } from "$lib/stores/debug.svelte";
  import { themeStore } from "$lib/stores/theme";
  import { systemStore } from "$lib/stores/tweaks";
  import Icon from "@iconify/svelte";
  import { getName, getVersion } from "@tauri-apps/api/app";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import ControlButton from "./ControlButton.svelte";

  let appWindow: ReturnType<typeof getCurrentWindow>;
  let appName = $state("");
  let appVersion = $state("");
  let isMaximized = $state(false);
  let isLoaded = $state(false);
  let appIcon = $state("/icons/Toolbox.ico");
  let isRestarting = $state(false);

  onMount(() => {
    let unlisten: (() => void) | undefined;

    const init = async () => {
      try {
        appWindow = getCurrentWindow();

        const [title, version, maximized] = await Promise.allSettled([
          getName(),
          getVersion(),
          appWindow.isMaximized(),
        ]);

        appName = title.status === "fulfilled" ? title.value : "MagicX Toolbox";
        appVersion = version.status === "fulfilled" ? version.value : "1.0.0";
        isMaximized = maximized.status === "fulfilled" ? maximized.value : false;

        isLoaded = true;

        unlisten = await appWindow.onResized(async () => {
          try {
            isMaximized = await appWindow.isMaximized();
          } catch (error) {
            console.warn("Failed to check maximized state:", error);
          }
        });
      } catch (error) {
        console.error("Failed to initialize titlebar:", error);
        appName = "MagicX Toolbox";
        appVersion = "1.0.0";
        isLoaded = true;
      }
    };

    init();

    return () => {
      if (unlisten) unlisten();
    };
  });

  const minimize = async () => {
    try {
      await appWindow?.minimize();
    } catch (error) {
      console.error("Failed to minimize:", error);
    }
  };

  const maximize = async () => {
    if (!appWindow) return;
    try {
      isMaximized ? await appWindow.unmaximize() : await appWindow.maximize();
    } catch (error) {
      console.error("Failed to maximize/restore:", error);
    }
  };

  const close = async () => {
    try {
      await appWindow?.close();
    } catch (error) {
      console.error("Failed to close:", error);
    }
  };

  // Use theme store for theme toggling
  const toggleTheme = () => {
    themeStore.toggle();
  };

  // Restart the app as admin
  const restartAsAdmin = async () => {
    if (isRestarting) return;
    isRestarting = true;
    try {
      await invoke("restart_as_admin");
    } catch (error) {
      console.error("Failed to restart as admin:", error);
      isRestarting = false;
    }
  };
</script>

{#if isLoaded}
  <div
    class="titlebar flex h-10 items-center justify-between border-b border-border bg-elevated pr-1 pl-1.5 text-foreground backdrop-blur-sm select-none drag-enable"
  >
    <div class="app-info flex items-center gap-3">
      <div
        class="icon-container flex h-7 w-7 items-center justify-center rounded-md bg-accent/10 p-1"
      >
        <img
          src={appIcon}
          alt="App Icon"
          class="h-full w-full rounded-sm object-contain"
          onerror={() => {
            appIcon = "";
          }}
        />
        {#if !appIcon}
          <Icon icon="tabler:app-window" width="16" height="16" class="text-accent" />
        {/if}
      </div>

      <div class="app-details flex items-center gap-2">
        <span class="font-semibold tracking-tight text-foreground">
          {appName}
          <span class="text-xs font-medium text-foreground-subtle">
            v{appVersion}
          </span>
        </span>

        <!-- Admin Status Indicator -->
        {#if $systemStore?.is_admin}
          <span
            class="flex items-center gap-1 rounded-md bg-success/15 px-1.5 py-0.5 text-[10px] font-bold text-success uppercase"
            title="Running as Administrator"
          >
            <Icon icon="tabler:shield-check-filled" width="12" height="12" />
            Admin
          </span>
        {:else if $systemStore !== null}
          <span
            class="flex items-center gap-1 rounded-md bg-warning/15 px-1.5 py-0.5 text-[10px] font-bold text-warning uppercase"
            title="Running as Standard User - Some features require Administrator"
          >
            <Icon icon="tabler:shield-x" width="12" height="12" />
            User
          </span>
        {/if}
      </div>
    </div>

    <div class="window-controls flex items-center drag-disable">
      <!-- Debug Toggle -->
      <button
        title={debugState.enabled
          ? `Debug ON (${debugState.logCounts.total} logs) - Click to open panel`
          : "Debug OFF - Click to enable"}
        onclick={() => {
          if (debugState.enabled) {
            debugState.togglePanel();
          } else {
            debugState.toggle();
          }
        }}
        oncontextmenu={(e) => {
          e.preventDefault();
          debugState.toggle();
        }}
        class="relative flex h-8 w-8 items-center justify-center rounded-md transition-all duration-150 hover:bg-foreground/10 {debugState.enabled
          ? 'text-warning'
          : 'text-foreground-muted'}"
      >
        <Icon icon="tabler:bug" width="18" height="18" />
        {#if debugState.enabled && debugState.logCounts.total > 0}
          <span
            class="absolute -top-0.5 -right-0.5 flex h-4 min-w-4 items-center justify-center rounded-full bg-warning px-1 text-[10px] font-bold text-background"
          >
            {debugState.logCounts.total > 99 ? "99+" : debugState.logCounts.total}
          </span>
        {/if}
      </button>

      <!-- Restart as Admin button (only shown if not running as admin) -->
      {#if $systemStore !== null && !$systemStore.is_admin}
        <button
          title="Restart as Administrator"
          onclick={restartAsAdmin}
          disabled={isRestarting}
          class="relative flex h-8 w-8 items-center justify-center rounded-md text-warning transition-all duration-150 hover:bg-foreground/10 hover:text-warning {isRestarting
            ? 'cursor-not-allowed opacity-50'
            : ''}"
        >
          <Icon
            icon={isRestarting ? "tabler:loader-2" : "tabler:shield-up"}
            width="18"
            height="18"
            class={isRestarting ? "animate-spin" : ""}
          />
        </button>
      {/if}

      <ControlButton
        title="Toggle Theme"
        icon={$themeStore === "dark" ? "tabler:moon" : "tabler:sun"}
        onClick={toggleTheme}
        variant="theme"
      />

      <!-- Divider -->
      <div class="mx-2 h-4 w-px bg-foreground-muted/20"></div>

      <ControlButton
        title="Minimize"
        icon="fluent:minimize-20-filled"
        onClick={minimize}
        variant="default"
      />
      <ControlButton
        title={isMaximized ? "Restore" : "Maximize"}
        icon={isMaximized ? "tabler:copy" : "fluent:maximize-20-filled"}
        onClick={maximize}
        variant="default"
      />
      <ControlButton title="Close" icon="tabler:x" onClick={close} variant="danger" />
    </div>
  </div>
{:else}
  <!-- Loading state with proper theme colors -->
  <div class="titlebar h-12 border-b border-border bg-elevated drag-enable">
    <div class="flex h-full items-center justify-center">
      <div class="animate-pulse text-xs text-foreground-muted">Loading...</div>
    </div>
  </div>
{/if}

<style>
  .titlebar {
    /* Add subtle glass effect */
    background-color: hsl(var(--elevated) / 0.9);
    backdrop-filter: blur(8px);
    border-bottom: 1px solid hsl(var(--border) / 0.5);
  }
</style>
