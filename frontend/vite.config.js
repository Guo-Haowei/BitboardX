/* eslint-disable no-undef */
import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
  base: '/pages/chess/',
  server: {
    host: true,
    port: 8000,
    fs: {
      allow: [
        path.resolve(__dirname, '../bitboard_x/pkg'),
        path.resolve(__dirname, 'src'),
      ]
    }
  }
});
