import { test, expect, APIRequestContext } from '@playwright/test';
import {
  SCORING_URL,
  createSession,
  addGuestQuiz,
  joinPlayer,
  startGame,
  submitAnswer,
  closeRound,
  getLeaderboard,
} from '../helpers/api';

/** Play one full quiz round with two players (p1 correct, p2 wrong) and close it. */
async function playRound(api: APIRequestContext) {
  const { code } = await createSession(api);
  await addGuestQuiz(api, code, { correct: 'A', order: 0 });
  await addGuestQuiz(api, code, { correct: 'A', order: 1 });
  const p1 = await joinPlayer(api, code, 'Schnell');
  const p2 = await joinPlayer(api, code, 'Langsam');
  await startGame(api, code);
  await submitAnswer(api, code, { player_id: p1, player_name: 'Schnell', answer: 'A' });
  await submitAnswer(api, code, { player_id: p2, player_name: 'Langsam', answer: 'B' });
  await closeRound(api, code);
  return { code, p1, p2 };
}

test.describe('Scoring — leaderboard basics', () => {
  test('leaderboard for an unknown code is an empty list (200)', async ({ request }) => {
    const res = await getLeaderboard(request, 'ZZZZZZ');
    expect(res.status()).toBe(200);
    const json = await res.json();
    expect(json.scores).toEqual([]);
  });

  test('leaderboard response carries session_code and a scores array', async ({ request }) => {
    const { code } = await createSession(request);
    const json = await (await getLeaderboard(request, code)).json();
    expect(json.session_code).toBe(code);
    expect(Array.isArray(json.scores)).toBe(true);
  });

  test('DELETE /scores/{code} resets scores (200)', async ({ request }) => {
    const { code } = await createSession(request);
    const res = await request.delete(`${SCORING_URL}/scores/${code}`);
    expect(res.status()).toBe(200);
  });
});

test.describe('Scoring — config proxy', () => {
  test('config endpoint returns 200 with all config fields', async ({ request }) => {
    const { code } = await createSession(request);
    const json = await (await request.get(`${SCORING_URL}/scores/${code}/config`)).json();
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

  test('config endpoint exposes the documented default values', async ({ request }) => {
    const { code } = await createSession(request);
    const json = await (await request.get(`${SCORING_URL}/scores/${code}/config`)).json();
    expect(json.tier1_max_seconds).toBe(10);
    expect(json.tier2_max_seconds).toBe(20);
    expect(json.tier1_multiplier).toBe(3.0);
    expect(json.base_points).toBe(100);
  });
});

test.describe('Scoring — computed scores', () => {
  test('a correct answer eventually produces a positive score', async ({ request }) => {
    const { code } = await playRound(request);
    await expect
      .poll(async () => {
        const json = await (await getLeaderboard(request, code)).json();
        const winner = json.scores.find((s: any) => s.player_name === 'Schnell');
        return winner?.total_score ?? 0;
      }, { timeout: 15_000, intervals: [500, 1000] })
      .toBeGreaterThan(0);
  });

  test('leaderboard stays sorted by total_score descending', async ({ request }) => {
    const { code } = await playRound(request);
    await expect
      .poll(async () => (await (await getLeaderboard(request, code)).json()).scores.length, {
        timeout: 15_000,
      })
      .toBeGreaterThan(0);
    const scores = (await (await getLeaderboard(request, code)).json()).scores;
    const totals = scores.map((s: any) => s.total_score);
    const sorted = [...totals].sort((a, b) => b - a);
    expect(totals).toEqual(sorted);
  });

  test('the leading player is ranked 1', async ({ request }) => {
    const { code } = await playRound(request);
    await expect
      .poll(async () => (await (await getLeaderboard(request, code)).json()).scores.length, {
        timeout: 15_000,
      })
      .toBeGreaterThan(0);
    const scores = (await (await getLeaderboard(request, code)).json()).scores;
    expect(scores[0].rank).toBe(1);
  });
});
