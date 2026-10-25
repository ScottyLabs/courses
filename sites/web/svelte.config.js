import adapter from "@sveltejs/adapter-static";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  compilerOptions: {
    runes: true,
  },
  kit: {
    adapter: adapter({
      pages: "build",
      assets: "build",
      fallback: "index.html",
      precompress: false,
      strict: true,
    }),
    alias: {
      "@": "./src",
    },
    // Ask every five minutes whether a new build is out, so an open tab can offer to reload (routes/+layout.svelte).
    version: {
      pollInterval: 300_000,
    },
  },
};

export default config;
