import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [tailwindcss()],

  // Vite options tailored for Tauri development
  //
  // 1. prevent Vite from obscuring Rust errors
  clearScreen: false,
  // 2. Tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    // Tauri uses WebView2 (Chromium) on Windows, WKWebView (Safari) on macOS
    target: "chrome110",
    minify: "esbuild",
    sourcemap: false,
    outDir: "dist",
    rollupOptions: { input: { main: "index.html", feedback: "feedback.html" } },
  },
});
