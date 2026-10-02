<script lang="ts">
  import { tooltip } from "$lib/actions/tooltip";
  import { ThemeToggle } from "$lib/components/settings";
  import { Icon } from "$lib/components/shared";
  import { SearchInput } from "$lib/components/ui";
  import { elevationStore } from "$lib/stores/elevation.svelte";
  import { LOGS_PANEL_ID, LOGS_TOGGLE_ID, logsStore } from "$lib/stores/logs.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { pageFilterStore } from "$lib/stores/pageFilter.svelte";
  import { searchStore } from "$lib/stores/search.svelte";
  import { sidebarStore } from "$lib/stores/sidebar.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { systemStore } from "$lib/stores/tweaksData.svelte";
  import { getName, getVersion } from "@tauri-apps/api/app";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount, tick } from "svelte";
  import WindowControlButton from "./WindowControlButton.svelte";

  let appWindow: ReturnType<typeof getCurrentWindow>;
  let appName = $state("MagicX Toolbox");
  let appVersion = $state("");
  let isMaximized = $state(false);
  let appIcon = $state("/icons/Toolbox.ico");

  const isRestarting = $derived(elevationStore.isRestarting);
  const isAdmin = $derived(systemStore.info?.is_admin ?? null);

  let searchEl = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (navigationStore.focusSearchSignal > 0) tick().then(() => searchEl?.focus());
  });

  const scoped = $derived(pageFilterStore.isScoped);
  const chipTab = $derived(pageFilterStore.chipTab);
  const scopeName = $derived(navigationStore.allTabs.find((t) => t.id === chipTab)?.name ?? "");

  function handleSearch(value: string) {
    if (scoped) pageFilterStore.setQuery(value);
    else if (navigationStore.isScopable) pageFilterStore.searchEverywhere(value);
    else {
      searchStore.setQuery(value);
      if (value && navigationStore.activeTab !== "search") navigationStore.navigateToTab("search");
    }
  }

  function toggleScope() {
    if (scoped) pageFilterStore.searchEverywhere();
    else pageFilterStore.scopeIn();
    searchEl?.focus();
  }

  onMount(() => {
    let unlisten: (() => void) | undefined;
    // The backend refuses to close mid-apply (a half-applied tweak has nothing to undo it); say so.
    let unlistenCloseBlocked: (() => void) | undefined;

    const init = async () => {
      try {
        appWindow = getCurrentWindow();
        const [title, version, maximized] = await Promise.allSettled([
          getName(),
          getVersion(),
          appWindow.isMaximized(),
        ]);
        if (title.status === "fulfilled") appName = title.value;
        if (version.status === "fulfilled") appVersion = version.value;
        isMaximized = maximized.status === "fulfilled" ? maximized.value : false;

        unlistenCloseBlocked = await listen<string>("close-blocked", (event) => {
          toastStore.show("warning", event.payload);
        });
        unlisten = await appWindow.onResized(async () => {
          try {
            isMaximized = await appWindow.isMaximized();
          } catch (error) {
            console.warn("Failed to check maximized state:", error);
          }
        });
      } catch (error) {
        console.error("Failed to initialize titlebar:", error);
      }
    };

    init();

    return () => {
      unlisten?.();
      unlistenCloseBlocked?.();
    };
  });

  async function windowCall(action: "minimize" | "toggleMaximize" | "close") {
    try {
      // The capability grants maximize and unmaximize, not toggle-maximize.
      if (action === "toggleMaximize") await (isMaximized ? appWindow?.unmaximize() : appWindow?.maximize());
      else await appWindow?.[action]();
    } catch (error) {
      console.error(`Window ${action} failed:`, error);
    }
  }
</script>

