import { defineConfig } from 'vite';

// The sandbox must be cross-origin isolated on its own (so its worker keeps
// SharedArrayBuffer for blocking stdin/Swing/debug) AND embeddable by the
// editor page, which itself sets COEP: require-corp. Production hosting must
// send these same three headers, plus a
// `Content-Security-Policy: frame-ancestors <editor-origin>` so only the
// editor may frame it. See specs/EXECUTION.md.
const isolationHeaders = {
  'Cross-Origin-Opener-Policy': 'same-origin',
  'Cross-Origin-Embedder-Policy': 'require-corp',
  'Cross-Origin-Resource-Policy': 'cross-origin',
};

export default defineConfig({
  base: process.env.VITE_BASE ?? '/',
  build: { chunkSizeWarningLimit: 1500 },
  server: {
    port: 5174,
    strictPort: true,
    headers: isolationHeaders,
  },
  preview: {
    port: 4174,
    strictPort: true,
    headers: isolationHeaders,
  },
});
