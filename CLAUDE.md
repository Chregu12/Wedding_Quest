# Wedding Quest — Projektregeln für Claude

## Framework: RustForge

Wir nutzen ausschließlich **https://github.com/Chregu12/RustForge** als Backend-Framework.

- Nichts selbst bauen, was RustForge bereits kann oder können sollte.
- Vor jeder neuen Implementierung prüfen: Gibt es das in RustForge? (`rf-auth`, `rf-cache`, `rf-queue`, `rf-web`, `rf-orm`, `rf-validation`, etc.)
- Wenn ein benötigtes Feature in RustForge fehlt oder unvollständig ist: **das Framework sinnvoll erweitern** statt es zu umgehen.

### Lokale RustForge-Kopie

Das Framework liegt lokal unter **`/Users/christian/Developer/Github_Projekte/RustForge`**.

- Alle Framework-Änderungen werden **zuerst lokal** implementiert und getestet, dann auf GitHub gepusht.
- Vor dem Starten: `git pull origin main` im RustForge-Verzeichnis ausführen, um die lokale Kopie aktuell zu halten.
- Wedding Quest nutzt RustForge über Git-Dependency (`git = "https://github.com/Chregu12/RustForge"`). Nach einem Push muss `cargo update` im Wedding-Quest-Workspace ausgeführt werden, damit die neue Version gezogen wird.

### RustForge erweitern

Workflow wenn ein Feature fehlt:
1. Lokale Kopie aktuell halten: `cd /Users/christian/Developer/Github_Projekte/RustForge && git pull`
2. Die passende Crate identifizieren (z.B. `crates/rf-web`, `crates/rf-cache`, `crates/rf-broadcast`)
3. Feature lokal implementieren
4. Lokal testen: `cargo check -p <crate-name> [--features ...]` im RustForge-Verzeichnis
5. Commiten und pushen: `git push origin main`
6. Wedding Quest updaten: `cargo update` im Wedding-Quest-Verzeichnis

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
