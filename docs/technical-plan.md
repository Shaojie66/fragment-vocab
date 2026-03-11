# 碎片时间背单词工具 技术方案与任务拆解

## 1. 文档信息
- 版本：v0.1
- 状态：开发前
- 技术路线：Tauri + TypeScript + SQLite
- 运行平台：macOS

## 2. 技术目标
用最小工程复杂度实现以下闭环：

1. 常驻菜单栏运行
2. 检测系统 idle
3. 在满足条件时弹出浮卡
4. 完成一次作答
5. 更新复习计划和统计
6. 支持暂停与静默控制

## 3. 总体架构
系统分为 5 个模块：

1. Shell 层
负责菜单栏、窗口、快捷键、启动与退出。

2. Scheduler 层
负责 idle 检测、触发判断、冷却控制、选词与调度。

3. SRS 层
负责复习阶段流转、到期时间计算、掌握判定。

4. Data 层
负责 SQLite 连接、schema migration、查询与写入。

5. UI 层
负责单词卡、菜单栏面板、统计页。

## 4. 推荐技术选型
### 4.1 桌面壳
- Tauri v2
- Rust 侧负责窗口与系统能力桥接
- 前端 UI 使用 TypeScript

### 4.2 前端
- Vue 3 或 React 二选一
- 首版建议 Vue 3，界面简单、开发速度快

### 4.3 存储
- SQLite
- 本地文件存储于用户目录应用数据路径

### 4.4 状态管理
- 轻量 store 即可，不引入重状态框架
- UI 状态与数据库状态分离

### 4.5 测试
- Vitest：算法与调度单测
- 手工验证：窗口、快捷键、菜单栏交互

## 5. 核心模块设计
### 5.1 App Shell
职责：
1. 创建菜单栏图标与菜单
2. 创建浮卡窗口和统计窗口
3. 管理窗口显示、隐藏、置顶
4. 处理开机启动和退出

关键要求：
1. 浮卡窗口轻量、无边框、固定尺寸
2. 默认不出现在 Dock
3. 菜单栏可随时打开统计页

### 5.2 Idle Detector
职责：
1. 获取系统 idle 秒数
2. 向 Scheduler 提供当前空闲状态

建议：
- 统一封装为 `getIdleSeconds(): Promise<number>`

### 5.3 Trigger Scheduler
职责：
1. 定时轮询系统状态
2. 判断是否满足弹卡条件
3. 计算当前应该展示哪个卡片
4. 避免重复弹出

推荐轮询频率：
- 每 15 秒执行一次调度检查

调度判断顺序：
1. 是否已暂停
2. 是否在夜间静默
3. 是否已有浮卡展示
4. 是否处于跳过冷却
5. 是否满足 idle 条件
6. 是否有可展示卡片

### 5.4 Card Selector
职责：
1. 获取到期复习词
2. 获取今日可引入新词
3. 按优先级返回 1 张卡

优先级规则：
1. 到期复习词按 `due_at ASC`
2. 新词按 `words.id ASC` 或词库默认顺序
3. 已跳过且仍在冷却中的卡不返回

### 5.5 SRS Engine
职责：
1. 计算答对后的下一阶段
2. 计算答错后的回退阶段
3. 计算下一次 `due_at`
4. 计算是否变为 `mastered`

接口建议：
- `applyReview(card, result, now): ReviewUpdate`
- `getNextDueAt(stage, now): string`
- `isMastered(card): boolean`

### 5.6 Stats Service
职责：
1. 统计今日展示次数
2. 统计今日认识率
3. 统计今日新词数
4. 统计累计掌握数

原则：
- 首版允许实时 SQL 聚合
- 数据量很小，无需预计算体系

## 6. SQLite 数据库设计
### 6.1 表清单
1. `words`
2. `srs_cards`
3. `review_logs`
4. `app_state`

### 6.2 建表 SQL
```sql
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS words (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  word TEXT NOT NULL UNIQUE,
  phonetic TEXT,
  part_of_speech TEXT,
  meaning_zh TEXT NOT NULL,
  source TEXT DEFAULT 'ielts-core',
  difficulty INTEGER DEFAULT 1,
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

CREATE INDEX IF NOT EXISTS idx_srs_cards_status_due_at
ON srs_cards(status, due_at);

CREATE INDEX IF NOT EXISTS idx_srs_cards_skip_cooldown_until
ON srs_cards(skip_cooldown_until);

CREATE INDEX IF NOT EXISTS idx_review_logs_shown_at
ON review_logs(shown_at);
```

