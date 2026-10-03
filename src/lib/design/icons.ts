import type { Component } from "svelte";
import type { SVGAttributes } from "svelte/elements";
import BiMotherboard from "~icons/bi/motherboard";
import FluentBug20Regular from "~icons/fluent/bug-20-regular";
import FluentNavigation20Regular from "~icons/fluent/navigation-20-regular";
import FluentShieldCheckmark16Filled from "~icons/fluent/shield-checkmark-16-filled";
import FluentShieldError16Filled from "~icons/fluent/shield-error-16-filled";
import FluentShieldKeyhole16Regular from "~icons/fluent/shield-keyhole-16-regular";
import MdiAccount from "~icons/mdi/account";
import MdiAlert from "~icons/mdi/alert";
import MdiAlertCircle from "~icons/mdi/alert-circle";
import MdiAlertCircleOutline from "~icons/mdi/alert-circle-outline";
import MdiAlertOctagon from "~icons/mdi/alert-octagon";
import MdiArrowLeft from "~icons/mdi/arrow-left";
import MdiArrowRight from "~icons/mdi/arrow-right";
import MdiBackupRestore from "~icons/mdi/backup-restore";
import MdiBroom from "~icons/mdi/broom";
import MdiCalendar from "~icons/mdi/calendar";
import MdiCancel from "~icons/mdi/cancel";
import MdiCheck from "~icons/mdi/check";
import MdiCheckCircle from "~icons/mdi/check-circle";
import MdiCheckCircleOutline from "~icons/mdi/check-circle-outline";
import MdiChevronDown from "~icons/mdi/chevron-down";
import MdiChevronRight from "~icons/mdi/chevron-right";
import MdiChevronUp from "~icons/mdi/chevron-up";
import MdiCircleOutline from "~icons/mdi/circle-outline";
import MdiClose from "~icons/mdi/close";
import MdiCloseCircle from "~icons/mdi/close-circle";
import MdiCog from "~icons/mdi/cog";
import MdiCogOutline from "~icons/mdi/cog-outline";
import MdiConsole from "~icons/mdi/console";
import MdiContentCopy from "~icons/mdi/content-copy";
import MdiCpu64Bit from "~icons/mdi/cpu-64-bit";
import MdiDatabase from "~icons/mdi/database";
import MdiDelete from "~icons/mdi/delete";
import MdiDeleteOutline from "~icons/mdi/delete-outline";
import MdiDesktopTowerMonitor from "~icons/mdi/desktop-tower-monitor";
import MdiDownload from "~icons/mdi/download";
import MdiEmail from "~icons/mdi/email";
import MdiEthernet from "~icons/mdi/ethernet";
import MdiExpansionCard from "~icons/mdi/expansion-card";
import MdiExport from "~icons/mdi/export";
import MdiFileCog from "~icons/mdi/file-cog";
import MdiFileDocument from "~icons/mdi/file-document";
import MdiFileDocumentOutline from "~icons/mdi/file-document-outline";
import MdiFileImport from "~icons/mdi/file-import";
import MdiFileMultiple from "~icons/mdi/file-multiple";
import MdiFileSearchOutline from "~icons/mdi/file-search-outline";
import MdiFilterVariant from "~icons/mdi/filter-variant";
import MdiFilterVariantRemove from "~icons/mdi/filter-variant-remove";
import MdiFlaskOutline from "~icons/mdi/flask-outline";
import MdiFolder from "~icons/mdi/folder";
import MdiFolderOpen from "~icons/mdi/folder-open";
import MdiFolderOutline from "~icons/mdi/folder-outline";
import MdiGithub from "~icons/mdi/github";
import MdiHammerWrench from "~icons/mdi/hammer-wrench";
import MdiHarddisk from "~icons/mdi/harddisk";
import MdiHarddiskPlus from "~icons/mdi/harddisk-plus";
import MdiHelpCircle from "~icons/mdi/help-circle";
import MdiHelpCircleOutline from "~icons/mdi/help-circle-outline";
import MdiHistory from "~icons/mdi/history";
import MdiImport from "~icons/mdi/import";
import MdiInformation from "~icons/mdi/information";
import MdiInformationOutline from "~icons/mdi/information-outline";
import MdiLan from "~icons/mdi/lan";
import MdiLaptop from "~icons/mdi/laptop";
import MdiLinkVariant from "~icons/mdi/link-variant";
import MdiLoading from "~icons/mdi/loading";
import MdiMagnify from "~icons/mdi/magnify";
import MdiMicrosoftWindows from "~icons/mdi/microsoft-windows";
import MdiMonitor from "~icons/mdi/monitor";
import MdiOpenInNew from "~icons/mdi/open-in-new";
import MdiPackageVariant from "~icons/mdi/package-variant";
import MdiPlay from "~icons/mdi/play";
import MdiPlus from "~icons/mdi/plus";
import MdiRefresh from "~icons/mdi/refresh";
import MdiRestart from "~icons/mdi/restart";
import MdiRobotOutline from "~icons/mdi/robot-outline";
import MdiServer from "~icons/mdi/server";
import MdiShieldAccountOutline from "~icons/mdi/shield-account-outline";
import MdiShieldCheck from "~icons/mdi/shield-check";
import MdiShieldHalfFull from "~icons/mdi/shield-half-full";
import MdiShieldKey from "~icons/mdi/shield-key";
import MdiShieldLock from "~icons/mdi/shield-lock";
import MdiShieldLockOutline from "~icons/mdi/shield-lock-outline";
import MdiShieldOutline from "~icons/mdi/shield-outline";
import MdiSpeedometer from "~icons/mdi/speedometer";
import MdiStar from "~icons/mdi/star";
import MdiStarOff from "~icons/mdi/star-off";
import MdiStarOutline from "~icons/mdi/star-outline";
import MdiStop from "~icons/mdi/stop";
import MdiTextSearch from "~icons/mdi/text-search";
import MdiTimerOutline from "~icons/mdi/timer-outline";
import MdiTuneVariant from "~icons/mdi/tune-variant";
import MdiUndoVariant from "~icons/mdi/undo-variant";
import MdiUpdate from "~icons/mdi/update";
import MdiViewDashboard from "~icons/mdi/view-dashboard";
import MdiWeb from "~icons/mdi/web";
import RiRamLine from "~icons/ri/ram-line";
import TablerFileText from "~icons/tabler/file-text";
import TablerMoon from "~icons/tabler/moon";
import TablerSun from "~icons/tabler/sun";

