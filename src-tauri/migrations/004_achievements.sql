CREATE TABLE IF NOT EXISTS achievements (
  id INTEGER PRIMARY KEY,
  achievement_key TEXT NOT NULL UNIQUE,
  unlocked_at TEXT NOT NULL
);
