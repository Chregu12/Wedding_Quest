-- The questions entity (infrastructure/persistence/models/question.rs) expects
-- `category` and `pair_index`, but 002_questions.sql never created them, so any
-- read of the questions table failed with "column questions.category does not exist".
-- Add the missing nullable columns.

ALTER TABLE questions ADD COLUMN IF NOT EXISTS category TEXT;
ALTER TABLE questions ADD COLUMN IF NOT EXISTS pair_index INTEGER;
