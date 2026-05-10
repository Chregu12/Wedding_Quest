# Wedding Quest 💍

Ein interaktives Hochzeits-Quiz-Spiel für Gäste. Alle Teilnehmer spielen live auf dem Smartphone mit, während die Rangliste auf einem Beamer für alle sichtbar ist.

---

## Features

- **Live-Quiz** — Multiple-Choice-Fragen mit 30-Sekunden-Timer und Zeitbonus-Multiplikatoren
- **Ich oder Du** — Das Brautpaar beantwortet persönliche Fragen, Gäste schauen gespannt zu
- **Beamer-Ansicht** — Cinematic Dark-Mode Display für die Leinwand mit Podium und Rangliste
- **Mobile-Ansicht** — Touch-optimierte Gäste-App mit animierter Frageansicht
- **Lucky Boost** — Letzter Platz bekommt nach jeder Runde einen zufälligen Multiplikator (×1.5 / ×2 / ×3)
- **Catch-Up Bonus** — Spieler mit weniger als 50% des Maximalscores erhalten automatisch ×1.5
- **Live-Scoring** — Punkte werden in Echtzeit über WebSockets übertragen
- **Admin Panel** — Fragen verwalten, Session starten, QR-Codes generieren

---

## Workflow

### Vorbereitung (Admin)

```
1. /admin → Session erstellen (Brautpaar-Namen + Moderator)
2. /admin/sessions/:code → Fragen & Einstellungen konfigurieren
   ├── Quiz-Fragen hinzufügen (4 Optionen, richtige Antwort, Punkte)
   ├── Ich-oder-Du Fragen hinzufügen
   └── Punktekonfiguration anpassen (Zeitlimits, Multiplikatoren)
```

### Am Spieltag

```
3. Beamer-URL öffnen:  /display/:code   → Lobby erscheint auf der Leinwand
4. Gäste scannen QR-Code → /join → Anzeigename eingeben → /game/:code
5. Admin drückt "Spiel starten ▶"
```

### Spielrunde

```
┌─ Admin startet Runde
│
├─ Frage erscheint (Beamer groß, Handy kompakt)
│   └─ 30-Sekunden Countdown mit Multiplikator-Zonen:
│       0–10s → ×3 (grün)  |  10–20s → ×2 (gelb)  |  20–30s → ×1 (grau)
│
├─ Gäste tippen ihre Antwort auf dem Handy
│
├─ Admin schließt Runde → richtige Antwort wird enthüllt
│   ├─ Beamer: Antwort leuchtet auf, Rangliste aktualisiert sich
│   └─ Handy: ✓ Richtig / ✗ Falsch + Punkte
│
├─ Scoring-Service berechnet Punkte:
│   base_points × time_multiplier × [catchup_bonus] × [lucky_boost]
│
└─ ⚡ Lucky Boost: letzter Platz bekommt zufälligen Multiplikator für die nächste Runde
```

### Ich-oder-Du Phase

```
Admin startet → Frage erscheint auf Beamer & Handys
→ Brautpaar antwortet live
→ Admin gibt Antwort ein → Enthüllung für alle
```

### Spielende

```
Admin beendet Spiel → Beamer zeigt großes Podium (🥇🥈🥉) mit Feier-Animation
```

---

## Architektur

```
┌─────────────────────────────────────────────────────────┐
│                      Angular Frontend                    │
│  /admin/*        /display/:code     /game/:code          │
│  Admin Panel     Beamer-Ansicht     Gäste-Handy          │
└──────────────────────┬──────────────────────────────────┘
                       │ HTTP + WebSocket
┌──────────────────────▼──────────────────────────────────┐
│                    Microservices (Rust / RustForge)       │
│                                                          │
│  session-service :3002    engine-service :3003           │
│  Sessions, Spieler,       Spielablauf, Fragen,           │
│  Fragen, QR-Codes         Antworten verwalten            │
│                                                          │
│  scoring-service :3004    realtime-service :3006         │
│  Punkte berechnen,        WebSocket-Hub,                 │
│  Lucky Boost zuweisen     Redis → WS Broadcast           │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│              Redis (Pub/Sub + Cache)                     │
│  wedding_quest:game:<code>     → Engine-Events           │
│  wedding_quest:session:<code>  → Client-Events           │
└─────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────┐
│       PostgreSQL (je eine DB pro Service)                │
│  wedding_session   wedding_engine   wedding_scoring       │
└──────────────────────────────────────────────────────────┘
```

