import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright E2E configuration for MiControl.
 *
 * Tests run against the Vite mock dev server (port 1421) which provides
 * mock Tauri API implementations (src/mocks/tauri-api.ts), allowing
 * browser-only testing without the full Tauri runtime.
 *
 * @see vite.config.mock.ts for mock server configuration
 * @see src/mocks/tauri-api.ts for mocked Tauri commands
 */
export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:1421',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: 'npm run dev:mock',
    url: 'http://localhost:1421',
    reuseExistingServer: !process.env.CI,
    timeout: 30_000,
  },
});
