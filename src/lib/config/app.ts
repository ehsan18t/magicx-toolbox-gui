export const APP_CONFIG = {
  // Theme settings
  theme: {
    default: "dark" as const,
    storageKey: "theme",
    followSystem: true,
  },
} as const;

export type AppConfig = typeof APP_CONFIG;
