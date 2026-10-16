import { defineConfig } from "vite";
import { fileURLToPath } from "node:url";

// Standalone Vite serves the test HTML without SvelteKit's route middleware.
export default defineConfig({
  root: fileURLToPath(new URL("..", import.meta.url)),
});