---

## Tech Stack

| Schicht      | Technologie                              |
|-------------|------------------------------------------|
| Backend     | Rust + [RustForge](https://github.com/Chregu12/RustForge) (Axum, SeaORM, Tokio) |
| Frontend    | Angular 18+ (Signals, Standalone Components) |
| Styling     | Tailwind CSS                             |
| Datenbank   | PostgreSQL (eine Instanz pro Service)    |
| Cache/Events| Redis (Pub/Sub)                          |
| WebSockets  | RustForge rf-broadcast + rf-cache        |
| Infra       | Docker Compose                           |

---

## Lokales Setup

### Voraussetzungen

- Docker + Docker Compose
- Rust (stable)
- Node.js 20+ / npm
- Angular CLI (`npm install -g @angular/cli`)

### Starten

```bash
# 1. Infrastruktur hochfahren (Postgres + Redis)
docker-compose up -d postgres-session postgres-engine postgres-scoring redis

# 2. Alle Services starten
docker-compose up

# 3. Frontend starten
cd frontend
npm install
ng serve
```

Die App ist dann erreichbar unter:

| URL | Beschreibung |
|-----|-------------|
| `http://localhost:4200/admin` | Admin Panel |
| `http://localhost:4200/join` | Gäste-Einstieg |
| `http://localhost:4200/display/:code` | Beamer-Ansicht |
| `http://localhost:4200/game/:code` | Gäste-Handy-App |

### Services einzeln starten (Entwicklung)

```bash
# Session Service
cargo run -p session-service

# Engine Service
cargo run -p engine-service

# Scoring Service
cargo run -p scoring-service

# Realtime Service
cargo run -p realtime-service
```

---

## Punkteberechnung

```
Endpunkte = base_points × time_multiplier × catchup_bonus × lucky_boost

time_multiplier:
  0–10s  → ×3   (Tier 1)
  10–20s → ×2   (Tier 2)
  20–30s → ×1   (Tier 3)

catchup_bonus = ×1.5
  greift automatisch, wenn Spieler < 50% des Maximalscores hat

lucky_boost = ×1.5 | ×2.0 | ×3.0 (zufällig)
  letzter Platz nach jeder Runde
  gilt für den nächsten richtigen Treffer, danach Reset
```

Alle Multiplikatoren sind über das Admin Panel konfigurierbar.

---

## Projektstruktur

```
Wedding_Quest/
├── services/
│   ├── session-service/     # Sessions, Spieler, Fragen
│   ├── engine-service/      # Spielablauf, Antworten
│   ├── scoring-service/     # Punkteberechnung, Lucky Boost
│   └── realtime-service/    # WebSocket-Hub
├── frontend/                # Angular App
│   └── src/app/
│       ├── admin/           # Admin Panel
│       ├── display/         # Beamer-Ansicht
│       └── guest/           # Gäste (Join + Game)
├── docker-compose.yml
└── Cargo.toml               # Rust Workspace
```

---

## Framework: RustForge

Dieses Projekt ist gleichzeitig ein Testbed für [RustForge](https://github.com/Chregu12/RustForge) — ein Laravel-inspiriertes Rust-Backend-Framework. Jeder Service testet und verbessert andere RustForge-Features:

- `rf-web` — Router, Middleware, CORS, Tracing
- `rf-orm` — DatabaseManager (SeaORM-Wrapper)
- `rf-cache` — RedisPubSub (Publish/Subscribe)
- `rf-broadcast` — RoomRegistry (WebSocket Room Management)
