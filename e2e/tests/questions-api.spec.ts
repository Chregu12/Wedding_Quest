import { test, expect } from '@playwright/test';
import { SESSION_URL, UUID_RE, createSession, addGuestQuiz, addIchOderDu } from '../helpers/api';

test.describe('Questions — guest_quiz', () => {
  test('adding a guest_quiz returns 201', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await request.post(`${SESSION_URL}/sessions/${code}/questions`, {
      data: { text: 'Q?', option_a: 'A', option_b: 'B', option_c: 'C', option_d: 'D', correct_answer: 'A' },
    });
    expect(res.status()).toBe(201);
  });

  test('guest_quiz question_type is "guest_quiz"', async ({ request }) => {
    const { code } = await createSession(request);
    const q = await addGuestQuiz(request, code);
    expect(q.question_type).toBe('guest_quiz');
    expect(q.id).toMatch(UUID_RE);
  });

  test('guest_quiz options are populated, category/pair_index null', async ({ request }) => {
    const { code } = await createSession(request);
    const q = await addGuestQuiz(request, code, { a: 'Blau', b: 'Grün', c: 'Rot', d: 'Gelb' });
    expect(q.option_a).toBe('Blau');
    expect(q.option_d).toBe('Gelb');
    expect(q.category).toBeNull();
    expect(q.pair_index).toBeNull();
  });

  test('points default to 100 when omitted', async ({ request }) => {
    const { code } = await createSession(request);
    const q = await addGuestQuiz(request, code);
    expect(q.points).toBe(100);
  });

  test('order_index defaults to 0 when omitted', async ({ request }) => {
    const { code } = await createSession(request);
    const q = await addGuestQuiz(request, code);
    expect(q.order_index).toBe(0);
  });

  test('custom points and order_index are respected', async ({ request }) => {
    const { code } = await createSession(request);
    const q = await addGuestQuiz(request, code, { points: 250, order: 5 });
    expect(q.points).toBe(250);
    expect(q.order_index).toBe(5);
  });

  test('lowercase correct_answer is normalized to uppercase', async ({ request }) => {
    const { code } = await createSession(request);
    const q = await addGuestQuiz(request, code, { correct: 'c' });
    expect(q.correct_answer).toBe('C');
  });

  test('invalid correct_answer "E" is rejected (400)', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await request.post(`${SESSION_URL}/sessions/${code}/questions`, {
      data: { text: 'Q?', option_a: 'A', option_b: 'B', option_c: 'C', option_d: 'D', correct_answer: 'E' },
    });
    expect(res.status()).toBe(400);
  });

  test('adding to a malformed code is rejected (400)', async ({ request }) => {
    const res = await request.post(`${SESSION_URL}/sessions/abc/questions`, {
      data: { text: 'Q?', option_a: 'A', option_b: 'B', option_c: 'C', option_d: 'D', correct_answer: 'A' },
    });
    expect(res.status()).toBe(400);
  });

  test('adding to an unknown (valid) code returns 404', async ({ request }) => {
    const res = await request.post(`${SESSION_URL}/sessions/ZZZZZZ/questions`, {
      data: { text: 'Q?', option_a: 'A', option_b: 'B', option_c: 'C', option_d: 'D', correct_answer: 'A' },
    });
    expect(res.status()).toBe(404);
  });
});

