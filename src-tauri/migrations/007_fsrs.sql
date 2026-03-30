-- FSRS v4 algorithm parameters for spaced repetition
-- Adds per-card memory modeling fields

ALTER TABLE srs_cards ADD COLUMN stability REAL DEFAULT 0;
ALTER TABLE srs_cards ADD COLUMN difficulty REAL DEFAULT 0;
ALTER TABLE srs_cards ADD COLUMN memory_strength REAL DEFAULT 0;
ALTER TABLE srs_cards ADD COLUMN reviews_count INTEGER DEFAULT 0;
ALTER TABLE srs_cards ADD COLUMN actual_interval INTEGER DEFAULT 0;
