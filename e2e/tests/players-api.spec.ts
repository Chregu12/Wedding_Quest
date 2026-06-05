import { test, expect } from '@playwright/test';
import { SESSION_URL, UUID_RE, createSession, joinPlayer } from '../helpers/api';

test.describe('Players — join', () => {
  test('join returns 201 with player_id and session_id', async ({ request }) => {
    const { code, sessionId } = await createSession(request);
    const res = await request.post(`${SESSION_URL}/sessions/${code}/join`, {
      data: { display_name: 'Gast1' },
    });
    expect(res.status()).toBe(201);
    const json = await res.json();
    expect(json.player_id).toMatch(UUID_RE);
    expect(json.session_id).toBe(sessionId);
  });

  test('two distinct names can join', async ({ request }) => {
    const { code } = await createSession(request);
    const p1 = await joinPlayer(request, code, 'Gast1');
    const p2 = await joinPlayer(request, code, 'Gast2');
    expect(p1).not.toBe(p2);
  });

  test('duplicate display_name is rejected (400)', async ({ request }) => {
    const { code } = await createSession(request);
    await joinPlayer(request, code, 'Doppelt');
    const res = await request.post(`${SESSION_URL}/sessions/${code}/join`, {
      data: { display_name: 'Doppelt' },
    });
    expect(res.status()).toBe(400);
  });

  test('malformed code is rejected (400)', async ({ request }) => {
    const res = await request.post(`${SESSION_URL}/sessions/abc/join`, {
      data: { display_name: 'Gast' },
    });
    expect(res.status()).toBe(400);
  });

  test('unknown (valid) code returns 404', async ({ request }) => {
    const res = await request.post(`${SESSION_URL}/sessions/ZZZZZZ/join`, {
      data: { display_name: 'Gast' },
    });
    expect(res.status()).toBe(404);
  });

  test('joining is still accepted after the session has started (201)', async ({ request }) => {
    // This build does not lock the lobby on start — late guests can still join.
    const { code } = await createSession(request);
    await request.post(`${SESSION_URL}/sessions/${code}/start`, { data: {} });
    const res = await request.post(`${SESSION_URL}/sessions/${code}/join`, {
      data: { display_name: 'SpaeterGast' },
    });
    expect(res.status()).toBe(201);
  });

  test('empty display_name is rejected (400)', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await request.post(`${SESSION_URL}/sessions/${code}/join`, {
      data: { display_name: '' },
    });
    expect(res.status()).toBe(400);
  });

  test('join is idempotent enough: different names always get unique ids', async ({ request }) => {
    const { code } = await createSession(request);
    const ids = new Set<string>();
    for (const n of ['A', 'B', 'C']) ids.add(await joinPlayer(request, code, n));
    expect(ids.size).toBe(3);
  });
});

test.describe('Players — lobby', () => {
  test('GET lobby returns session_code, couple names and players array', async ({ request }) => {
    const { code } = await createSession(request, { personA: 'Mia', personB: 'Tom' });
    const json = await (await request.get(`${SESSION_URL}/sessions/${code}/players`)).json();
    expect(json.session_code).toBe(code);
    expect(json.person_a_name).toBe('Mia');
    expect(json.person_b_name).toBe('Tom');
    expect(Array.isArray(json.players)).toBe(true);
  });

  test('a fresh lobby has no players', async ({ request }) => {
    const { code } = await createSession(request);
    const json = await (await request.get(`${SESSION_URL}/sessions/${code}/players`)).json();
    expect(json.players.length).toBe(0);
  });

  test('joined players show up in the lobby', async ({ request }) => {
    const { code } = await createSession(request);
    await joinPlayer(request, code, 'Gast1');
    await joinPlayer(request, code, 'Gast2');
    const json = await (await request.get(`${SESSION_URL}/sessions/${code}/players`)).json();
    expect(json.players.length).toBe(2);
    expect(json.players.map((p: any) => p.display_name).sort()).toEqual(['Gast1', 'Gast2']);
  });

  test('lobby player entries carry the expected fields', async ({ request }) => {
    const { code } = await createSession(request);
    await joinPlayer(request, code, 'Gast1');
    const json = await (await request.get(`${SESSION_URL}/sessions/${code}/players`)).json();
    const p = json.players[0];
    for (const f of ['id', 'display_name', 'total_score', 'is_connected', 'joined_at']) {
      expect(p).toHaveProperty(f);
    }
  });

  test('a new player starts with total_score 0', async ({ request }) => {
    const { code } = await createSession(request);
    await joinPlayer(request, code, 'Gast1');
    const json = await (await request.get(`${SESSION_URL}/sessions/${code}/players`)).json();
    expect(json.players[0].total_score).toBe(0);
  });

  test('lobby for unknown code returns 404', async ({ request }) => {
    const res = await request.get(`${SESSION_URL}/sessions/ZZZZZZ/players`);
    expect(res.status()).toBe(404);
  });
});

test.describe('Players — clear', () => {
  test('DELETE players returns 204', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await request.delete(`${SESSION_URL}/sessions/${code}/players`);
    expect(res.status()).toBe(204);
  });

  test('clearing removes all players from the lobby', async ({ request }) => {
    const { code } = await createSession(request);
    await joinPlayer(request, code, 'Gast1');
    await joinPlayer(request, code, 'Gast2');
    await request.delete(`${SESSION_URL}/sessions/${code}/players`);
    const json = await (await request.get(`${SESSION_URL}/sessions/${code}/players`)).json();
    expect(json.players.length).toBe(0);
  });

  test('the same name can re-join after a clear', async ({ request }) => {
    const { code } = await createSession(request);
    await joinPlayer(request, code, 'Gast1');
    await request.delete(`${SESSION_URL}/sessions/${code}/players`);
    const res = await request.post(`${SESSION_URL}/sessions/${code}/join`, {
      data: { display_name: 'Gast1' },
    });
    expect(res.status()).toBe(201);
  });
});