<header class="flex h-titlebar shrink-0 items-stretch bg-background text-foreground select-none drag-enable">
  <div class="flex min-w-0 shrink items-center gap-1 pl-1">
    <button
      type="button"
      class="flex h-9 w-10 shrink-0 cursor-pointer items-center justify-center rounded-md text-foreground drag-disable hover:bg-muted"
      aria-label={sidebarStore.isOpen ? "Collapse navigation" : "Expand navigation"}
      aria-expanded={sidebarStore.isOpen}
      onclick={() => sidebarStore.toggle()}
      use:tooltip={sidebarStore.isOpen ? "Collapse navigation" : "Expand navigation"}
    >
      <Icon icon="fluent:navigation-20-regular" width="18" />
    </button>

    <div class="flex min-w-0 items-center gap-2.5 pl-1.5">
      {#if appIcon}
        <img src={appIcon} alt="" class="h-4 w-4 shrink-0" onerror={() => (appIcon = "")} />
      {/if}
      <span class="hidden truncate text-xs text-foreground titlebar-name:inline">{appName}</span>
      {#if appVersion}
        <span class="hidden shrink-0 text-xs text-foreground-subtle titlebar-version:inline">{appVersion}</span>
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
          <Icon icon={isAdmin ? "fluent:shield-checkmark-16-filled" : "fluent:shield-error-16-filled"} width="12" />
          {isAdmin ? "Admin" : "Standard user"}
        </span>
      {/if}
    </div>
  </div>

  <div class="flex min-w-36 flex-1 items-center justify-center px-3">
    {#snippet scopeToggle()}
      <button
        type="button"
        class="inline-flex max-w-9/20 shrink-0 animate-pop-in cursor-pointer items-center gap-1 rounded border px-1.5 py-0.5 text-xs font-medium {scoped
          ? 'border-accent/40 bg-accent/15 text-accent hover:bg-accent/20'
          : 'border-border-hover bg-muted text-foreground-muted hover:border-accent/50 hover:text-foreground'}"
        aria-pressed={scoped}
        aria-label="Search only in {scopeName}"
        use:tooltip={scoped
          ? `Searching only in ${scopeName}. Select to search everywhere.`
          : `Search only in ${scopeName}`}
        onclick={toggleScope}
      >
        <Icon icon={scoped ? "mdi:filter-variant" : "mdi:filter-variant-remove"} width="13" class="shrink-0" />
        <span class="truncate">{scopeName}</span>
      </button>
    {/snippet}
    <SearchInput
      bind:inputRef={searchEl}
      value={scoped ? pageFilterStore.query : searchStore.query}
      placeholder={scoped ? "Filter (Ctrl+K)" : "Search tweaks and apps (Ctrl+K)"}
      label={scoped ? `Filter ${scopeName}` : "Search tweaks and apps"}
      class="w-full max-w-100 drag-disable"
      trailing={chipTab ? scopeToggle : undefined}
      onbackspace={scoped ? toggleScope : undefined}
      oninput={handleSearch}
    />
  </div>

  <div class="flex shrink-0 items-center gap-0.5 pr-0 drag-disable">
    {#if isAdmin === false}
      <button
        type="button"
        class="flex h-8 animate-fade-in cursor-pointer items-center gap-1.5 rounded-md px-2 text-xs font-medium text-warning hover:bg-muted disabled:cursor-wait disabled:opacity-60"
        onclick={elevationStore.restartAsAdmin}
        disabled={isRestarting}
        use:tooltip={"Restart as administrator"}
        aria-label="Restart as administrator"
      >
        <Icon
          icon={isRestarting ? "mdi:loading" : "fluent:shield-keyhole-16-regular"}
          width="16"
          class={isRestarting ? "animate-spin" : ""}
        />
        <span class="hidden titlebar-admin:inline">Restart as admin</span>
      </button>
    {/if}

    <button
      type="button"
      id={LOGS_TOGGLE_ID}
      aria-label="Logs"
      aria-expanded={logsStore.isPanelOpen}
      aria-controls={LOGS_PANEL_ID}
      use:tooltip={"Logs"}
      onclick={() => logsStore.togglePanel()}
      class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-md hover:bg-muted {logsStore.isPanelOpen
        ? 'text-accent'
        : 'text-foreground-muted'}"
    >
      <Icon icon="fluent:bug-20-regular" width="18" />
    </button>

    <ThemeToggle />

    <div class="ml-2 flex items-center gap-0.5 pr-1.5">
      <WindowControlButton title="Minimize" glyph="minimize" onclick={() => windowCall("minimize")} />
      <WindowControlButton
        title={isMaximized ? "Restore" : "Maximize"}
        glyph={isMaximized ? "restore" : "maximize"}
        onclick={() => windowCall("toggleMaximize")}
      />
      <WindowControlButton title="Close" glyph="close" onclick={() => windowCall("close")} />
    </div>
  </div>
</header>
