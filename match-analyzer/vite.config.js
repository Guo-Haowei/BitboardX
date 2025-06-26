/* eslint-disable no-undef */
import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
  server: {
    port: 8001,
    fs: {
      allow: [
        path.resolve(__dirname, '../bitboard_x/pkg'),
        path.resolve(__dirname, 'src'),
      ]
    }
  }
});
