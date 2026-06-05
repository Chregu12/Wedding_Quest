import { test, expect } from '@playwright/test';
import {
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
});
