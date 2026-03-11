# Fragment Vocab 手动测试指南

## 测试环境
- 应用已启动（PID: 84389）
- 数据库路径: `/Users/chenshaojie/Library/Application Support/com.chenshaojie.fragment-vocab/fragment-vocab.db`
- 当前词库: 5 个单词（abandon, ability, abroad, absence, absolute）

## 测试步骤

### 1. 快捷键测试
**目标**: 验证全局快捷键是否正常工作

1. **Cmd+Shift+K** - 应该触发"认识"
2. **Cmd+Shift+J** - 应该触发"不认识"  
3. **Cmd+Shift+Esc** - 应该触发"跳过"

**验证方法**:
- 按下快捷键后，观察是否弹出单词卡窗口
- 检查数据库 `review_logs` 表是否有新记录
- 检查 `srs_cards` 表的 `stage` 和 `status` 是否更新

### 2. Idle 触发测试（90秒）
**目标**: 验证空闲 90 秒后自动弹卡

1. 保持电脑空闲（不移动鼠标、不按键盘）
2. 等待 90 秒
3. 观察是否自动弹出单词卡

**验证方法**:
```bash
# 查看调度器日志（应该看到 "🎯 Triggering card display"）
# 查看卡片窗口是否可见
```

### 3. 兜底触发测试（25分钟）
**目标**: 验证 25 分钟后强制弹卡

1. 正常使用电脑（不让 idle 达到 90 秒）
2. 等待 25 分钟
3. 观察是否自动弹出单词卡

### 4. 交互流程测试

#### 4.1 认识流程
1. 触发单词卡（快捷键或等待自动触发）
2. 点击"认识"或按 Cmd+Shift+K
3. 验证：
   - 窗口自动关闭
   - 数据库记录 `result='know'`
   - `correct_streak` +1
   - `stage` 按 SRS 规则递增

#### 4.2 不认识流程
1. 触发单词卡
2. 点击"不认识"或按 Cmd+Shift+J
3. 验证：
   - 窗口自动关闭
   - 数据库记录 `result='dont_know'`
   - `correct_streak` 重置为 0
   - `stage` 重置为 0

#### 4.3 跳过流程
1. 触发单词卡
2. 点击"跳过"或按 Cmd+Shift+Esc 或等待 10-12 秒自动跳过
3. 验证：
   - 窗口自动关闭
   - 数据库记录 `result='skip'`
   - 进入 30 分钟冷却期（`skip_cooldown_until`）

### 5. 边界情况测试

#### 5.1 无待复习词
```sql
-- 将所有卡片标记为 mastered 且 due_at 设为未来
UPDATE srs_cards SET status='mastered', stage=8, due_at=datetime('now', '+7 days');
```
预期：调用 `get_next_card` 返回 null，不弹卡

#### 5.2 新词配额用完
```sql
-- 查看今日已学新词数
SELECT COUNT(*) FROM review_logs 
WHERE result IN ('know', 'dont_know') 
AND created_at >= date('now');
```
预期：当达到 15 个新词后，不再展示新词

#### 5.3 连续跳过
1. 连续跳过 3 次
2. 验证每次跳过后都进入 30 分钟冷却
3. 验证冷却期内不会再次弹卡

### 6. 夜间静默测试
**目标**: 验证 23:00-07:00 不弹卡

1. 修改系统时间到 23:30
2. 验证即使满足 idle 条件也不弹卡
3. 修改系统时间到 07:30
4. 验证恢复正常弹卡

### 7. 暂停功能测试
1. 点击托盘图标 -> "暂停 1 小时"
2. 验证 1 小时内不弹卡
3. 1 小时后验证恢复弹卡

### 8. 今日不再提醒测试
1. 点击托盘图标 -> "今日不再提醒"
2. 验证当天剩余时间不弹卡
3. 第二天验证恢复弹卡

### 9. 数据持久化测试
1. 完成几次复习
2. 退出应用
3. 重新启动应用
4. 验证：
   - 复习记录保留
   - SRS 状态保留
   - 统计数据正确

### 10. 性能测试
```bash
# 内存占用
ps aux | grep fragment-vocab | grep -v grep | awk '{print $6/1024 " MB"}'

# CPU 占用（idle 时应接近 0）
top -pid <PID> -l 1 | grep fragment-vocab

# 启动速度（从启动到显示托盘图标）
time npm run tauri dev
```

## 数据库查询命令

```bash
DB_PATH="/Users/chenshaojie/Library/Application Support/com.chenshaojie.fragment-vocab/fragment-vocab.db"

# 查看所有卡片状态
sqlite3 "$DB_PATH" "SELECT c.id, w.word, c.status, c.stage, c.due_at, c.last_result FROM srs_cards c JOIN words w ON c.word_id = w.id;"

# 查看复习日志
sqlite3 "$DB_PATH" "SELECT l.id, w.word, l.result, l.created_at FROM review_logs l JOIN srs_cards c ON l.card_id = c.id JOIN words w ON c.word_id = w.id ORDER BY l.created_at DESC LIMIT 10;"

# 查看今日统计
sqlite3 "$DB_PATH" "SELECT result, COUNT(*) FROM review_logs WHERE created_at >= date('now') GROUP BY result;"

# 查看应用状态
sqlite3 "$DB_PATH" "SELECT * FROM app_state;"
```

## 预期结果

### 成功标准
- ✅ 所有快捷键响应正常
- ✅ Idle 90秒触发正常
- ✅ 兜底 25分钟触发正常
- ✅ 认识/不认识/跳过流程正确
- ✅ SRS 阶段流转符合预期
- ✅ 每日新词配额生效
- ✅ 跳过冷却生效
- ✅ 夜间静默生效
- ✅ 暂停功能正常
- ✅ 数据持久化正常
- ✅ 内存占用 < 50MB
- ✅ CPU idle 时接近 0%

### 当前已知问题
1. 内存占用 125MB（超出目标 50MB）
2. 编译警告（6个未使用的方法）
