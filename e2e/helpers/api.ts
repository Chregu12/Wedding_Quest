import { APIRequestContext, expect } from '@playwright/test';

// ---- Service base URLs (override via env for CI / remote) --------------------
export const SESSION_URL = process.env.WQ_SESSION_URL ?? 'http://localhost:3002';
export const ENGINE_URL = process.env.WQ_ENGINE_URL ?? 'http://localhost:3003';
export const SCORING_URL = process.env.WQ_SCORING_URL ?? 'http://localhost:3004';
export const REALTIME_URL = process.env.WQ_REALTIME_URL ?? 'http://localhost:3006';
export const REALTIME_WS = process.env.WQ_REALTIME_WS ?? 'ws://localhost:3006';

// ---- Shared matchers --------------------------------------------------------
export const UUID_RE =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
export const CODE_RE = /^[A-Z0-9]{6}$/;

export interface CreatedSession {
  sessionId: string;
  code: string;
}

async function bodyText(res: any): Promise<string> {
  try {
    return await res.text();
  } catch {
    return '<no body>';
  }
}

/** POST /sessions — returns { sessionId, code }. Fails the test on non-201. */
export async function createSession(
  api: APIRequestContext,
  opts: { personA?: string; personB?: string; host?: string } = {},
): Promise<CreatedSession> {
  const res = await api.post(`${SESSION_URL}/sessions`, {
    data: {
      person_a_name: opts.personA ?? 'Anna',
      person_b_name: opts.personB ?? 'Ben',
      host_name: opts.host ?? 'Moderator',
    },
  });
  expect(res.status(), await bodyText(res)).toBe(201);
  const json = await res.json();
  return { sessionId: json.session_id, code: json.code };
}

export function getSession(api: APIRequestContext, code: string) {
  return api.get(`${SESSION_URL}/sessions/${code}`);
}

/** POST /sessions/{code}/questions — add a guest_quiz question. */
export async function addGuestQuiz(
  api: APIRequestContext,
  code: string,
  opts: {
    text?: string;
    a?: string;
    b?: string;
    c?: string;
    d?: string;
    correct?: string;
    order?: number;
    points?: number;
  } = {},
): Promise<any> {
  const data: any = {
    text: opts.text ?? 'Lieblingsfarbe des Bräutigams?',
    option_a: opts.a ?? 'Blau',
    option_b: opts.b ?? 'Grün',
    option_c: opts.c ?? 'Rot',
    option_d: opts.d ?? 'Gelb',
    correct_answer: opts.correct ?? 'A',
  };
  if (opts.order !== undefined) data.order_index = opts.order;
  if (opts.points !== undefined) data.points = opts.points;
  const res = await api.post(`${SESSION_URL}/sessions/${code}/questions`, { data });
  expect(res.status(), await bodyText(res)).toBe(201);
  return res.json();
}

/** Add N guest_quiz questions with incrementing order_index. */
export async function addManyGuestQuiz(
  api: APIRequestContext,
  code: string,
  count: number,
): Promise<any[]> {
  const out: any[] = [];
  for (let i = 0; i < count; i++) {
    out.push(await addGuestQuiz(api, code, { text: `Frage ${i + 1}`, correct: 'A', order: i }));
  }
  return out;
}

/** POST /sessions/{code}/ich-oder-du — add an ich_oder_du question. */
export async function addIchOderDu(
  api: APIRequestContext,
  code: string,
  opts: { text?: string; correct?: string; order?: number; category?: string; pairIndex?: number } = {},
): Promise<any> {
  const data: any = {
    text: opts.text ?? 'Wer hat den Antrag gemacht?',
    correct_answer: opts.correct ?? 'ich',
  };
  if (opts.order !== undefined) data.order_index = opts.order;
  if (opts.category !== undefined) data.category = opts.category;
  if (opts.pairIndex !== undefined) data.pair_index = opts.pairIndex;
  const res = await api.post(`${SESSION_URL}/sessions/${code}/ich-oder-du`, { data });
  expect(res.status(), await bodyText(res)).toBe(201);
  return res.json();
}

/** POST /sessions/{code}/join — returns the new player_id. */
export async function joinPlayer(
  api: APIRequestContext,
  code: string,
  displayName: string,
): Promise<string> {
  const res = await api.post(`${SESSION_URL}/sessions/${code}/join`, {
    data: { display_name: displayName },
  });
  expect(res.status(), await bodyText(res)).toBe(201);
  return (await res.json()).player_id;
}

// ---- engine-service ---------------------------------------------------------
export async function startGame(
  api: APIRequestContext,
  code: string,
  questionId?: string,
): Promise<any> {
  const res = await api.post(`${ENGINE_URL}/games/${code}/start`, {
    data: questionId ? { question_id: questionId } : {},
  });
  expect(res.status(), await bodyText(res)).toBe(200);
  return res.json();
}

export function getState(api: APIRequestContext, code: string) {
  return api.get(`${ENGINE_URL}/games/${code}/state`);
}

export function submitAnswer(
  api: APIRequestContext,
  code: string,
  body: { player_id: string; player_name: string; answer: string; couple?: boolean },
) {
  return api.post(`${ENGINE_URL}/games/${code}/answer`, { data: body });
}

export function closeRound(api: APIRequestContext, code: string) {
  return api.post(`${ENGINE_URL}/games/${code}/close-round`, { data: {} });
}

export function nextQuestion(api: APIRequestContext, code: string, questionId?: string) {
  return api.post(`${ENGINE_URL}/games/${code}/next-question`, {
    data: questionId ? { question_id: questionId } : {},
  });
}

export function resetGame(api: APIRequestContext, code: string) {
  return api.post(`${ENGINE_URL}/games/${code}/reset`, { data: {} });
}

export function coupleIndividualAnswer(
  api: APIRequestContext,
  code: string,
  person: 'a' | 'b',
  answer: string,
) {
  return api.post(`${ENGINE_URL}/games/${code}/couple-individual-answer`, {
    data: { person, answer },
  });
}

// ---- scoring-service --------------------------------------------------------
export function getLeaderboard(api: APIRequestContext, code: string) {
  return api.get(`${SCORING_URL}/scores/${code}`);
}

/**
 * Build a fully-prepared, started game: a session with `questionCount`
 * guest_quiz questions, `playerNames` joined, and the first round open.
 * Returns the session + the StartGameResponse + the joined player ids.
 */
export async function startedGame(
  api: APIRequestContext,
  opts: { questionCount?: number; playerNames?: string[] } = {},
): Promise<{ code: string; sessionId: string; round: any; players: Record<string, string> }> {
  const session = await createSession(api);
  await addManyGuestQuiz(api, session.code, opts.questionCount ?? 2);
  const players: Record<string, string> = {};
  for (const name of opts.playerNames ?? []) {
    players[name] = await joinPlayer(api, session.code, name);
  }
  const round = await startGame(api, session.code);
  return { code: session.code, sessionId: session.sessionId, round, players };
}
