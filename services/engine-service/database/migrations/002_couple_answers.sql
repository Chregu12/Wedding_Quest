-- The game_round entity (infrastructure/persistence/models/game_round.rs) has
-- `couple_answer_a` and `couple_answer_b`, but 001_engine_tables.sql only created
-- `couple_answer`. Starting a game failed with
-- "column couple_answer_a of relation game_rounds does not exist".

ALTER TABLE game_rounds ADD COLUMN IF NOT EXISTS couple_answer_a VARCHAR(10);
ALTER TABLE game_rounds ADD COLUMN IF NOT EXISTS couple_answer_b VARCHAR(10);
