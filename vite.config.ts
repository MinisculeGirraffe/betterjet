import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";

export default defineConfig({
  plugins: [react(), ],
  // prevent vite from obscuring rust errors
  clearScreen: false,
  server: {
    host: "10.1.20.70", // listen on all addresses
    port: 5173,
    // Tauri expects a fixed port, fail if that port is not available
    strictPort: true,
    hmr: {
      protocol: "ws",
      host: "10.1.20.70",
      port: 5183,
    },
  },
  // to make use of `TAURI_PLATFORM`, `TAURI_ARCH`, `TAURI_FAMILY`,
  // `TAURI_PLATFORM_VERSION`, `TAURI_PLATFORM_TYPE` and `TAURI_DEBUG`
  // env variables
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    // Tauri supports es2021
    target: ["es2021", "chrome100", "safari13"],
    // don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
