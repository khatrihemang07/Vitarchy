import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { fileURLToPath } from "node:url";
import { visualizer } from "rollup-plugin-visualizer";
import { loadEnv } from "vite";
import { defineConfig } from "vitest/config";

/**
 * Vendor libraries every page needs get their own long-lived chunks, so an app-code
 * change does not invalidate them. Anything else from node_modules (Recharts and its
 * d3 stack) stays with the lazy route that imports it.
 */
const VENDOR_CHUNKS: [chunk: string, packages: RegExp][] = [
  ["vendor-react", /node_modules\/(react|react-dom|scheduler)\//],
  ["vendor-tanstack", /node_modules\/@tanstack\//],
  ["vendor-radix", /node_modules\/(radix-ui|@radix-ui)\//],
  ["vendor-motion", /node_modules\/(motion|motion-dom|motion-utils|framer-motion)\//],
];

function vendorChunk(id: string): string | undefined {
  return VENDOR_CHUNKS.find(([, packages]) => packages.test(id))?.[0];
}

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  const port = Number(env.WEB_PORT) || 7420;
  // The API listens on BIND_ADDR (exported by `make dev` / `make stage`), so its port
  // is derived from there; API_PORT is only the fallback for a bare `pnpm dev`.
  const apiPort =
    Number(env.BIND_ADDR?.split(":").pop()) || Number(env.API_PORT) || 3000;

  return {
    plugins: [
      react(),
      tailwindcss(),
      // `ANALYZE=1 pnpm build` writes a size treemap to node_modules/.vite/bundle-stats.html.
      ...(env.ANALYZE === "1"
        ? [visualizer({ filename: "node_modules/.vite/bundle-stats.html", gzipSize: true })]
        : []),
    ],
    build: { rollupOptions: { output: { manualChunks: vendorChunk } } },
    resolve: {
      alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) },
    },
    server: {
      host: "127.0.0.1",
      port,
      strictPort: true,
      proxy: { "/api": `http://127.0.0.1:${String(apiPort)}` },
    },
    preview: { host: "127.0.0.1", port, strictPort: true },
    test: {
      environment: "jsdom",
      globals: true,
      setupFiles: ["./src/test-setup.ts"],
      globalSetup: ["./src/test-timezone.ts"],
    },
  };
});
