CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE player_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_code VARCHAR(6) NOT NULL,
    player_id UUID NOT NULL,
    player_name VARCHAR(100) NOT NULL,
    total_score INTEGER NOT NULL DEFAULT 0,
    rounds_played INTEGER NOT NULL DEFAULT 0,
    last_round_score INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (session_code, player_id)
);

CREATE TABLE round_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    round_id UUID NOT NULL,
    session_code VARCHAR(6) NOT NULL,
    player_id UUID NOT NULL,
    player_name VARCHAR(100) NOT NULL,
    base_points INTEGER NOT NULL DEFAULT 0,
    time_multiplier DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    final_points INTEGER NOT NULL DEFAULT 0,
    is_correct BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
