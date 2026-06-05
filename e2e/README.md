# Wedding Quest — E2E API Tests

API-level end-to-end tests for the Wedding Quest microservices. They talk HTTP/WS
directly to the services via Playwright's `request` fixture — **no browser**.

## What is covered (~100 tests)

| File | Domain |
|------|--------|
| `tests/health-api.spec.ts` | health endpoints of all 4 services |
| `tests/session-api.spec.ts` | create / read / list / update / start sessions |
| `tests/players-api.spec.ts` | join, lobby, clear players |
| `tests/questions-api.spec.ts` | guest_quiz + ich_oder_du, list / update / delete |
| `tests/engine-api.spec.ts` | start / reset / state / answer / close / next / couple |
| `tests/scoring-api.spec.ts` | leaderboard, reset, config proxy, computed scores |
| `tests/flow-api.spec.ts` | full game flow, **stale-question reset regression**, couple disagree, WebSocket handshake |

## Prerequisites

The local dev stack must be running:

```bash
# from the repo root
docker compose up -d --build
```

This exposes: session `:3002`, engine `:3003`, scoring `:3004`, realtime `:3006`.

## Run

```bash
cd e2e
npm install
npx playwright install   # not strictly needed (no browser), but installs the runner deps
npm test                 # all tests
npm run test:report      # open the last HTML report
```

## Configuration

Override service URLs via env vars (defaults target the local stack):

```
WQ_SESSION_URL   (default http://localhost:3002)
WQ_ENGINE_URL    (default http://localhost:3003)
WQ_SCORING_URL   (default http://localhost:3004)
WQ_REALTIME_URL  (default http://localhost:3006)
WQ_REALTIME_WS   (default ws://localhost:3006)
```

Tests are self-contained: each creates its own session/players/questions via the
API, so they can run in parallel and need no seed data.
