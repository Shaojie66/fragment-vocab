PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS words (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  word TEXT NOT NULL UNIQUE,
  phonetic TEXT,
  part_of_speech TEXT,
  meaning_zh TEXT NOT NULL,
  source TEXT NOT NULL DEFAULT 'ielts-core',
  difficulty INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS srs_cards (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  word_id INTEGER NOT NULL UNIQUE,
  status TEXT NOT NULL DEFAULT 'new' CHECK (status IN ('new', 'learning', 'review', 'mastered')),
  stage INTEGER NOT NULL DEFAULT -1,
  due_at TEXT,
  last_seen_at TEXT,
  last_result TEXT CHECK (last_result IN ('know', 'dont_know', 'skip')),
  correct_streak INTEGER NOT NULL DEFAULT 0,
  lifetime_correct INTEGER NOT NULL DEFAULT 0,
  lifetime_wrong INTEGER NOT NULL DEFAULT 0,
  skip_cooldown_until TEXT,
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY (word_id) REFERENCES words(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS review_logs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  card_id INTEGER NOT NULL,
  shown_at TEXT NOT NULL,
  result TEXT NOT NULL CHECK (result IN ('know', 'dont_know', 'skip')),
  trigger_type TEXT NOT NULL CHECK (trigger_type IN ('idle', 'fallback', 'manual')),
  response_ms INTEGER,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY (card_id) REFERENCES srs_cards(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS app_state (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_words_word
ON words(word);

CREATE INDEX IF NOT EXISTS idx_srs_cards_status_due_at
ON srs_cards(status, due_at);

CREATE INDEX IF NOT EXISTS idx_srs_cards_skip_cooldown_until
ON srs_cards(skip_cooldown_until);

CREATE INDEX IF NOT EXISTS idx_review_logs_card_id
ON review_logs(card_id);

CREATE INDEX IF NOT EXISTS idx_review_logs_shown_at
ON review_logs(shown_at);
