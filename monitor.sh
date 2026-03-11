#!/bin/bash

DB_PATH="/Users/chenshaojie/Library/Application Support/com.chenshaojie.fragment-vocab/fragment-vocab.db"

echo "=== Fragment Vocab 实时监控 ==="
echo "按 Ctrl+C 停止监控"
echo ""

while true; do
    clear
    echo "=== 时间: $(date '+%Y-%m-%d %H:%M:%S') ==="
    echo ""
    
    echo "📊 卡片状态:"
    sqlite3 "$DB_PATH" "SELECT c.id, w.word, c.status, c.stage, c.last_result, c.correct_streak FROM srs_cards c JOIN words w ON c.word_id = w.id ORDER BY c.id LIMIT 5;"
    echo ""
    
    echo "📝 最近复习记录:"
    sqlite3 "$DB_PATH" "SELECT l.id, w.word, l.result, l.created_at FROM review_logs l JOIN srs_cards c ON l.card_id = c.id JOIN words w ON c.word_id = w.id ORDER BY l.created_at DESC LIMIT 5;"
    echo ""
    
    echo "📈 今日统计:"
    sqlite3 "$DB_PATH" "SELECT result, COUNT(*) as count FROM review_logs WHERE created_at >= date('now') GROUP BY result;"
    echo ""
    
    echo "💾 应用状态:"
    sqlite3 "$DB_PATH" "SELECT * FROM app_state;"
    echo ""
    
    echo "💻 进程状态:"
    ps aux | grep fragment-vocab | grep -v grep | awk '{printf "PID: %s, CPU: %s%%, MEM: %.1f MB\n", $2, $3, $6/1024}'
    echo ""
    
    sleep 5
done