// Imported one by one so the build bundles only these; a tweak YAML `icon:` must be one of them.
export const iconRegistry = {
  "bi:motherboard": BiMotherboard,
  "fluent:bug-20-regular": FluentBug20Regular,
  "fluent:navigation-20-regular": FluentNavigation20Regular,
  "fluent:shield-checkmark-16-filled": FluentShieldCheckmark16Filled,
  "fluent:shield-error-16-filled": FluentShieldError16Filled,
  "fluent:shield-keyhole-16-regular": FluentShieldKeyhole16Regular,
  "mdi:account": MdiAccount,
  "mdi:alert": MdiAlert,
  "mdi:alert-circle": MdiAlertCircle,
  "mdi:alert-circle-outline": MdiAlertCircleOutline,
  "mdi:alert-octagon": MdiAlertOctagon,
  "mdi:arrow-left": MdiArrowLeft,
  "mdi:arrow-right": MdiArrowRight,
  "mdi:backup-restore": MdiBackupRestore,
  "mdi:broom": MdiBroom,
  "mdi:calendar": MdiCalendar,
  "mdi:cancel": MdiCancel,
  "mdi:check": MdiCheck,
  "mdi:check-circle": MdiCheckCircle,
  "mdi:check-circle-outline": MdiCheckCircleOutline,
  "mdi:chevron-down": MdiChevronDown,
  "mdi:chevron-right": MdiChevronRight,
  "mdi:chevron-up": MdiChevronUp,
  "mdi:circle-outline": MdiCircleOutline,
  "mdi:close": MdiClose,
  "mdi:close-circle": MdiCloseCircle,
  "mdi:cog": MdiCog,
  "mdi:cog-outline": MdiCogOutline,
  "mdi:console": MdiConsole,
  "mdi:content-copy": MdiContentCopy,
  "mdi:cpu-64-bit": MdiCpu64Bit,
  "mdi:database": MdiDatabase,
  "mdi:delete": MdiDelete,
  "mdi:delete-outline": MdiDeleteOutline,
  "mdi:desktop-tower-monitor": MdiDesktopTowerMonitor,
  "mdi:download": MdiDownload,
  "mdi:email": MdiEmail,
  "mdi:ethernet": MdiEthernet,
  "mdi:expansion-card": MdiExpansionCard,
  "mdi:export": MdiExport,
  "mdi:file-cog": MdiFileCog,
  "mdi:file-document": MdiFileDocument,
  "mdi:file-document-outline": MdiFileDocumentOutline,
  "mdi:file-import": MdiFileImport,
  "mdi:file-multiple": MdiFileMultiple,
  "mdi:file-search-outline": MdiFileSearchOutline,
  "mdi:filter-variant": MdiFilterVariant,
  "mdi:filter-variant-remove": MdiFilterVariantRemove,
  "mdi:flask-outline": MdiFlaskOutline,
  "mdi:folder": MdiFolder,
  "mdi:folder-open": MdiFolderOpen,
  "mdi:folder-outline": MdiFolderOutline,
  "mdi:github": MdiGithub,
  "mdi:hammer-wrench": MdiHammerWrench,
  "mdi:harddisk": MdiHarddisk,
  "mdi:harddisk-plus": MdiHarddiskPlus,
  "mdi:help-circle": MdiHelpCircle,
  "mdi:help-circle-outline": MdiHelpCircleOutline,
  "mdi:history": MdiHistory,
  "mdi:import": MdiImport,
  "mdi:information": MdiInformation,
  "mdi:information-outline": MdiInformationOutline,
  "mdi:lan": MdiLan,
  "mdi:laptop": MdiLaptop,
  "mdi:link-variant": MdiLinkVariant,
  "mdi:loading": MdiLoading,
  "mdi:magnify": MdiMagnify,
  "mdi:microsoft-windows": MdiMicrosoftWindows,
  "mdi:monitor": MdiMonitor,
  "mdi:open-in-new": MdiOpenInNew,
  "mdi:package-variant": MdiPackageVariant,
  "mdi:play": MdiPlay,
  "mdi:plus": MdiPlus,
  "mdi:refresh": MdiRefresh,
  "mdi:restart": MdiRestart,
  "mdi:robot-outline": MdiRobotOutline,
  "mdi:server": MdiServer,
  "mdi:shield-account-outline": MdiShieldAccountOutline,
  "mdi:shield-check": MdiShieldCheck,
  "mdi:shield-half-full": MdiShieldHalfFull,
  "mdi:shield-key": MdiShieldKey,
  "mdi:shield-lock": MdiShieldLock,
  "mdi:shield-lock-outline": MdiShieldLockOutline,
  "mdi:shield-outline": MdiShieldOutline,
  "mdi:speedometer": MdiSpeedometer,
  "mdi:star": MdiStar,
  "mdi:star-off": MdiStarOff,
  "mdi:star-outline": MdiStarOutline,
  "mdi:stop": MdiStop,
  "mdi:text-search": MdiTextSearch,
  "mdi:timer-outline": MdiTimerOutline,
  "mdi:tune-variant": MdiTuneVariant,
  "mdi:undo-variant": MdiUndoVariant,
  "mdi:update": MdiUpdate,
  "mdi:view-dashboard": MdiViewDashboard,
  "mdi:web": MdiWeb,
  "ri:ram-line": RiRamLine,
  "tabler:file-text": TablerFileText,
  "tabler:moon": TablerMoon,
  "tabler:sun": TablerSun,
} satisfies Record<string, Component<SVGAttributes<SVGSVGElement>>>;

export type IconName = keyof typeof iconRegistry;

export const isIconName = (name: string): name is IconName => Object.hasOwn(iconRegistry, name);
