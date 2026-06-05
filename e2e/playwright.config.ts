import { defineConfig } from '@playwright/test';

/**
 * API-only e2e tests for the Wedding Quest backend services.
 *
 * No browser is launched — every test talks HTTP/WS directly to the services
 * via Playwright's `request` fixture (and the `ws` package for WebSockets).
 *
 * Default targets are the local dev stack (docker-compose.yml):
 *   session-service  -> http://localhost:3002
 *   engine-service   -> http://localhost:3003
 *   scoring-service  -> http://localhost:3004
 *   realtime-service -> http://localhost:3006  (ws://localhost:3006/ws/<code>)
 *
 * Override per service with WQ_SESSION_URL / WQ_ENGINE_URL / WQ_SCORING_URL /
 * WQ_REALTIME_URL / WQ_REALTIME_WS.
 */
export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  workers: process.env.CI ? 2 : 4,
  timeout: 30_000,
  expect: { timeout: 12_000 },
  reporter: process.env.CI ? [['list'], ['html', { open: 'never' }]] : [['list']],
  projects: [{ name: 'api' }],
});
