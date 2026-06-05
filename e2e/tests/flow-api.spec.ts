import { test, expect } from '@playwright/test';
import {
  ENGINE_URL,
  REALTIME_WS,
  createSession,
  addManyGuestQuiz,
  joinPlayer,
  startGame,
  getState,
  submitAnswer,
  closeRound,
  nextQuestion,
  resetGame,
  coupleIndividualAnswer,
} from '../helpers/api';
import { connectWs, waitForMessage, closeWs } from '../helpers/ws';

test.describe('End-to-end flows', () => {
  // Regression for the production bug: after a game has been played, the guest
  // and couple screens showed the previous game's first question until the next
  // game started. Opening the moderator console resets the engine to "waiting",
  // so /state must no longer expose a stale round.
  test('reset clears a stale round so waiting screens show no question', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);
    await joinPlayer(request, code, 'Gast');
    const round = await startGame(request, code);

    // A round is live...
    const live = await (await getState(request, code)).json();
    expect(live.status).toBe('question');
    expect(live.current_round_id).toBe(round.round_id);

    // ...moderator (re)opens the console -> reset.
    expect((await resetGame(request, code)).status()).toBe(200);

    const after = await (await getState(request, code)).json();
    expect(after.status).toBe('waiting');
    expect(after.current_round_id).toBeNull();
    expect(after.question_text).toBeNull();
  });

  test('a full two-round game plays through to game over', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 4); // total_questions = 2
    const p1 = await joinPlayer(request, code, 'P1');
    const p2 = await joinPlayer(request, code, 'P2');

    // Round 1
    const r1 = await startGame(request, code);
    expect(r1.round_number).toBe(1);
    await submitAnswer(request, code, { player_id: p1, player_name: 'P1', answer: 'A' });
    await submitAnswer(request, code, { player_id: p2, player_name: 'P2', answer: 'B' });
    expect((await closeRound(request, code)).status()).toBe(200);

    // Round 2
    const r2 = await (await nextQuestion(request, code)).json();
    expect(r2.round_number).toBe(2);
    await submitAnswer(request, code, { player_id: p1, player_name: 'P1', answer: 'A' });
    await submitAnswer(request, code, { player_id: p2, player_name: 'P2', answer: 'A' });
    expect((await closeRound(request, code)).status()).toBe(200);

    // Past the last round -> game over (204), state finished
    expect((await nextQuestion(request, code)).status()).toBe(204);
    const state = await (await getState(request, code)).json();
    expect(state.status).toBe('finished');
    expect(state.current_round_id).toBeNull();
  });

  test('couple disagreement resolves to "uneinig" (empty final answer)', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);
    await startGame(request, code);
    await coupleIndividualAnswer(request, code, 'a', 'ich');
    const res = await (await coupleIndividualAnswer(request, code, 'b', 'du')).json();
    expect(res.both_answered).toBe(true);
    expect(res.agree).toBe(false);
    expect(res.final_answer).toBe('');
  });

  test('a WebSocket client receives a Connected handshake for its room', async ({ request }) => {
    const { code } = await createSession(request);
    let ws;
    try {
      ws = await connectWs(`${REALTIME_WS}/ws/${code}`);
      // realtime serializes ServerEvent in SCREAMING_SNAKE_CASE -> type "CONNECTED".
      const msg = await waitForMessage(ws, (m) => m.type === 'CONNECTED');
      expect(msg.session_code).toBe(code);
    } finally {
      closeWs(ws);
    }
  });

  // Regression for the second production bug: guests loading the game page saw
  // the previous game's first question before the moderator pressed "Timer
  // starten". The moderator's "Spiel starten" now resets the engine first.
  test('moderator start sequence keeps guests waiting until the timer starts', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);

    // A previous game left the engine on a live question.
    await startGame(request, code);
    expect((await (await getState(request, code)).json()).status).toBe('question');

    // Moderator clicks "Spiel starten" -> engine reset (what guests poll next).
    await resetGame(request, code);
    expect((await (await getState(request, code)).json()).status).toBe('waiting');

    // Only when the moderator clicks "Timer starten" does a question go live.
    await startGame(request, code);
    expect((await (await getState(request, code)).json()).status).toBe('question');
  });

  test('a WebSocket client sees the QuestionStarted -> RoundClosed lifecycle', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);
    let ws;
    try {
      ws = await connectWs(`${REALTIME_WS}/ws/${code}`);
      await waitForMessage(ws, (m) => m.type === 'CONNECTED', 5000);
      const started = waitForMessage(ws, (m) => m.type === 'QuestionStarted');
      await startGame(request, code);
      await started;
      const closed = waitForMessage(ws, (m) => m.type === 'RoundClosed');
      await closeRound(request, code);
      expect((await closed).correct_answer).toBe('A');
    } finally {
      closeWs(ws);
    }
  });

  test('all players answers are recorded with their correctness', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);
    const p1 = await joinPlayer(request, code, 'Eins');
    const p2 = await joinPlayer(request, code, 'Zwei');
    const p3 = await joinPlayer(request, code, 'Drei');
    const round = await startGame(request, code);
    await submitAnswer(request, code, { player_id: p1, player_name: 'Eins', answer: 'A' });
    await submitAnswer(request, code, { player_id: p2, player_name: 'Zwei', answer: 'B' });
    await submitAnswer(request, code, { player_id: p3, player_name: 'Drei', answer: 'A' });
    const list = await (
      await request.get(`${ENGINE_URL}/games/${code}/rounds/${round.round_id}/answers`)
    ).json();
    expect(list.length).toBe(3);
    expect(list.filter((a: any) => a.is_correct).length).toBe(2);
  });
});
