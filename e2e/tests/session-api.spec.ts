import { test, expect } from '@playwright/test';
import { SESSION_URL, CODE_RE, UUID_RE, createSession, getSession } from '../helpers/api';

test.describe('Sessions — create', () => {
  test('POST /sessions returns 201', async ({ request }) => {
    const res = await request.post(`${SESSION_URL}/sessions`, {
      data: { person_a_name: 'Anna', person_b_name: 'Ben', host_name: 'Host' },
    });
    expect(res.status()).toBe(201);
  });

  test('create returns a session_id (UUID) and a code', async ({ request }) => {
    const res = await request.post(`${SESSION_URL}/sessions`, {
      data: { person_a_name: 'Anna', person_b_name: 'Ben', host_name: 'Host' },
    });
    const json = await res.json();
    expect(json.session_id).toMatch(UUID_RE);
    expect(typeof json.code).toBe('string');
  });

  test('generated code is 6 uppercase alphanumerics', async ({ request }) => {
    const { code } = await createSession(request);
    expect(code).toMatch(CODE_RE);
  });

  test('two sessions get different codes', async ({ request }) => {
    const a = await createSession(request);
    const b = await createSession(request);
    expect(a.code).not.toBe(b.code);
  });

  test('missing person_a_name is rejected (400/422)', async ({ request }) => {
    const res = await request.post(`${SESSION_URL}/sessions`, {
      data: { person_b_name: 'Ben', host_name: 'Host' },
    });
    expect([400, 422]).toContain(res.status());
  });

  test('empty body is rejected (400/422)', async ({ request }) => {
    const res = await request.post(`${SESSION_URL}/sessions`, { data: {} });
    expect([400, 422]).toContain(res.status());
  });
});

test.describe('Sessions — read', () => {
  test('GET /sessions/{code} returns the session', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await getSession(request, code);
    expect(res.status()).toBe(200);
    expect((await res.json()).code).toBe(code);
  });

  test('a fresh session has status "lobby"', async ({ request }) => {
    const { code } = await createSession(request);
    const json = await (await getSession(request, code)).json();
    expect(json.status).toBe('lobby');
  });

  test('session echoes the couple names', async ({ request }) => {
    const { code } = await createSession(request, { personA: 'Clara', personB: 'David' });
    const json = await (await getSession(request, code)).json();
    expect(json.person_a_name).toBe('Clara');
    expect(json.person_b_name).toBe('David');
  });

  test('session response carries the expected fields', async ({ request }) => {
    const { code } = await createSession(request);
    const json = await (await getSession(request, code)).json();
    for (const f of ['id', 'code', 'status', 'person_a_name', 'person_b_name', 'created_at']) {
      expect(json).toHaveProperty(f);
    }
    expect(json).toHaveProperty('started_at'); // null in lobby
  });

  test('unknown code returns 404', async ({ request }) => {
    const res = await getSession(request, 'ZZZZZZ');
    expect(res.status()).toBe(404);
  });

  test('code lookup is case-insensitive (lowercase resolves)', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await getSession(request, code.toLowerCase());
    expect(res.status()).toBe(200);
    expect((await res.json()).code).toBe(code);
  });
});

test.describe('Sessions — list', () => {
  test('GET /sessions returns an array', async ({ request }) => {
    const res = await request.get(`${SESSION_URL}/sessions`);
    expect(res.status()).toBe(200);
    expect(Array.isArray(await res.json())).toBe(true);
  });

  test('list contains a freshly created session', async ({ request }) => {
    const { code } = await createSession(request);
    const list = await (await request.get(`${SESSION_URL}/sessions`)).json();
    expect(list.some((s: any) => s.code === code)).toBe(true);
  });

  test('list items have id, code and status', async ({ request }) => {
    await createSession(request);
    const list = await (await request.get(`${SESSION_URL}/sessions`)).json();
    expect(list.length).toBeGreaterThan(0);
    for (const f of ['id', 'code', 'status']) expect(list[0]).toHaveProperty(f);
  });
});

test.describe('Sessions — update', () => {
  test('PUT partial name update returns 204', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await request.put(`${SESSION_URL}/sessions/${code}`, {
      data: { person_a_name: 'Renamed' },
    });
    expect(res.status()).toBe(204);
  });

  test('update is reflected on subsequent GET', async ({ request }) => {
    const { code } = await createSession(request);
    await request.put(`${SESSION_URL}/sessions/${code}`, { data: { person_a_name: 'Zoe' } });
    const json = await (await getSession(request, code)).json();
    expect(json.person_a_name).toBe('Zoe');
  });

  test('update with malformed code returns 400', async ({ request }) => {
    const res = await request.put(`${SESSION_URL}/sessions/abc`, {
      data: { person_a_name: 'X' },
    });
    expect(res.status()).toBe(400);
  });

  test('update of unknown (valid) code returns 404', async ({ request }) => {
    const res = await request.put(`${SESSION_URL}/sessions/ZZZZZZ`, {
      data: { person_a_name: 'X' },
    });
    expect(res.status()).toBe(404);
  });

  test('updating only host_name returns 204 and keeps session readable', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await request.put(`${SESSION_URL}/sessions/${code}`, {
      data: { host_name: 'New Host' },
    });
    expect(res.status()).toBe(204);
    expect((await getSession(request, code)).status()).toBe(200);
  });
});

test.describe('Sessions — start', () => {
  test('POST /sessions/{code}/start returns 204', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await request.post(`${SESSION_URL}/sessions/${code}/start`, { data: {} });
    expect(res.status()).toBe(204);
  });

  test('after start the session leaves the lobby state', async ({ request }) => {
    const { code } = await createSession(request);
    await request.post(`${SESSION_URL}/sessions/${code}/start`, { data: {} });
    const json = await (await getSession(request, code)).json();
    expect(json.status).not.toBe('lobby');
  });

  test('starting an unknown code returns 404', async ({ request }) => {
    const res = await request.post(`${SESSION_URL}/sessions/ZZZZZZ/start`, { data: {} });
    expect(res.status()).toBe(404);
  });
});
