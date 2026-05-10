# Wedding Quest — Projektregeln für Claude

## Framework: RustForge

Wir nutzen ausschließlich **https://github.com/Chregu12/RustForge** als Backend-Framework.

- Nichts selbst bauen, was RustForge bereits kann oder können sollte.
- Vor jeder neuen Implementierung prüfen: Gibt es das in RustForge? (`rf-auth`, `rf-cache`, `rf-queue`, `rf-web`, `rf-orm`, `rf-validation`, etc.)
- Wenn ein benötigtes Feature in RustForge fehlt oder unvollständig ist: **das Framework sinnvoll erweitern** statt es zu umgehen.

### RustForge erweitern

Wenn ein Feature fehlt:
1. Die passende RustForge-Crate identifizieren (z.B. `crates/rf-web` für Routing, `crates/rf-cache` für Redis)
2. Das Feature dort implementieren — nicht als Workaround im Service-Code
3. Den Service dann die RustForge-API nutzen lassen

Ziel: Dieses Projekt treibt die Weiterentwicklung von RustForge. Jeder Service testet und verbessert andere RustForge-Features.

## Architektur

- **Microservices + DDD** — ein Service pro Bounded Context
- Jeder Service ist eine eigenständige RustForge-App
- Struktur pro Service: `domain/` → `application/` → `infrastructure/` → `api/`
- Domain Events über Redis Pub/Sub (Channel-Pattern: `wedding_quest:session:<id>`)

## Stack

| Schicht | Technologie |
|---|---|
| Backend | RustForge (Axum + SeaORM + Tokio) |
| Frontend | Angular 18+ |
| Datenbank | PostgreSQL (eine DB pro Service) |
| Cache / Events | Redis |
| WebSockets | RustForge rf-echo / Axum ws |
