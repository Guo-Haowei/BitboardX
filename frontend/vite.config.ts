import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
  base: '/pages/chess/',
  server: {
    fs: {
      allow: [
        path.resolve(__dirname, '../pkg'),
        path.resolve(__dirname, 'src'),
      ]
    }
  }
});
