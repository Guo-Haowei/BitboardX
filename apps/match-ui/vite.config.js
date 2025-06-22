import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
  server: {
    fs: {
      allow: [
        // eslint-disable-next-line no-undef
        path.resolve(__dirname, '../../pkg'),
        // eslint-disable-next-line no-undef
        path.resolve(__dirname, 'src'),
      ]
    }
  }
});
