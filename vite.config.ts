import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import { APP_CONFIG } from "./src/lib/config/app";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
	plugins: [sveltekit(), tailwindcss()],

	// Define the config as build-time constants
	define: {
		__APP_CONFIG__: JSON.stringify(APP_CONFIG)
	},

	// Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
	//
	// 1. prevent vite from obscuring rust errors
	clearScreen: false,
	// 2. tauri expects a fixed port, fail if that port is not available
	server: {
		port: 1420,
		strictPort: true,
		host: host || false,
		hmr: host
			? {
				protocol: "ws",
				host,
				port: 1421
			}
			: undefined,
		watch: {
			// 3. tell vite to ignore watching `src-tauri`
			ignored: ["**/src-tauri/**"]
		}
	},
	resolve: {
		alias: {
			"@": new URL("./src", import.meta.url).pathname
		}
	},
	optimizeDeps: {
		exclude: ["@tauri-apps/api"]
	}
}));