test.describe('Questions — ich_oder_du', () => {
  test('adding an ich-oder-du returns 201', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await request.post(`${SESSION_URL}/sessions/${code}/ich-oder-du`, {
      data: { text: 'Wer kocht?', correct_answer: 'ich' },
    });
    expect(res.status()).toBe(201);
  });

  test('ich_oder_du question_type is "ich_oder_du"', async ({ request }) => {
    const { code } = await createSession(request);
    const q = await addIchOderDu(request, code);
    expect(q.question_type).toBe('ich_oder_du');
  });

  test('ich_oder_du forces points to 0', async ({ request }) => {
    const { code } = await createSession(request);
    const q = await addIchOderDu(request, code);
    expect(q.points).toBe(0);
  });

  test('ich_oder_du has no answer options (all null)', async ({ request }) => {
    const { code } = await createSession(request);
    const q = await addIchOderDu(request, code);
    expect(q.option_a).toBeNull();
    expect(q.option_b).toBeNull();
    expect(q.option_c).toBeNull();
    expect(q.option_d).toBeNull();
  });

  test('uppercase "ICH" is normalized to "ich"', async ({ request }) => {
    const { code } = await createSession(request);
    const q = await addIchOderDu(request, code, { correct: 'ICH' });
    expect(q.correct_answer).toBe('ich');
  });

  test('invalid couple answer is rejected (400)', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await request.post(`${SESSION_URL}/sessions/${code}/ich-oder-du`, {
      data: { text: 'Wer?', correct_answer: 'vielleicht' },
    });
    expect(res.status()).toBe(400);
  });

  test('category and pair_index are passed through', async ({ request }) => {
    const { code } = await createSession(request);
    const q = await addIchOderDu(request, code, { category: 'Haushalt', pairIndex: 2 });
    expect(q.category).toBe('Haushalt');
    expect(q.pair_index).toBe(2);
  });
});

test.describe('Questions — list / update / delete', () => {
  test('GET questions returns an array containing the added question', async ({ request }) => {
    const { code } = await createSession(request);
    const q = await addGuestQuiz(request, code);
    const list = await (await request.get(`${SESSION_URL}/sessions/${code}/questions`)).json();
    expect(Array.isArray(list)).toBe(true);
    expect(list.some((x: any) => x.id === q.id)).toBe(true);
  });

  test('list reflects the number of added questions', async ({ request }) => {
    const { code } = await createSession(request);
    await addGuestQuiz(request, code, { order: 0 });
    await addGuestQuiz(request, code, { order: 1 });
    await addGuestQuiz(request, code, { order: 2 });
    const list = await (await request.get(`${SESSION_URL}/sessions/${code}/questions`)).json();
    expect(list.length).toBe(3);
  });

  test('mixed guest_quiz + ich_oder_du both appear in the list', async ({ request }) => {
    const { code } = await createSession(request);
    await addGuestQuiz(request, code);
    await addIchOderDu(request, code);
    const list = await (await request.get(`${SESSION_URL}/sessions/${code}/questions`)).json();
    const types = list.map((q: any) => q.question_type).sort();
    expect(types).toEqual(['guest_quiz', 'ich_oder_du']);
  });

  test('PUT updates the question text', async ({ request }) => {
    const { code } = await createSession(request);
    const q = await addGuestQuiz(request, code);
    const res = await request.put(`${SESSION_URL}/sessions/${code}/questions/${q.id}`, {
      data: { text: 'Neuer Text' },
    });
    expect(res.status()).toBe(200);
    expect((await res.json()).text).toBe('Neuer Text');
  });

  test('PUT can update points only (partial)', async ({ request }) => {
    const { code } = await createSession(request);
    const q = await addGuestQuiz(request, code);
    const res = await request.put(`${SESSION_URL}/sessions/${code}/questions/${q.id}`, {
      data: { points: 999 },
    });
    expect(res.status()).toBe(200);
    expect((await res.json()).points).toBe(999);
  });

  test('PUT on an unknown question_id returns 404', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await request.put(
      `${SESSION_URL}/sessions/${code}/questions/00000000-0000-0000-0000-000000000000`,
      { data: { text: 'X' } },
    );
    expect(res.status()).toBe(404);
  });

  test('PUT with a non-UUID question_id returns 400', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await request.put(`${SESSION_URL}/sessions/${code}/questions/not-a-uuid`, {
      data: { text: 'X' },
    });
    expect(res.status()).toBe(400);
  });

  test('DELETE removes the question from the list', async ({ request }) => {
    const { code } = await createSession(request);
    const q = await addGuestQuiz(request, code);
    const del = await request.delete(`${SESSION_URL}/sessions/${code}/questions/${q.id}`);
    expect(del.status()).toBe(204);
    const list = await (await request.get(`${SESSION_URL}/sessions/${code}/questions`)).json();
    expect(list.some((x: any) => x.id === q.id)).toBe(false);
  });
});
