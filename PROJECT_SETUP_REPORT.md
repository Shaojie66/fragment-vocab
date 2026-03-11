# 碎片时间背单词工具 - 项目管理报告

## 执行时间
2026-03-12 01:35

## 完成情况

### ✅ 标签体系创建完成
- epic:shell (应用骨架与窗口管理)
- epic:database (数据库与存储)
- epic:domain (领域逻辑与算法)
- epic:ui (交互与界面)
- priority:high/medium/low
- status:todo/in-progress/done

### ✅ 里程碑规划完成
1. **MVP** - 2026-03-26 (2周内)
   - 核心功能完成，可进行基础测试
   
2. **Beta** - 2026-04-12 (1个月内)
   - 内测版本，功能完整可用
   
3. **v1.0** - 2026-05-12 (2个月内)
   - 正式发布版本

### ✅ GitHub Issues 创建完成 (18个)

#### Epic 1: Shell 层 (4个)
1. #1 初始化 Tauri 项目并配置 macOS 菜单栏模式 [high, MVP]
2. #2 创建浮卡窗口和统计窗口 [high, MVP]
3. #3 实现开机启动功能 [medium, Beta]
4. #4 配置日志与基础错误处理 [medium, MVP]

#### Epic 2: Database 层 (4个)
5. #5 建立 SQLite 连接与 migration 机制 [high, MVP]
6. #6 创建数据库表结构(001_init.sql) [high, MVP]
7. #7 编写词库导入脚本 [high, MVP]
8. #8 增加词库校验脚本 [low, Beta]

#### Epic 3: Domain 层 (5个)
9. #9 实现 SRS 复习算法引擎(srsEngine.ts) [high, MVP]
10. #10 实现选词逻辑(cardSelector.ts) [high, MVP]
11. #11 实现触发调度器(triggerScheduler.ts) [high, MVP]
12. #12 为 SRS 算法编写单元测试 [medium, MVP]
13. #13 为选词逻辑编写单元测试 [medium, MVP]

#### Epic 4: UI 层 (5个)
14. #14 开发单词卡 UI 组件 [high, MVP]
15. #15 实现 idle 检测功能 [high, MVP]
16. #16 实现 submit_review 命令 [high, MVP]
17. #17 实现菜单栏统计摘要 [medium, MVP]
18. #18 实现暂停与静默控制 [medium, MVP]

### ✅ 项目看板创建完成
- 项目名称: 碎片时间背单词 - 开发看板
- 项目地址: https://github.com/users/Shaojie66/projects/1
- 看板列: Todo / In Progress / Done
- 所有 18 个 Issue 已添加到看板并设置为 Todo 状态

## 优先级分布
- **High**: 11 个 (核心功能)
- **Medium**: 6 个 (辅助功能)
- **Low**: 1 个 (优化项)

## 里程碑分布
- **MVP**: 15 个 Issue
- **Beta**: 3 个 Issue

## 下一步建议

### 立即开始 (本周)
1. #1 初始化 Tauri 项目
2. #5 建立 SQLite 连接
3. #6 创建数据库表结构

### 第一周重点
- 完成 Shell 层基础框架
- 完成 Database 层核心功能
- 开始 Domain 层算法实现

### 风险提示
1. Tauri v2 在 macOS 上的菜单栏行为需要验证
2. idle 检测的系统 API 调用需要测试稳定性
3. 词库质量直接影响首版体验，需要提前准备

## 项目链接
- 仓库: https://github.com/Shaojie66/fragment-vocab
- 看板: https://github.com/users/Shaojie66/projects/1
- Issues: https://github.com/Shaojie66/fragment-vocab/issues
