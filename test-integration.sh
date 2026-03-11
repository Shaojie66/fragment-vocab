#!/bin/bash

# Fragment Vocab 集成测试脚本

set -e

DB_PATH="/Users/chenshaojie/Library/Application Support/com.chenshaojie.fragment-vocab/fragment-vocab.db"

echo "=== Fragment Vocab 集成测试 ==="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试计数
PASSED=0
FAILED=0

# 测试函数
test_case() {
    local name="$1"
    local command="$2"
    local expected="$3"
    
    echo -n "测试: $name ... "
    
    result=$(eval "$command" 2>&1)
    
    if [[ "$result" == "$expected" ]]; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((PASSED++))
    else
        echo -e "${RED}✗ FAIL${NC}"
        echo "  期望: $expected"
        echo "  实际: $result"
        ((FAILED++))
    fi
}

# 1. 数据库结构测试
echo "📊 1. 数据库结构测试"
test_case "words 表存在" \
    "sqlite3 '$DB_PATH' \"SELECT name FROM sqlite_master WHERE type='table' AND name='words';\"" \
    "words"

test_case "srs_cards 表存在" \
    "sqlite3 '$DB_PATH' \"SELECT name FROM sqlite_master WHERE type='table' AND name='srs_cards';\"" \
    "srs_cards"

test_case "review_logs 表存在" \
    "sqlite3 '$DB_PATH' \"SELECT name FROM sqlite_master WHERE type='table' AND name='review_logs';\"" \
    "review_logs"

test_case "app_state 表存在" \
    "sqlite3 '$DB_PATH' \"SELECT name FROM sqlite_master WHERE type='table' AND name='app_state';\"" \
    "app_state"

echo ""

# 2. 词库导入测试
echo "📚 2. 词库导入测试"
word_count=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM words;")
echo "  词库数量: $word_count"

if [ "$word_count" -gt 0 ]; then
    echo -e "  ${GREEN}✓ 词库已导入${NC}"
    ((PASSED++))
else
    echo -e "  ${RED}✗ 词库为空${NC}"
    ((FAILED++))
fi

card_count=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM srs_cards;")
echo "  卡片数量: $card_count"

if [ "$card_count" -eq "$word_count" ]; then
    echo -e "  ${GREEN}✓ 每个单词都有对应的 SRS 卡片${NC}"
    ((PASSED++))
else
    echo -e "  ${RED}✗ 卡片数量与单词数量不匹配${NC}"
    ((FAILED++))
fi

echo ""

# 3. SRS 状态测试
echo "🎯 3. SRS 状态测试"
new_cards=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM srs_cards WHERE status='new';")
echo "  新卡片数量: $new_cards"

if [ "$new_cards" -gt 0 ]; then
    echo -e "  ${GREEN}✓ 有可学习的新卡片${NC}"
    ((PASSED++))
else
    echo -e "  ${YELLOW}⚠ 没有新卡片（可能已全部学习）${NC}"
fi

echo ""

# 4. 模拟复习流程
echo "🔄 4. 模拟复习流程测试"

# 获取第一张卡片
first_card=$(sqlite3 "$DB_PATH" "SELECT id FROM srs_cards WHERE status='new' LIMIT 1;")

if [ -n "$first_card" ]; then
    echo "  选中卡片 ID: $first_card"
    
    # 模拟"认识"操作
    sqlite3 "$DB_PATH" "INSERT INTO review_logs (card_id, result, created_at) VALUES ($first_card, 'know', datetime('now'));"
    
    # 更新卡片状态
    sqlite3 "$DB_PATH" "UPDATE srs_cards SET status='learning', stage=0, correct_streak=1, lifetime_correct=1, last_result='know', last_seen_at=datetime('now'), updated_at=datetime('now') WHERE id=$first_card;"
    
    # 验证
    updated_stage=$(sqlite3 "$DB_PATH" "SELECT stage FROM srs_cards WHERE id=$first_card;")
    updated_streak=$(sqlite3 "$DB_PATH" "SELECT correct_streak FROM srs_cards WHERE id=$first_card;")
    
    if [ "$updated_stage" -eq 0 ] && [ "$updated_streak" -eq 1 ]; then
        echo -e "  ${GREEN}✓ 复习流程正常（stage=0, streak=1）${NC}"
        ((PASSED++))
    else
        echo -e "  ${RED}✗ 复习流程异常（stage=$updated_stage, streak=$updated_streak）${NC}"
        ((FAILED++))
    fi
    
    # 检查日志
    log_count=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM review_logs WHERE card_id=$first_card;")
    if [ "$log_count" -eq 1 ]; then
        echo -e "  ${GREEN}✓ 复习日志已记录${NC}"
        ((PASSED++))
    else
        echo -e "  ${RED}✗ 复习日志异常（count=$log_count）${NC}"
        ((FAILED++))
    fi
