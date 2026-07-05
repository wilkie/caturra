import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// SharedArrayBuffer (blocking stdin — see specs/EXECUTION.md) requires
// cross-origin isolation in both dev and preview.
const isolationHeaders = {
  'Cross-Origin-Opener-Policy': 'same-origin',
  'Cross-Origin-Embedder-Policy': 'require-corp',
};

export default defineConfig({
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
