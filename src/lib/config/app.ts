export const APP_CONFIG = {
  // App info
  appName: "MagicX Toolbox",
  githubOwner: "ehsan18t",
  githubRepo: "https://github.com/ehsan18t/magicx-toolbox-gui",

  // Update settings
  update: {
    // Regex pattern to match the correct asset for this platform
    // Matches: MagicX-Toolbox_x.x.x_x64-setup.exe or MagicX-Toolbox_x.x.x_x64_en-US.msi
    assetPattern: /MagicX[-_]Toolbox.*x64.*\.(exe|msi)$/i,
    // GitHub API endpoint for releases
    releasesApiUrl: "https://api.github.com/repos/ehsan18t/magicx-toolbox-gui/releases/latest",
  },

  // Theme settings
  theme: {
    default: "dark" as const,
    storageKey: "theme",
    followSystem: true,
  },
} as const;

export type AppConfig = typeof APP_CONFIG;
