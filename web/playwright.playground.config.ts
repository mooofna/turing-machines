// Runs the playground tests on their own while the shared playwright.config.ts (web shell) is
// not merged yet: npx playwright test -c playwright.playground.config.ts
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: 'e2e',
  testMatch: 'playground.spec.ts',
  fullyParallel: true,
  reporter: 'list',
  // A port of its own, so that it never reuses the preview of another worktree.
  use: { baseURL: 'http://localhost:4183', trace: 'retain-on-failure' },
  webServer: {
    command: 'npx vite preview --port 4183 --strictPort',
    port: 4183,
    reuseExistingServer: false
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'], viewport: { width: 1440, height: 900 } }
    }
  ]
});
