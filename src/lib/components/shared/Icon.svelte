<script lang="ts" module>
  // Pre-import all icons used in the app for build-time bundling
  // MDI Icons
  import BiMotherboard from "~icons/bi/motherboard";
  import MdiAccount from "~icons/mdi/account";
  import MdiAccountCircle from "~icons/mdi/account-circle";
  import MdiAlert from "~icons/mdi/alert";
  import MdiAlertCircle from "~icons/mdi/alert-circle";
  import MdiAlertCircleOutline from "~icons/mdi/alert-circle-outline";
  import MdiAlertOctagon from "~icons/mdi/alert-octagon";
  import MdiFilterVariant from "~icons/mdi/filter-variant";
  import MdiFilterVariantRemove from "~icons/mdi/filter-variant-remove";
  import MdiArrowLeft from "~icons/mdi/arrow-left";
  import MdiArrowRight from "~icons/mdi/arrow-right";
  import MdiArrowRightCircle from "~icons/mdi/arrow-right-circle";
  import MdiArrowUpCircle from "~icons/mdi/arrow-up-circle";
  import MdiBackupRestore from "~icons/mdi/backup-restore";
  import MdiBroom from "~icons/mdi/broom";
  import MdiCalendar from "~icons/mdi/calendar";
  import MdiCalendarClock from "~icons/mdi/calendar-clock";
  import MdiCancel from "~icons/mdi/cancel";
  import MdiChartBar from "~icons/mdi/chart-bar";
  import MdiCheck from "~icons/mdi/check";
  import MdiCheckAll from "~icons/mdi/check-all";
  import MdiCheckCircle from "~icons/mdi/check-circle";
  import MdiCheckCircleOutline from "~icons/mdi/check-circle-outline";
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
  import MdiContentCopy from "~icons/mdi/content-copy";
  import MdiCpu64Bit from "~icons/mdi/cpu-64-bit";
  import MdiDatabase from "~icons/mdi/database";
  import MdiDatabaseCogOutline from "~icons/mdi/database-cog-outline";
  import MdiDatabaseExport from "~icons/mdi/database-export";
  import MdiDelete from "~icons/mdi/delete";
  import MdiDeleteOutline from "~icons/mdi/delete-outline";
  import MdiDesktopTowerMonitor from "~icons/mdi/desktop-tower-monitor";
  import MdiDownload from "~icons/mdi/download";
  import MdiEmail from "~icons/mdi/email";
  import MdiEthernet from "~icons/mdi/ethernet";
  import MdiExpansionCard from "~icons/mdi/expansion-card";
  import MdiExport from "~icons/mdi/export";
  import MdiEyeOffOutline from "~icons/mdi/eye-off-outline";
  import MdiFileCog from "~icons/mdi/file-cog";
  import MdiFileDocument from "~icons/mdi/file-document";
  import MdiFileDocumentMultiple from "~icons/mdi/file-document-multiple";
  import MdiFileImport from "~icons/mdi/file-import";
  import MdiFileMultiple from "~icons/mdi/file-multiple";
  import MdiFileSearchOutline from "~icons/mdi/file-search-outline";
  import MdiFlaskOutline from "~icons/mdi/flask-outline";
  import MdiFolder from "~icons/mdi/folder";
  import MdiFolderOpen from "~icons/mdi/folder-open";
  import MdiFolderOutline from "~icons/mdi/folder-outline";
  import MdiGamepadVariant from "~icons/mdi/gamepad-variant";
  import MdiGithub from "~icons/mdi/github";
  import MdiHarddisk from "~icons/mdi/harddisk";
  import MdiHarddiskPlus from "~icons/mdi/harddisk-plus";
  import MdiHammerWrench from "~icons/mdi/hammer-wrench";
  import MdiHelpCircle from "~icons/mdi/help-circle";
  import MdiHelpCircleOutline from "~icons/mdi/help-circle-outline";
  import MdiHistory from "~icons/mdi/history";
  import MdiImport from "~icons/mdi/import";
  import MdiInformation from "~icons/mdi/information";
  import MdiInformationOutline from "~icons/mdi/information-outline";
  import MdiKeyVariant from "~icons/mdi/key-variant";
  import MdiLaptop from "~icons/mdi/laptop";
  import MdiLightbulbOutline from "~icons/mdi/lightbulb-outline";
  import MdiLan from "~icons/mdi/lan";
  import MdiLightningBolt from "~icons/mdi/lightning-bolt";
  import MdiLinkVariant from "~icons/mdi/link-variant";
  import MdiLoading from "~icons/mdi/loading";
  import MdiMagnify from "~icons/mdi/magnify";
  import MdiMemory from "~icons/mdi/memory";
  import MdiMonitor from "~icons/mdi/monitor";
  import MdiMicrosoftWindows from "~icons/mdi/microsoft-windows";
  import MdiOpenInNew from "~icons/mdi/open-in-new";
  import MdiPackageVariant from "~icons/mdi/package-variant";
  import MdiPalette from "~icons/mdi/palette";
  import MdiPlay from "~icons/mdi/play";
  import MdiPlus from "~icons/mdi/plus";
  import MdiPowershell from "~icons/mdi/powershell";
  import MdiRefresh from "~icons/mdi/refresh";
  import MdiRestart from "~icons/mdi/restart";
  import MdiRestartAlert from "~icons/mdi/restart-alert";
  import MdiRestore from "~icons/mdi/restore";
  import MdiRobotOutline from "~icons/mdi/robot-outline";
  import MdiSecurity from "~icons/mdi/security";
  import MdiServer from "~icons/mdi/server";
  import MdiShieldAccountOutline from "~icons/mdi/shield-account-outline";
  import MdiShieldAlert from "~icons/mdi/shield-alert";
  import MdiShieldCheck from "~icons/mdi/shield-check";
  import MdiShieldKey from "~icons/mdi/shield-key";
  import MdiShieldLock from "~icons/mdi/shield-lock";
  import MdiShieldLockOutline from "~icons/mdi/shield-lock-outline";
  import MdiSpeedometer from "~icons/mdi/speedometer";
  import MdiStar from "~icons/mdi/star";
  import MdiStarOff from "~icons/mdi/star-off";
  import MdiStarOutline from "~icons/mdi/star-outline";
  import MdiStop from "~icons/mdi/stop";
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
  import MdiWifi from "~icons/mdi/wifi";
  // Effect-detail icons (per-option change breakdown in the tweak details modal)
  import MdiArrowDownBold from "~icons/mdi/arrow-down-bold";
  import MdiArrowUpBold from "~icons/mdi/arrow-up-bold";
  import MdiFileDocumentOutline from "~icons/mdi/file-document-outline";
  import MdiMinusCircle from "~icons/mdi/minus-circle";
  import MdiPlusCircle from "~icons/mdi/plus-circle";
  import MdiShieldOutline from "~icons/mdi/shield-outline";
  import RiRamLine from "~icons/ri/ram-line";

  // Tabler Icons
  import TablerFileText from "~icons/tabler/file-text";
  import TablerMinus from "~icons/tabler/minus";
  import TablerMoon from "~icons/tabler/moon";
  import TablerSun from "~icons/tabler/sun";

  // Fluent Icons
  import FluentNavigation20Regular from "~icons/fluent/navigation-20-regular";
  import FluentShieldCheckmark16Filled from "~icons/fluent/shield-checkmark-16-filled";
  import FluentShieldError16Filled from "~icons/fluent/shield-error-16-filled";
  import FluentShieldKeyhole16Regular from "~icons/fluent/shield-keyhole-16-regular";
  import FluentBug20Regular from "~icons/fluent/bug-20-regular";
  import MdiShieldHalfFull from "~icons/mdi/shield-half-full";

  // Icon Park Twotone Icons
  import IconParkTwotoneNewComputer from "~icons/icon-park-twotone/new-computer";

  // Icon registry mapping icon names to components
  const iconRegistry: Record<string, typeof MdiLoading> = {
    // MDI icons
    "mdi:loading": MdiLoading,
    "mdi:alert-circle": MdiAlertCircle,
    "mdi:alert-circle-outline": MdiAlertCircleOutline,
    "mdi:refresh": MdiRefresh,
    "mdi:tune-vertical": MdiTuneVertical,
    "mdi:check-circle": MdiCheckCircle,
    "mdi:check-circle-outline": MdiCheckCircleOutline,
    "mdi:circle": MdiCircle,
    "mdi:circle-half-full": MdiCircleHalfFull,
    "mdi:circle-outline": MdiCircleOutline,
    "mdi:check": MdiCheck,
    "mdi:close": MdiClose,
    "mdi:cog-outline": MdiCogOutline,
    "mdi:broom": MdiBroom,
    "mdi:lan": MdiLan,
    "mdi:monitor": MdiMonitor,
    "mdi:filter-variant": MdiFilterVariant,
    "mdi:filter-variant-remove": MdiFilterVariantRemove,
    "mdi:console": MdiConsole,
    "mdi:database": MdiDatabase,
    "mdi:shield-account-outline": MdiShieldAccountOutline,
    "mdi:shield-lock": MdiShieldLock,
    "mdi:shield-key": MdiShieldKey,
    "mdi:powershell": MdiPowershell,
    "mdi:restart": MdiRestart,
    "mdi:restart-alert": MdiRestartAlert,
    "mdi:robot-outline": MdiRobotOutline,
    "mdi:information-outline": MdiInformationOutline,
    "mdi:database-cog-outline": MdiDatabaseCogOutline,
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
    "mdi:folder-open": MdiFolderOpen,
    "mdi:folder-outline": MdiFolderOutline,
    "mdi:hammer-wrench": MdiHammerWrench,
    "mdi:file-document": MdiFileDocument,
    "mdi:check-all": MdiCheckAll,
    "mdi:undo-variant": MdiUndoVariant,
    "mdi:file-search-outline": MdiFileSearchOutline,
    "mdi:package-variant": MdiPackageVariant,
    "mdi:close-circle-outline": MdiCloseCircleOutline,
    "mdi:restore": MdiRestore,
    "mdi:lightbulb-outline": MdiLightbulbOutline,
    "mdi:eye-off-outline": MdiEyeOffOutline,
    "mdi:help-circle": MdiHelpCircle,
    "mdi:chart-bar": MdiChartBar,
    "mdi:alert": MdiAlert,
    "mdi:alert-octagon": MdiAlertOctagon,
    "mdi:arrow-up-circle": MdiArrowUpCircle,
    "mdi:arrow-right": MdiArrowRight,
    "mdi:arrow-left": MdiArrowLeft,
    "mdi:arrow-down-bold": MdiArrowDownBold,
    "mdi:arrow-up-bold": MdiArrowUpBold,
    "mdi:file-document-outline": MdiFileDocumentOutline,
    "mdi:minus-circle": MdiMinusCircle,
    "mdi:plus-circle": MdiPlusCircle,
    "mdi:shield-outline": MdiShieldOutline,
    "mdi:calendar": MdiCalendar,
    "mdi:calendar-clock": MdiCalendarClock,
    "mdi:cog": MdiCog,
    "mdi:database-export": MdiDatabaseExport,
    "mdi:download": MdiDownload,
    "mdi:email": MdiEmail,
    "mdi:export": MdiExport,
    "mdi:file-document-multiple": MdiFileDocumentMultiple,
    "mdi:github": MdiGithub,
    "mdi:import": MdiImport,
    "mdi:information": MdiInformation,
    "mdi:open-in-new": MdiOpenInNew,
    "mdi:web": MdiWeb,
    "mdi:account-circle": MdiAccountCircle,
    "mdi:arrow-right-circle": MdiArrowRightCircle,
    "mdi:backup-restore": MdiBackupRestore,
    "mdi:clipboard-check-outline": MdiClipboardCheckOutline,
    "mdi:close-circle": MdiCloseCircle,
    "mdi:history": MdiHistory,
    "mdi:cancel": MdiCancel,
    "mdi:delete-outline": MdiDeleteOutline,
    "mdi:help-circle-outline": MdiHelpCircleOutline,
    "mdi:link-variant": MdiLinkVariant,
    "mdi:shield-lock-outline": MdiShieldLockOutline,

    // Category icons (used in sidebar/navigation)
    "mdi:view-dashboard": MdiViewDashboard,
    "mdi:security": MdiSecurity,
    "mdi:speedometer": MdiSpeedometer,
    "mdi:gamepad-variant": MdiGamepadVariant,
    "mdi:cogs": MdiCogs,
    "mdi:palette": MdiPalette,
    "mdi:star": MdiStar,
    "mdi:star-off": MdiStarOff,
    "mdi:star-outline": MdiStarOutline,

    // Tabler icons
    "tabler:moon": TablerMoon,
    "tabler:sun": TablerSun,
    "tabler:file-text": TablerFileText,
    "tabler:minus": TablerMinus,

    // Fluent icons
    "fluent:navigation-20-regular": FluentNavigation20Regular,
    "fluent:shield-checkmark-16-filled": FluentShieldCheckmark16Filled,
    "fluent:shield-error-16-filled": FluentShieldError16Filled,
    "fluent:shield-keyhole-16-regular": FluentShieldKeyhole16Regular,
    "fluent:bug-20-regular": FluentBug20Regular,
    "mdi:shield-half-full": MdiShieldHalfFull,

    // Bi icons
    "bi:motherboard": BiMotherboard,

    // Ri icons
    "ri:ram-line": RiRamLine,

    // Icon Park Twotone icons
    "icon-park-twotone:new-computer": IconParkTwotoneNewComputer,
    "mdi:file-import": MdiFileImport,
    "mdi:file-multiple": MdiFileMultiple,
    "mdi:file-cog": MdiFileCog,
    "mdi:play": MdiPlay,
    "mdi:stop": MdiStop,
    "mdi:flask-outline": MdiFlaskOutline,
    "mdi:content-copy": MdiContentCopy,
    "mdi:plus": MdiPlus,
    "mdi:delete": MdiDelete,
    "mdi:lightning-bolt": MdiLightningBolt,
    "mdi:wifi": MdiWifi,
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

  let { icon, width = 24, height, class: className = "" }: Props = $props();

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
