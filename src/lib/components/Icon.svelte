<script lang="ts" module>
  // Pre-import all icons used in the app for build-time bundling
  // MDI Icons
  import BiMotherboard from "~icons/bi/motherboard";
  import MdiMinimize20Filled from "~icons/fluent/minimize-20-filled";
  import MdiAccount from "~icons/mdi/account";
  import MdiAccountCircle from "~icons/mdi/account-circle";
  import MdiAlert from "~icons/mdi/alert";
  import MdiAlertCircle from "~icons/mdi/alert-circle";
  import MdiAlertOctagon from "~icons/mdi/alert-octagon";
  import MdiArrowRightCircle from "~icons/mdi/arrow-right-circle";
  import MdiArrowUpCircle from "~icons/mdi/arrow-up-circle";
  import MdiBackupRestore from "~icons/mdi/backup-restore";
  import MdiBug from "~icons/mdi/bug";
  import MdiCalendar from "~icons/mdi/calendar";
  import MdiCalendarClock from "~icons/mdi/calendar-clock";
  import MdiCheck from "~icons/mdi/check";
  import MdiCheckAll from "~icons/mdi/check-all";
  import MdiCheckCircle from "~icons/mdi/check-circle";
  import MdiChevronDown from "~icons/mdi/chevron-down";
  import MdiChevronRight from "~icons/mdi/chevron-right";
  import MdiChevronUp from "~icons/mdi/chevron-up";
  import MdiCircle from "~icons/mdi/circle";
  import MdiCircleHalfFull from "~icons/mdi/circle-half-full";
  import MdiCircleOutline from "~icons/mdi/circle-outline";
  import MdiClipboardCheckOutline from "~icons/mdi/clipboard-check-outline";
  import MdiClose from "~icons/mdi/close";
  import MdiCloseCircle from "~icons/mdi/close-circle";
  import MdiCloseCircleOutline from "~icons/mdi/close-circle-outline";
  import MdiCog from "~icons/mdi/cog";
  import MdiCogOutline from "~icons/mdi/cog-outline";
  import MdiCogs from "~icons/mdi/cogs";
  import MdiConsole from "~icons/mdi/console";
  import MdiCpu64Bit from "~icons/mdi/cpu-64-bit";
  import MdiDatabaseCogOutline from "~icons/mdi/database-cog-outline";
  import MdiDatabaseExport from "~icons/mdi/database-export";
  import MdiDesktopTowerMonitor from "~icons/mdi/desktop-tower-monitor";
  import MdiDownload from "~icons/mdi/download";
  import MdiEmail from "~icons/mdi/email";
  import MdiEthernet from "~icons/mdi/ethernet";
  import MdiExpansionCard from "~icons/mdi/expansion-card";
  import MdiExport from "~icons/mdi/export";
  import MdiFileDocumentMultiple from "~icons/mdi/file-document-multiple";
  import MdiFileSearchOutline from "~icons/mdi/file-search-outline";
  import MdiFolder from "~icons/mdi/folder";
  import MdiGamepadVariant from "~icons/mdi/gamepad-variant";
  import MdiGithub from "~icons/mdi/github";
  import MdiHarddisk from "~icons/mdi/harddisk";
  import MdiHarddiskPlus from "~icons/mdi/harddisk-plus";
  import MdiHelpCircle from "~icons/mdi/help-circle";
  import MdiHistory from "~icons/mdi/history";
  import MdiImport from "~icons/mdi/import";
  import MdiInformation from "~icons/mdi/information";
  import MdiInformationOutline from "~icons/mdi/information-outline";
  import MdiKeyVariant from "~icons/mdi/key-variant";
  import MdiLaptop from "~icons/mdi/laptop";
  import MdiLicense from "~icons/mdi/license";
  import MdiLightbulbOutline from "~icons/mdi/lightbulb-outline";
  import MdiLoading from "~icons/mdi/loading";
  import MdiMagicStaff from "~icons/mdi/magic-staff";
  import MdiMagnify from "~icons/mdi/magnify";
  import MdiMemory from "~icons/mdi/memory";
  import MdiMicrosoftWindows from "~icons/mdi/microsoft-windows";
  import MdiOpenInNew from "~icons/mdi/open-in-new";
  import MdiPackageVariant from "~icons/mdi/package-variant";
  import MdiPalette from "~icons/mdi/palette";
  import MdiPin from "~icons/mdi/pin";
  import MdiPinOutline from "~icons/mdi/pin-outline";
  import MdiPowershell from "~icons/mdi/powershell";
  import MdiRefresh from "~icons/mdi/refresh";
  import MdiRestart from "~icons/mdi/restart";
  import MdiRestartAlert from "~icons/mdi/restart-alert";
  import MdiRestore from "~icons/mdi/restore";
  import MdiSecurity from "~icons/mdi/security";
  import MdiServer from "~icons/mdi/server";
  import MdiSettingsOutline from "~icons/mdi/settings-outline";
  import MdiShieldAccountOutline from "~icons/mdi/shield-account-outline";
  import MdiShieldAlert from "~icons/mdi/shield-alert";
  import MdiShieldAlertOutline from "~icons/mdi/shield-alert-outline";
  import MdiShieldCheck from "~icons/mdi/shield-check";
  import MdiShieldKey from "~icons/mdi/shield-key";
  import MdiShieldLock from "~icons/mdi/shield-lock";
  import MdiSpeedometer from "~icons/mdi/speedometer";
  import MdiTextSearch from "~icons/mdi/text-search";
  import MdiTimerOutline from "~icons/mdi/timer-outline";
  import MdiToggleSwitch from "~icons/mdi/toggle-switch";
  import MdiToggleSwitchOffOutline from "~icons/mdi/toggle-switch-off-outline";
  import MdiToggleSwitchVariantOff from "~icons/mdi/toggle-switch-variant-off";
  import MdiTuneVariant from "~icons/mdi/tune-variant";
  import MdiTuneVertical from "~icons/mdi/tune-vertical";
  import MdiUndoVariant from "~icons/mdi/undo-variant";
  import MdiUpdate from "~icons/mdi/update";
  import MdiViewDashboard from "~icons/mdi/view-dashboard";
  import MdiWeb from "~icons/mdi/web";
  import RiRamLine from "~icons/ri/ram-line";

  // Tabler Icons
  import TablerAppWindow from "~icons/tabler/app-window";
  import TablerArrowBarToDown from "~icons/tabler/arrow-bar-to-down";
  import TablerBug from "~icons/tabler/bug";
  import TablerCopy from "~icons/tabler/copy";
  import TablerFileSearch from "~icons/tabler/file-search";
  import TablerLoader2 from "~icons/tabler/loader-2";
  import TablerMinus from "~icons/tabler/minus";
  import TablerMoon from "~icons/tabler/moon";
  import TablerShieldCheckFilled from "~icons/tabler/shield-check-filled";
  import TablerShieldUp from "~icons/tabler/shield-up";
  import TablerShieldX from "~icons/tabler/shield-x";
  import TablerSun from "~icons/tabler/sun";
  import TablerTrash from "~icons/tabler/trash";
  import TablerX from "~icons/tabler/x";

  // Fluent Icons
  import FluentMaximize20Filled from "~icons/fluent/maximize-20-filled";

  // Icon registry mapping icon names to components
  const iconRegistry: Record<string, typeof MdiLoading> = {
    // MDI icons
    "mdi:loading": MdiLoading,
    "mdi:alert-circle": MdiAlertCircle,
    "mdi:refresh": MdiRefresh,
    "mdi:magic-staff": MdiMagicStaff,
    "mdi:tune-vertical": MdiTuneVertical,
    "mdi:check-circle": MdiCheckCircle,
    "mdi:circle": MdiCircle,
    "mdi:circle-half-full": MdiCircleHalfFull,
    "mdi:circle-outline": MdiCircleOutline,
    "mdi:check": MdiCheck,
    "mdi:close": MdiClose,
    "mdi:cog-outline": MdiCogOutline,
    "mdi:console": MdiConsole,
    "mdi:shield-account-outline": MdiShieldAccountOutline,
    "mdi:shield-lock": MdiShieldLock,
    "mdi:shield-key": MdiShieldKey,
    "mdi:powershell": MdiPowershell,
    "mdi:restart": MdiRestart,
    "mdi:restart-alert": MdiRestartAlert,
    "mdi:information-outline": MdiInformationOutline,
    "mdi:database-cog-outline": MdiDatabaseCogOutline,
    "mdi:settings-outline": MdiSettingsOutline,
    "mdi:key-variant": MdiKeyVariant,
    "mdi:chevron-up": MdiChevronUp,
    "mdi:chevron-down": MdiChevronDown,
    "mdi:chevron-right": MdiChevronRight,
    "mdi:magnify": MdiMagnify,
    "mdi:microsoft-windows": MdiMicrosoftWindows,
    "mdi:update": MdiUpdate,
    "mdi:account": MdiAccount,
    "mdi:shield-check": MdiShieldCheck,
    "mdi:shield-alert": MdiShieldAlert,
    "mdi:shield-alert-outline": MdiShieldAlertOutline,
    "mdi:text-search": MdiTextSearch,
    "mdi:tune-variant": MdiTuneVariant,
    "mdi:cpu-64-bit": MdiCpu64Bit,
    "mdi:expansion-card": MdiExpansionCard,
    "mdi:timer-outline": MdiTimerOutline,
    "mdi:toggle-switch": MdiToggleSwitch,
    "mdi:toggle-switch-off-outline": MdiToggleSwitchOffOutline,
    "mdi:toggle-switch-variant-off": MdiToggleSwitchVariantOff,
    "mdi:laptop": MdiLaptop,
    "mdi:desktop-tower-monitor": MdiDesktopTowerMonitor,
    "mdi:harddisk": MdiHarddisk,
    "mdi:harddisk-plus": MdiHarddiskPlus,
    "mdi:ethernet": MdiEthernet,
    "mdi:memory": MdiMemory,
    "mdi:server": MdiServer,
    "mdi:folder": MdiFolder,
    "mdi:check-all": MdiCheckAll,
    "mdi:undo-variant": MdiUndoVariant,
    "mdi:file-search-outline": MdiFileSearchOutline,
    "mdi:package-variant": MdiPackageVariant,
    "mdi:close-circle-outline": MdiCloseCircleOutline,
    "mdi:restore": MdiRestore,
    "mdi:lightbulb-outline": MdiLightbulbOutline,
    "mdi:pin": MdiPin,
    "mdi:pin-outline": MdiPinOutline,
    "mdi:help-circle": MdiHelpCircle,
    "mdi:alert": MdiAlert,
    "mdi:alert-octagon": MdiAlertOctagon,
    "mdi:arrow-up-circle": MdiArrowUpCircle,
    "mdi:calendar": MdiCalendar,
    "mdi:calendar-clock": MdiCalendarClock,
    "mdi:bug": MdiBug,
    "mdi:cog": MdiCog,
    "mdi:database-export": MdiDatabaseExport,
    "mdi:download": MdiDownload,
    "mdi:email": MdiEmail,
    "mdi:export": MdiExport,
    "mdi:file-document-multiple": MdiFileDocumentMultiple,
    "mdi:github": MdiGithub,
    "mdi:import": MdiImport,
    "mdi:information": MdiInformation,
    "mdi:license": MdiLicense,
    "mdi:open-in-new": MdiOpenInNew,
    "mdi:web": MdiWeb,
    "mdi:account-circle": MdiAccountCircle,
    "mdi:arrow-right-circle": MdiArrowRightCircle,
    "mdi:backup-restore": MdiBackupRestore,
    "mdi:clipboard-check-outline": MdiClipboardCheckOutline,
    "mdi:close-circle": MdiCloseCircle,
    "mdi:history": MdiHistory,

    // Category icons (used in sidebar/navigation)
    "mdi:view-dashboard": MdiViewDashboard,
    "mdi:security": MdiSecurity,
    "mdi:speedometer": MdiSpeedometer,
    "mdi:gamepad-variant": MdiGamepadVariant,
    "mdi:cogs": MdiCogs,
    "mdi:palette": MdiPalette,

    // Tabler icons
    "tabler:bug": TablerBug,
    "tabler:arrow-bar-to-down": TablerArrowBarToDown,
    "tabler:trash": TablerTrash,
    "tabler:x": TablerX,
    "tabler:file-search": TablerFileSearch,
    "tabler:app-window": TablerAppWindow,
    "tabler:shield-check-filled": TablerShieldCheckFilled,
    "tabler:shield-x": TablerShieldX,
    "tabler:shield-up": TablerShieldUp,
    "tabler:loader-2": TablerLoader2,
    "tabler:moon": TablerMoon,
    "tabler:sun": TablerSun,
    "tabler:copy": TablerCopy,
    "tabler:minus": TablerMinus,

    // Fluent icons
    "fluent:minimize-20-filled": MdiMinimize20Filled,
    "fluent:maximize-20-filled": FluentMaximize20Filled,

    // Bi icons
    "bi:motherboard": BiMotherboard,

    // Ri icons
    "ri:ram-line": RiRamLine,
  };

  export function getIconComponent(name: string) {
    return iconRegistry[name];
  }
</script>

<script lang="ts">
  interface Props {
    icon: string;
    width?: number | string;
    height?: number | string;
    class?: string;
  }

  const { icon, width = 24, height, class: className = "" }: Props = $props();

  const IconComponent = $derived(iconRegistry[icon]);
  const computedHeight = $derived(height ?? width);
</script>

{#if IconComponent}
  <IconComponent {width} height={computedHeight} class={className} />
{:else}
  <!-- Fallback: show nothing or a placeholder for unregistered icons -->
  <span
    class="inline-flex items-center justify-center {className}"
    style="width: {width}px; height: {computedHeight}px;"
  >
    ?
  </span>
{/if}
