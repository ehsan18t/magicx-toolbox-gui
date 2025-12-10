// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    // interface PageData {}
    // interface PageState {}
    // interface Platform {}
  }

  // Build-time constants from vite.config.ts
  declare const __APP_CONFIG__: import("./lib/config/app").AppConfig;
}

export {};
