/* eslint-disable no-undef */
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  base: "/pages/chess/",

  plugins: [react()],

  server: {
    host: true,
    port: 8000,

    fs: {
      allow: [path.resolve(__dirname), path.resolve(__dirname, "../pkg")],
    },
  },
});