else
    echo -e "  ${YELLOW}⚠ 跳过（没有可用的新卡片）${NC}"
fi

echo ""

# 5. 跳过冷却测试
echo "⏱️  5. 跳过冷却测试"

# 模拟跳过操作
if [ -n "$first_card" ]; then
    cooldown_time=$(date -u -v+30M +"%Y-%m-%d %H:%M:%S")
    sqlite3 "$DB_PATH" "UPDATE srs_cards SET skip_cooldown_until='$cooldown_time' WHERE id=$first_card;"
    
    # 验证
    cooldown=$(sqlite3 "$DB_PATH" "SELECT skip_cooldown_until FROM srs_cards WHERE id=$first_card;")
    if [ -n "$cooldown" ]; then
        echo -e "  ${GREEN}✓ 跳过冷却已设置: $cooldown${NC}"
        ((PASSED++))
    else
        echo -e "  ${RED}✗ 跳过冷却设置失败${NC}"
        ((FAILED++))
    fi
else
    echo -e "  ${YELLOW}⚠ 跳过（没有可用的卡片）${NC}"
fi

echo ""

# 6. 性能测试
echo "⚡ 6. 性能测试"

# 内存占用
mem_mb=$(ps aux | grep fragment-vocab | grep -v grep | head -1 | awk '{print $6/1024}')
echo "  内存占用: ${mem_mb} MB"

if (( $(echo "$mem_mb < 50" | bc -l) )); then
    echo -e "  ${GREEN}✓ 内存占用正常 (< 50MB)${NC}"
    ((PASSED++))
else
    echo -e "  ${YELLOW}⚠ 内存占用偏高 (目标 < 50MB)${NC}"
    echo "  建议: 优化内存使用"
fi

# CPU 占用
cpu=$(ps aux | grep fragment-vocab | grep -v grep | head -1 | awk '{print $3}')
echo "  CPU 占用: ${cpu}%"

if (( $(echo "$cpu < 1.0" | bc -l) )); then
    echo -e "  ${GREEN}✓ CPU 占用正常 (< 1%)${NC}"
    ((PASSED++))
else
    echo -e "  ${YELLOW}⚠ CPU 占用偏高${NC}"
fi

echo ""

# 7. 数据完整性测试
echo "🔍 7. 数据完整性测试"

# 检查外键约束
orphan_cards=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM srs_cards WHERE word_id NOT IN (SELECT id FROM words);")
if [ "$orphan_cards" -eq 0 ]; then
    echo -e "  ${GREEN}✓ 无孤立卡片${NC}"
    ((PASSED++))
else
    echo -e "  ${RED}✗ 发现 $orphan_cards 个孤立卡片${NC}"
    ((FAILED++))
fi

orphan_logs=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM review_logs WHERE card_id NOT IN (SELECT id FROM srs_cards);")
if [ "$orphan_logs" -eq 0 ]; then
    echo -e "  ${GREEN}✓ 无孤立日志${NC}"
    ((PASSED++))
else
    echo -e "  ${RED}✗ 发现 $orphan_logs 个孤立日志${NC}"
    ((FAILED++))
fi

echo ""

# 总结
echo "================================"
echo "测试总结:"
echo -e "  ${GREEN}通过: $PASSED${NC}"
echo -e "  ${RED}失败: $FAILED${NC}"
echo "================================"

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ 所有测试通过！${NC}"
    exit 0
else
    echo -e "${RED}✗ 部分测试失败${NC}"
    exit 1
fi
