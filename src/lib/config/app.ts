export const APP_CONFIG = {
  appName: "MagicX Toolbox",
  appIcon: "/icons/Toolbox.ico",
  githubRepo: "https://github.com/ehsan18t/magicx-toolbox-gui",
  author: {
    name: "Ehsan Khan",
    github: "https://github.com/ehsan18t",
    website: "https://ehsankhan.me",
    email: "ehsan18t@gmail.com",
  },
  update: {
    // Matches MagicX-Toolbox_x.x.x_x64-setup.exe and MagicX-Toolbox_x.x.x_x64_en-US.msi.
    assetPattern: /MagicX[-_]Toolbox.*x64.*\.(exe|msi)$/i,
    releasesApiUrl: "https://api.github.com/repos/ehsan18t/magicx-toolbox-gui/releases/latest",
  },
} as const;

// app.html reads `theme` and `colorScheme` before the bundle loads: rename them there too.
export const STORAGE_KEYS = {
  theme: "theme",
  colorScheme: "magicx-color-scheme",
  settings: "magicx-app-settings",
  favorites: "magicx-favorites",
  navCollapsed: "magicx-nav-collapsed",
  // Persisted under this spelling already, so it keeps its underscores.
  profileDir: "magicx_profile_dir",
  systemInfoCache: "magicx-system-info-cache",
} as const;

export const RETIRED_STORAGE_KEYS = [
  "magicx-debug-mode",
  "magicx-sidebar-pinned",
  "magicx-sidebar-widgets-open",
] as const;

/** Profile file extension, without the dot. */
export const PROFILE_EXT = "mgx";
