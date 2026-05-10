import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  timeout: 180000,
  expect: {
    timeout: 120000,
  },
  use: {
    headless: true,
    baseURL: 'http://127.0.0.1:4173',
  },
  webServer: {
    command: 'npm run dev',
    url: 'http://127.0.0.1:4173',
    timeout: 120000,
    reuseExistingServer: !process.env.CI,
  },
});
