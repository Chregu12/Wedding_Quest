CREATE TABLE questions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES game_sessions(id) ON DELETE CASCADE,
    question_type VARCHAR(20) NOT NULL DEFAULT 'guest_quiz',
    text TEXT NOT NULL,
    option_a TEXT,
    option_b TEXT,
    option_c TEXT,
    option_d TEXT,
    correct_answer VARCHAR(10) NOT NULL,
    order_index INTEGER NOT NULL DEFAULT 0,
    points INTEGER NOT NULL DEFAULT 100,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE score_config (
    session_id UUID PRIMARY KEY REFERENCES game_sessions(id) ON DELETE CASCADE,
    tier1_max_seconds INTEGER NOT NULL DEFAULT 10,
    tier2_max_seconds INTEGER NOT NULL DEFAULT 20,
    tier1_multiplier NUMERIC(3,1) NOT NULL DEFAULT 3.0,
    tier2_multiplier NUMERIC(3,1) NOT NULL DEFAULT 2.0,
    tier3_multiplier NUMERIC(3,1) NOT NULL DEFAULT 1.0,
    perfect_match_multiplier NUMERIC(3,1) NOT NULL DEFAULT 2.0,
    catchup_multiplier NUMERIC(3,1) NOT NULL DEFAULT 1.5,
    catchup_threshold_percent INTEGER NOT NULL DEFAULT 50,
    base_points INTEGER NOT NULL DEFAULT 100
);
