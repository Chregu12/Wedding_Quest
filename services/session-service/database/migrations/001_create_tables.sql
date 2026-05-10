CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE game_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(6) NOT NULL UNIQUE,
    status VARCHAR(20) NOT NULL DEFAULT 'lobby',
    host_name VARCHAR(100) NOT NULL,
    person_a_name VARCHAR(100) NOT NULL,
    person_b_name VARCHAR(100) NOT NULL,
    current_round INTEGER,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_game_sessions_code ON game_sessions(code);
CREATE INDEX idx_game_sessions_status ON game_sessions(status);

CREATE TABLE players (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES game_sessions(id) ON DELETE CASCADE,
    display_name VARCHAR(30) NOT NULL,
    avatar VARCHAR(255),
    total_score BIGINT NOT NULL DEFAULT 0,
    is_connected BOOLEAN NOT NULL DEFAULT TRUE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(session_id, display_name)
);

CREATE INDEX idx_players_session_id ON players(session_id);
