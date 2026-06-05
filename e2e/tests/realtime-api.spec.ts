import { test, expect } from '@playwright/test';
import {
  REALTIME_WS,
  createSession,
  addManyGuestQuiz,
  startGame,
  closeRound,
  nextQuestion,
  coupleAnswer,
} from '../helpers/api';
import { connectWs, waitForMessage, openAndCollect, closeWs } from '../helpers/ws';

test.describe('Realtime — event broadcasts', () => {
  test('starting a game broadcasts QuestionStarted to the room', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);
    let ws;
    try {
      ws = await connectWs(`${REALTIME_WS}/ws/${code}`);
      await waitForMessage(ws, (m) => m.type === 'CONNECTED', 5000);
      const pending = waitForMessage(ws, (m) => m.type === 'QuestionStarted');
      await startGame(request, code);
      const msg = await pending;
      expect(msg.round_id).toBeTruthy();
      expect(msg.question_text).toBeTruthy();
    } finally {
      closeWs(ws);
    }
  });

  test('closing a round broadcasts RoundClosed with the correct answer', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);
    await startGame(request, code);
    let ws;
    try {
      ws = await connectWs(`${REALTIME_WS}/ws/${code}`);
      await waitForMessage(ws, (m) => m.type === 'CONNECTED', 5000);
      const pending = waitForMessage(ws, (m) => m.type === 'RoundClosed');
      await closeRound(request, code);
      const msg = await pending;
      expect(msg.correct_answer).toBe('A');
    } finally {
      closeWs(ws);
    }
  });

  test('a couple answer broadcasts CoupleAnswered', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);
    await startGame(request, code);
    let ws;
    try {
      ws = await connectWs(`${REALTIME_WS}/ws/${code}`);
      await waitForMessage(ws, (m) => m.type === 'CONNECTED', 5000);
      const pending = waitForMessage(ws, (m) => m.type === 'CoupleAnswered');
      await coupleAnswer(request, code, 'ich');
      const msg = await pending;
      expect(msg.couple_answer).toBe('ich');
    } finally {
      closeWs(ws);
    }
  });

  test('finishing the game broadcasts GameEnded', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2); // total_questions = 1
    await startGame(request, code);
    let ws;
    try {
      ws = await connectWs(`${REALTIME_WS}/ws/${code}`);
      await waitForMessage(ws, (m) => m.type === 'CONNECTED', 5000);
      const pending = waitForMessage(ws, (m) => m.type === 'GameEnded');
      await nextQuestion(request, code); // past the last round
      const msg = await pending;
      expect(msg.session_code).toBe(code);
    } finally {
      closeWs(ws);
    }
  });

  test('two clients in the same room both receive QuestionStarted', async ({ request }) => {
    const { code } = await createSession(request);
    await addManyGuestQuiz(request, code, 2);
    let ws1, ws2;
    try {
      const a = await openAndCollect(`${REALTIME_WS}/ws/${code}`);
      const b = await openAndCollect(`${REALTIME_WS}/ws/${code}`);
      ws1 = a.ws;
      ws2 = b.ws;
      const c1 = a.messages;
      const c2 = b.messages;
      // both subscribed (CONNECTED received) before we trigger the event
      await expect.poll(() => c1.some((m) => m.type === 'CONNECTED')).toBeTruthy();
      await expect.poll(() => c2.some((m) => m.type === 'CONNECTED')).toBeTruthy();
      await startGame(request, code);
      await expect.poll(() => c1.find((m) => m.type === 'QuestionStarted')?.round_id ?? null).toBeTruthy();
      await expect.poll(() => c2.find((m) => m.type === 'QuestionStarted')?.round_id ?? null).toBeTruthy();
      const r1 = c1.find((m) => m.type === 'QuestionStarted').round_id;
      const r2 = c2.find((m) => m.type === 'QuestionStarted').round_id;
      expect(r1).toBe(r2);
    } finally {
      closeWs(ws1);
      closeWs(ws2);
    }
  });

  test('a client does not receive events from another room', async ({ request }) => {
    const a = await createSession(request);
    await addManyGuestQuiz(request, a.code, 2);
    const b = await createSession(request);
    let ws;
    try {
      ws = await connectWs(`${REALTIME_WS}/ws/${b.code}`);
      await waitForMessage(ws, (m) => m.type === 'CONNECTED', 5000);
      const leaked = waitForMessage(ws, (m) => m.type === 'QuestionStarted', 2500);
      await startGame(request, a.code); // event published to room A only
      await expect(leaked).rejects.toThrow(/timeout/i);
    } finally {
      closeWs(ws);
    }
  });
});