### 6.3 字段说明
#### words
- `word`：英文单词
- `phonetic`：音标
- `part_of_speech`：词性
- `meaning_zh`：中文义
- `source`：词库来源
- `difficulty`：预留难度分层

#### srs_cards
- `status`：new / learning / review / mastered
- `stage`：-1 表示未进入复习链；0-4 表示复习阶段
- `due_at`：下次应展示时间
- `last_result`：最近一次结果
- `skip_cooldown_until`：跳过后冷却截止时间

#### review_logs
- `result`：作答结果
- `trigger_type`：由哪种触发策略弹出
- `response_ms`：用户响应时长

#### app_state
建议存储这些 key：
- `pause_until`
- `silence_until_end_of_day`
- `today_new_count_YYYYMMDD`
- `last_popup_at`
- `last_popup_trigger_type`

## 7. 复习算法定义
### 7.1 阶段间隔
```ts
const SRS_STAGES = [
  { stage: 0, delayMs: 10 * 60 * 1000 },
  { stage: 1, delayMs: 24 * 60 * 60 * 1000 },
  { stage: 2, delayMs: 3 * 24 * 60 * 60 * 1000 },
  { stage: 3, delayMs: 7 * 24 * 60 * 60 * 1000 },
  { stage: 4, delayMs: 14 * 24 * 60 * 60 * 1000 },
]
```

### 7.2 状态更新规则
#### know
1. 若为新词，置为 `learning`
2. `stage = min(stage + 1, 4)`；新词从 `-1 -> 0`
3. `correct_streak += 1`
4. `lifetime_correct += 1`
5. 更新 `due_at`

#### dont_know
1. 若当前 `stage > 0`，则 `stage -= 1`
2. 若当前 `stage <= 0`，则保持 `0`
3. `status = learning`
4. `correct_streak = 0`
5. `lifetime_wrong += 1`
6. 更新 `due_at`

#### skip
1. 不改变 `stage`
2. 不计入正确率
3. `skip_cooldown_until = now + 30 min`

#### mastered
满足以下条件可转为 `mastered`：
1. 当前已到达 `stage = 4`
2. 本次结果为 `know`
3. `correct_streak >= 2`

首版可简化为：
- 在 `stage = 4` 再答对 1 次即 mastered

## 8. 选词逻辑
### 8.1 复习优先
查询条件：
1. `status IN ('learning', 'review')`
2. `due_at <= now`
3. `skip_cooldown_until IS NULL OR skip_cooldown_until <= now`

排序：
- `due_at ASC`
- `id ASC`

### 8.2 新词补位
仅在无到期复习词时选新词。

条件：
1. `status = 'new'`
2. 今日新词数 < 每日新词上限
3. 不在跳过冷却

## 9. 目录结构建议
```text
project-root/
  src-tauri/
    src/
      main.rs
      tray.rs
      window.rs
      idle.rs
      autostart.rs
      db.rs
      commands/
        mod.rs
        app.rs
        review.rs
        stats.rs
    tauri.conf.json

  src/
    app/
      main.ts
      router.ts
      store/
        appStore.ts
        statsStore.ts

    features/
      tray/
        TrayMenu.vue
      flashcard/
        FlashcardWindow.vue
        flashcardService.ts
        flashcardShortcuts.ts
      stats/
        StatsPage.vue
        statsService.ts
      settings/
        settingsService.ts

    domain/
      srs/
        srsEngine.ts
        srsTypes.ts
      scheduler/
        triggerScheduler.ts
        cardSelector.ts
        triggerRules.ts
      words/
        wordTypes.ts

    data/
      db.ts
      migrations/
        001_init.sql
      repositories/
        wordsRepo.ts
        cardsRepo.ts
        logsRepo.ts
        stateRepo.ts

    shared/
      types/
      utils/
      constants/
      time/
      logger/

  scripts/
    import-wordbook.ts
    validate-wordbook.ts

  assets/
    wordbooks/
      ielts-core-3000.json

  docs/
    mvp-prd.md
    technical-plan.md
```

