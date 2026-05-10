CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE game_rounds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_code VARCHAR(6) NOT NULL,
    question_id UUID NOT NULL,
    question_type VARCHAR(20) NOT NULL DEFAULT 'guest_quiz',
    question_text TEXT NOT NULL,
    option_a TEXT,
    option_b TEXT,
    option_c TEXT,
    option_d TEXT,
    correct_answer VARCHAR(10) NOT NULL,
    ich_oder_du_id UUID,
    ich_oder_du_text TEXT,
    ich_oder_du_correct VARCHAR(10),
    couple_answer VARCHAR(10),
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    round_number INTEGER NOT NULL DEFAULT 1,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMPTZ
);

CREATE TABLE player_answers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    round_id UUID NOT NULL REFERENCES game_rounds(id) ON DELETE CASCADE,
    player_id UUID NOT NULL,
    player_name VARCHAR(100) NOT NULL,
    answer VARCHAR(10) NOT NULL,
    is_correct BOOLEAN NOT NULL DEFAULT FALSE,
    answered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    time_taken_seconds NUMERIC(6,2) NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX player_answers_round_player ON player_answers(round_id, player_id);

CREATE TABLE game_state (
    session_code VARCHAR(6) PRIMARY KEY,
    status VARCHAR(20) NOT NULL DEFAULT 'waiting',
    current_round_id UUID,
    current_round_number INTEGER NOT NULL DEFAULT 0,
    total_questions INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
