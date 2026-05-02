import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

export default defineConfig({
    plugins: [sveltekit()],
    server: {
        proxy: {
            "/catalog": "http://localhost:3002",
            "/health": "http://localhost:3002",
        },
    },
});
