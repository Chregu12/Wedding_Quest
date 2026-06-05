import { test, expect } from '@playwright/test';
import { SESSION_URL, UUID_RE, createSession } from '../helpers/api';

function putConfig(request: any, code: string, data: any) {
  return request.put(`${SESSION_URL}/sessions/${code}/config`, { data });
}

test.describe('Score config', () => {
  test('PUT config returns 200 with session_id and all fields', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await putConfig(request, code, { base_points: 150 });
    expect(res.status()).toBe(200);
    const json = await res.json();
    expect(json.session_id).toMatch(UUID_RE);
    for (const f of [
      'tier1_max_seconds',
      'tier2_max_seconds',
      'tier1_multiplier',
      'tier2_multiplier',
      'tier3_multiplier',
      'perfect_match_multiplier',
      'catchup_multiplier',
      'catchup_threshold_percent',
      'base_points',
    ]) {
      expect(json).toHaveProperty(f);
    }
  });

  test('base_points partial update is reflected', async ({ request }) => {
    const { code } = await createSession(request);
    const json = await (await putConfig(request, code, { base_points: 250 })).json();
    expect(json.base_points).toBe(250);
  });

  test('float multipliers are stored', async ({ request }) => {
    const { code } = await createSession(request);
    const json = await (await putConfig(request, code, { tier1_multiplier: 5.5 })).json();
    expect(json.tier1_multiplier).toBe(5.5);
  });

  test('integer second/threshold fields are stored', async ({ request }) => {
    const { code } = await createSession(request);
    const json = await (
      await putConfig(request, code, { tier1_max_seconds: 7, catchup_threshold_percent: 40 })
    ).json();
    expect(json.tier1_max_seconds).toBe(7);
    expect(json.catchup_threshold_percent).toBe(40);
  });

  test('multiple fields update in one call', async ({ request }) => {
    const { code } = await createSession(request);
    const json = await (
      await putConfig(request, code, { base_points: 80, tier2_multiplier: 2.5, tier2_max_seconds: 25 })
    ).json();
    expect(json.base_points).toBe(80);
    expect(json.tier2_multiplier).toBe(2.5);
    expect(json.tier2_max_seconds).toBe(25);
  });

  test('unspecified fields keep their defaults', async ({ request }) => {
    const { code } = await createSession(request);
    const json = await (await putConfig(request, code, { base_points: 999 })).json();
    // defaults remain for everything not set
    expect(json.tier1_max_seconds).toBe(10);
    expect(json.tier1_multiplier).toBe(3.0);
    expect(json.perfect_match_multiplier).toBe(2.0);
  });

  test('config update with a malformed code returns 400', async ({ request }) => {
    const res = await putConfig(request, 'abc', { base_points: 100 });
    expect(res.status()).toBe(400);
  });

  test('config update for an unknown (valid) code returns 404', async ({ request }) => {
    const res = await putConfig(request, 'ZZZZZZ', { base_points: 100 });
    expect(res.status()).toBe(404);
  });
});
