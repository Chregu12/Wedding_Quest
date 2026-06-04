# Deployment — Wedding Quest

Production runs as a **single-port stack** behind an nginx gateway. The browser
only ever talks to one origin (`http://<host>:8080`); the gateway reverse-proxies
to the four Rust services internally. Nothing else is exposed to the host.

## Architecture

```
Browser ──► :8080  nginx gateway (serves Angular SPA)
                    ├─ /api/session/  ─► session-service  :3002
                    ├─ /api/engine/   ─► engine-service   :3003
                    ├─ /api/scoring/  ─► scoring-service  :3004
                    └─ /ws/           ─► realtime-service :3006  (WebSocket)
                                         Postgres ×3 + Redis (internal only)
```

The Angular services use **relative paths** (`/api/...`, `/ws/...`) so the same
build works on any host/port without rebuilding.

## First deploy

```bash
git clone https://github.com/Chregu12/Wedding_Quest.git /opt/wedding-quest
cd /opt/wedding-quest

# 1. Secrets
cp .env.prod.example .env
sed -i "s/change-me.*/$(openssl rand -hex 24)/" .env

# 2. Build + start (Rust build takes a while on first run)
docker compose -f docker-compose.prod.yml up -d --build

# 3. Open the firewall for the public port
ufw allow 8080/tcp
```

App is then reachable at `http://<host>:8080` (admin panel at `/admin`).

## Database schema

The services do **not** auto-migrate. The SQL under
`services/<svc>/database/migrations/` is mounted into each Postgres container's
`/docker-entrypoint-initdb.d/` and runs automatically **the first time** each
volume is created. To re-apply from scratch, remove the volume:

```bash
docker compose -f docker-compose.prod.yml down
docker volume rm wedding-quest_wq_postgres_session   # etc.
```

## Isolation

- Dedicated compose project `wedding-quest`, network `wedding-quest-net`,
  volumes `wq_*`. Shares the host with other apps but touches none of them.
- Only the gateway publishes a host port (`8080`). Postgres/Redis/services are
  reachable only inside the network.
