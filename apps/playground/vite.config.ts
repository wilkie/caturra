import { resolve } from 'node:path';

import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// SharedArrayBuffer (blocking stdin — see specs/EXECUTION.md) requires
// cross-origin isolation in both dev and preview.
const isolationHeaders = {
  'Cross-Origin-Opener-Policy': 'same-origin',
  'Cross-Origin-Embedder-Policy': 'require-corp',
};

export default defineConfig({
  // `/` for dev, CI, and local preview; the GitHub Pages build sets
  // VITE_BASE=/caturra/ (project pages serve under the repo path).
  base: process.env.VITE_BASE ?? '/',
  plugins: [react()],
  build: {
    // Per-unit CSA level chunks are intentionally large and load on demand.
    chunkSizeWarningLimit: 1500,
    // Two pages: the playground, and `compat.html` — the Java compatibility page,
    // which runs its own proof programs in the same engine.
    rollupOptions: {
      input: {
        index: resolve(import.meta.dirname, 'index.html'),
        compat: resolve(import.meta.dirname, 'compat.html'),
      },
    },
  },
  server: {
    port: 5173,
    strictPort: true,
    headers: isolationHeaders,
  },
  preview: {
    port: 4173,
    strictPort: true,
    headers: isolationHeaders,
  },
});
