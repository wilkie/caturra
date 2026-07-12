import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  outputDir: './e2e/test-results',
  fullyParallel: true,
  forbidOnly: !!process.env['CI'],
  retries: process.env['CI'] ? 2 : 0,
  reporter: [['html', { outputFolder: 'e2e/playwright-report', open: 'never' }], ['list']],
  use: {
    baseURL: 'http://localhost:4173',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: [
    {
      // Editor origin. Build once, serve the production bundle; requires
      // `pnpm build:wasm` to have produced packages/core/src/wasm/generated.
      command:
        'pnpm --filter @caturra/playground build && pnpm --filter @caturra/playground preview',
      url: 'http://localhost:4173',
      reuseExistingServer: !process.env['CI'],
      timeout: 120_000,
    },
    {
      // Sandbox origin on a different host (127.0.0.1 vs localhost) so the
      // iframe is cross-origin. Allow the editor origin, and serve the real
      // COOP/COEP/CORP headers (from apps/sandbox/vite.config.ts) that the
      // cross-origin isolated iframe needs. Only sandbox.spec.ts uses it.
      command:
        'VITE_EDITOR_ORIGINS=http://localhost:4173 pnpm --filter @caturra/sandbox build && ' +
        'pnpm --filter @caturra/sandbox preview --host 127.0.0.1',
      url: 'http://127.0.0.1:4174',
      reuseExistingServer: !process.env['CI'],
      timeout: 120_000,
    },
  ],
});