## 10. 核心接口建议
### 10.1 Rust -> Frontend Commands
- `get_app_summary()`
- `get_today_stats()`
- `show_flashcard()`
- `hide_flashcard()`
- `pause_for_one_hour()`
- `silence_for_today()`
- `submit_review(cardId, result, responseMs)`
- `get_next_card()`

### 10.2 Frontend 内部服务
- `flashcardService.open(card)`
- `flashcardService.close()`
- `statsService.fetchTodayStats()`
- `settingsService.pauseOneHour()`

## 11. 调度时序
1. 应用启动
2. 加载配置和数据库
3. 每 15 秒执行调度循环
4. 调用 idle detector
5. 检查静默、暂停、冷却
6. 若满足条件，取下一张卡
7. 展示卡片并记录 `shown_at`
8. 用户回答或超时
9. 写入 review log
10. 更新 card 状态
11. 刷新菜单栏统计

## 12. 首周任务拆解
### Day 1
1. 初始化 Tauri 工程
2. 创建菜单栏和基础窗口
3. 接入 SQLite
4. 跑通 migration

### Day 2
1. 导入词库脚本
2. 建立 Repository 层
3. 实现 SRS Engine
4. 写 SRS 单元测试

### Day 3
1. 实现 idle 检测
2. 实现 Trigger Scheduler
3. 实现 Card Selector
4. 跑通"自动拿到下一张卡"

### Day 4
1. 开发单词卡 UI
2. 接入 `认识 / 不认识 / 跳过`
3. 写 review 日志
4. 卡片自动消失和跳过逻辑

### Day 5
1. 菜单栏统计
2. 暂停与夜间静默
3. 统计页
4. 联调和手测

## 13. 首周 Issue 清单
### Epic 1：应用骨架
1. 初始化 Tauri 项目并配置 macOS 菜单栏模式
2. 创建浮卡窗口和统计窗口
3. 实现开机启动
4. 配置日志与基础错误处理

### Epic 2：数据库
5. 建立 SQLite 连接与 migration 机制
6. 创建 `001_init.sql`
7. 编写词库导入脚本
8. 增加词库校验脚本

### Epic 3：领域逻辑
9. 实现 `srsEngine.ts`
10. 实现 `cardSelector.ts`
11. 实现 `triggerRules.ts`
12. 为复习算法编写单元测试
13. 为选词逻辑编写单元测试

### Epic 4：交互闭环
14. 开发 Flashcard UI
15. 接入快捷键
16. 实现超时自动跳过
17. 实现 `submit_review` 命令
18. 写入 review_logs 和 srs_cards 更新

### Epic 5：设置与统计
19. 实现菜单栏 summary
20. 实现暂停 1 小时
21. 实现今日不再提醒
22. 实现统计页 SQL 聚合
23. 增加默认夜间静默配置

### Epic 6：联调与验证
24. 手测首次启动和建库
25. 手测 idle 触发
26. 手测连续跳过冷却
27. 手测暂停恢复
28. 修复窗口焦点和重复弹卡问题

## 14. 测试清单
### 单元测试
1. 新词答对后进入 stage 0
2. stage 2 答错后回退 stage 1
3. stage 0 答错后不低于 stage 0
4. stage 4 再答对后进入 mastered
5. 跳过后 30 分钟内不会再次出现
6. 有到期复习词时不会抽到新词

### 手工测试
1. 冷启动首次建库
2. 词库导入后卡片可正常展示
3. 快捷键可靠触发
4. 菜单栏数据与数据库一致
5. 暂停 1 小时后自动恢复
6. 静默时段内不弹卡

## 15. 风险项
1. Tauri 菜单栏与浮卡窗口在 macOS 上的行为差异
2. 快捷键监听方式可能受焦点影响
3. idle 检测桥接实现是否稳定
4. 开机启动行为在不同系统版本上的兼容性
5. 词库质量会直接影响首版体验

## 16. 建议的开发顺序
先把"能弹、能答、能记、能再来"做通，再做设置和统计。

优先级顺序：
1. 数据库
2. SRS 算法
3. 选词逻辑
4. 浮卡交互
5. idle 调度
6. 设置与统计
