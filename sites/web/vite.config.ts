import { defineConfig, type PluginOption, type UserConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";
import { sveltekit } from "@sveltejs/kit/vite";

// MOCK_API=1 answers /api and /auth from dev/mock-api.ts instead of the Rust API on port 3002. The mock is loaded only
// then, and only for `vite dev` (the plugin applies to serve), so it never reaches a build.
const mock = process.env.MOCK_API === "1";

export default defineConfig(async (): Promise<UserConfig> => {
  const plugins: PluginOption[] = [tailwindcss(), sveltekit()];
  if (mock) plugins.push((await import("./dev/mock-api.ts")).mockApi());
  const api = "http://localhost:3002";
  return {
    plugins,
    server: { proxy: mock ? {} : { "/api": api, "/auth": api } },
  };
});
