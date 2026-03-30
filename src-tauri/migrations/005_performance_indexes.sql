CREATE INDEX IF NOT EXISTS idx_words_source
ON words(source);

CREATE INDEX IF NOT EXISTS idx_srs_cards_status_due_at
ON srs_cards(status, due_at);

CREATE INDEX IF NOT EXISTS idx_srs_cards_word_id
ON srs_cards(word_id);

CREATE INDEX IF NOT EXISTS idx_review_logs_card_id_shown_at
ON review_logs(card_id, shown_at);
