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
