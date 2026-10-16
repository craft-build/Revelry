import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import pkg from "./package.json" with { type: "json" };

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  define: {
    // Shown in Settings → About. package.json, Cargo.toml and tauri.conf.json
    // are bumped together on release, so any of them is the app version.
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  server: {
    port: 1420,
    strictPort: true,
  },
});
