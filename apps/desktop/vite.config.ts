import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Dev: `cargo run -p lwe-server` serves the API on 4747; Vite proxies /api to it.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 5173, strictPort: true, proxy: { "/api": "http://127.0.0.1:4747" } },
  build: { target: "es2021", outDir: "dist" },
});
