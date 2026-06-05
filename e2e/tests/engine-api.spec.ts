import { test, expect } from '@playwright/test';
import { randomUUID } from 'node:crypto';
import {
  UUID_RE,
  createSession,
  addGuestQuiz,
  addManyGuestQuiz,
  addIchOderDu,
  startGame,
  getState,
  submitAnswer,
  closeRound,
  nextQuestion,
  resetGame,
  coupleIndividualAnswer,
} from '../helpers/api';

function fakePlayer() {
  return { player_id: randomUUID(), player_name: 'Spieler ' + Math.floor(performance.now()) };
}

test.describe('Engine — start', () => {
  test('start returns 200 with the round payload', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);
    const round = await startGame(request, code);
    for (const f of ['round_id', 'question_text', 'correct_answer', 'round_number', 'total_questions']) {
      expect(round).toHaveProperty(f);
    }
  });

  test('first round has round_number 1 and a UUID round_id', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);
    const round = await startGame(request, code);
    expect(round.round_number).toBe(1);
    expect(round.round_id).toMatch(UUID_RE);
  });

  test('total_questions is floor(questionCount / 2)', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 4);
    const round = await startGame(request, code);
    expect(round.total_questions).toBe(2);
  });

  test('starting a game with no questions returns 400', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await request.post(`${process.env.WQ_ENGINE_URL ?? 'http://localhost:3003'}/games/${code}/start`, {
      data: {},
    });
    expect(res.status()).toBe(400);
  });

  test('starting with only ich_oder_du questions returns 400', async ({ request }) => {
    const { code } = await createSession(request);
    await addIchOderDu(request, code);
    const res = await request.post(`${process.env.WQ_ENGINE_URL ?? 'http://localhost:3003'}/games/${code}/start`, {
      data: {},
    });
    expect(res.status()).toBe(400);
  });

  test('starting with an explicit question_id uses that question', async ({ request }) => {
    const { code } = await createSession(request);
    const qs = await addManyGuestQuiz(request, code, 2);
    const round = await startGame(request, code, qs[1].id);
    expect(round.question_text).toBe(qs[1].text);
  });

  test('starting with a bogus question_id returns 404', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);
    const res = await request.post(`${process.env.WQ_ENGINE_URL ?? 'http://localhost:3003'}/games/${code}/start`, {
      data: { question_id: randomUUID() },
    });
    expect(res.status()).toBe(404);
  });
});

test.describe('Engine — reset', () => {
  test('reset returns 200', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);
    await startGame(request, code);
    const res = await resetGame(request, code);
    expect(res.status()).toBe(200);
  });

  test('after reset the state is waiting with no current round', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);
    await startGame(request, code);
    await resetGame(request, code);
    const state = await (await getState(request, code)).json();
    expect(state.status).toBe('waiting');
    expect(state.current_round_id).toBeNull();
  });
});

test.describe('Engine — state', () => {
  test('state before any start returns 404', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await getState(request, code);
    expect(res.status()).toBe(404);
  });

  test('state after start reports status "question" with the round details', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);
    const round = await startGame(request, code);
    const state = await (await getState(request, code)).json();
    expect(state.status).toBe('question');
    expect(state.current_round_id).toBe(round.round_id);
    expect(state.question_text).toBeTruthy();
  });

  test('state reflects total_questions', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 4);
    await startGame(request, code);
    const state = await (await getState(request, code)).json();
    expect(state.total_questions).toBe(2);
  });
});

test.describe('Engine — answers', () => {
  test('a correct answer is accepted and marked correct', async ({ request }) => {
    const { code } = await createSession(request);
    await addGuestQuiz(request, code, { correct: 'A', order: 0 });
    await addGuestQuiz(request, code, { correct: 'A', order: 1 });
    await startGame(request, code);
    const res = await submitAnswer(request, code, { ...fakePlayer(), answer: 'A' });
    expect(res.status()).toBe(200);
    const json = await res.json();
    expect(json.accepted).toBe(true);
    expect(json.is_correct).toBe(true);
  });

  test('a wrong answer is accepted but not correct', async ({ request }) => {
    const { code } = await createSession(request);
    await addGuestQuiz(request, code, { correct: 'A', order: 0 });
    await addGuestQuiz(request, code, { correct: 'A', order: 1 });
    await startGame(request, code);
    const json = await (await submitAnswer(request, code, { ...fakePlayer(), answer: 'B' })).json();
    expect(json.accepted).toBe(true);
    expect(json.is_correct).toBe(false);
  });

  test('answer comparison is case-insensitive', async ({ request }) => {
    const { code } = await createSession(request);
    await addGuestQuiz(request, code, { correct: 'A', order: 0 });
    await addGuestQuiz(request, code, { correct: 'A', order: 1 });
    await startGame(request, code);
    const json = await (await submitAnswer(request, code, { ...fakePlayer(), answer: 'a' })).json();
    expect(json.is_correct).toBe(true);
  });

  test('a couple answer (couple=true) is accepted', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);
    await startGame(request, code);
    const json = await (
      await submitAnswer(request, code, { ...fakePlayer(), answer: 'A', couple: true })
    ).json();
    expect(json.accepted).toBe(true);
  });

  test('answering when no game has started returns 404', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await submitAnswer(request, code, { ...fakePlayer(), answer: 'A' });
    expect(res.status()).toBe(404);
  });
});

test.describe('Engine — round lifecycle', () => {
  test('close-round returns the correct answer and no paired ich-oder-du', async ({ request }) => {
    const { code } = await createSession(request);
    await addGuestQuiz(request, code, { correct: 'B', order: 0 });
    await addGuestQuiz(request, code, { correct: 'B', order: 1 });
    await startGame(request, code);
    const res = await closeRound(request, code);
    expect(res.status()).toBe(200);
    const json = await res.json();
    expect(json.correct_answer).toBe('B');
    expect(json.has_ich_oder_du).toBe(false);
  });

  test('next-question advances to round 2', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 4);
    await startGame(request, code);
    const res = await nextQuestion(request, code);
    expect(res.status()).toBe(200);
    expect((await res.json()).round_number).toBe(2);
  });

  test('next-question past the last round ends the game (204)', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2); // total_questions = 1
    await startGame(request, code);
    const res = await nextQuestion(request, code);
    expect(res.status()).toBe(204);
  });

  test('round answers can be fetched for an open round', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);
    const round = await startGame(request, code);
    const player = { ...fakePlayer(), answer: 'A' };
    await submitAnswer(request, code, player);
    const list = await (
      await request.get(
        `${process.env.WQ_ENGINE_URL ?? 'http://localhost:3003'}/games/${code}/rounds/${round.round_id}/answers`,
      )
    ).json();
    expect(Array.isArray(list)).toBe(true);
    expect(list.some((a: any) => a.player_id === player.player_id)).toBe(true);
  });

  test('couple-individual-answer resolves agreement when both pick the same', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);
    await startGame(request, code);
    const first = await (await coupleIndividualAnswer(request, code, 'a', 'ich')).json();
    expect(first.both_answered).toBe(false);
    const second = await (await coupleIndividualAnswer(request, code, 'b', 'ich')).json();
    expect(second.both_answered).toBe(true);
    expect(second.agree).toBe(true);
    expect(second.final_answer).toBe('ich');
  });
});
