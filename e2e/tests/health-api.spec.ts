import { test, expect } from '@playwright/test';
import { SESSION_URL, ENGINE_URL, SCORING_URL, REALTIME_URL } from '../helpers/api';

test.describe('Health', () => {
  test('session-service /health returns 200 OK', async ({ request }) => {
    const res = await request.get(`${SESSION_URL}/health`);
    expect(res.status()).toBe(200);
    expect((await res.text()).trim()).toBe('OK');
  });

  test('engine-service /health returns 200 OK', async ({ request }) => {
    const res = await request.get(`${ENGINE_URL}/health`);
    expect(res.status()).toBe(200);
    expect((await res.text()).trim()).toBe('OK');
  });

  test('scoring-service /health returns 200 OK', async ({ request }) => {
    const res = await request.get(`${SCORING_URL}/health`);
    expect(res.status()).toBe(200);
    expect((await res.text()).trim()).toBe('OK');
  });

  test('realtime-service /health returns 200 with status ok', async ({ request }) => {
    const res = await request.get(`${REALTIME_URL}/health`);
    expect(res.status()).toBe(200);
    // realtime reports JSON health: {"service":"realtime-service","status":"ok"}
    expect((await res.json()).status).toBe('ok');
  });
});
