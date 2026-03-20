-- Pet state table for Pixel Pet feature
CREATE TABLE IF NOT EXISTS pets (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- Singleton table, only one pet per user
    stage INTEGER NOT NULL DEFAULT 0,       -- 0-4: egg, hatchling, juvenile, adult, fully-evolved
    health REAL NOT NULL DEFAULT 1.0,       -- 0.0 - 1.0
    experience INTEGER NOT NULL DEFAULT 0,  -- Cumulative experience points
    current_streak INTEGER NOT NULL DEFAULT 0,  -- Consecutive study days
    vitality_multiplier REAL NOT NULL DEFAULT 1.0,  -- 1.0 - 3.0
    last_study_at TEXT,                    -- Last study timestamp (nullable for first launch)
    last_review_at TEXT,                    -- Last review timestamp (nullable for first launch)
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert default pet if not exists
INSERT OR IGNORE INTO pets (id, stage, health, experience, current_streak, vitality_multiplier, created_at, updated_at)
VALUES (1, 0, 1.0, 0, 0, 1.0, datetime('now'), datetime('now'));
