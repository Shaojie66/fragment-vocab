# Fragment Vocab - 碎片时间背单词工具

一款 macOS 菜单栏常驻应用,利用系统空闲时间自动弹出单词卡片,让背单词融入工作节奏。

## 核心特性

- 🎯 **零启动成本** - 系统空闲时自动弹出,无需主动打开
- ⚡ **3秒完成学习** - 一次点击或快捷键即可完成单词判断
- 🔄 **智能复习** - 基于 SRS 算法自动安排复习计划
- 🌙 **免打扰模式** - 夜间静默、手动暂停、跳过冷却
- 📊 **简洁统计** - 今日学习数据一目了然

## 技术栈

- **桌面框架**: Tauri v2
- **前端**: TypeScript + Vue 3
- **数据存储**: SQLite
- **构建工具**: Vite
- **测试**: Vitest

## 快速开始

### 环境要求

- macOS 12.0+
- Node.js 18+
- Rust 1.70+

### 开发

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run tauri dev

# 运行测试
npm test
```

### 构建

```bash
# 构建 macOS 应用
npm run tauri build

# 生成 .dmg 安装包
npm run build:dmg
```

## 使用说明

### 基本操作

- **认识**: `Cmd+K` 或点击"认识"按钮
- **不认识**: `Cmd+J` 或点击"不认识"按钮
- **跳过**: `Esc` 或点击"跳过"按钮

### 菜单栏功能

- 查看今日学习统计
- 暂停 1 小时
- 今日不再提醒
- 打开详细统计页

### 触发规则

- 系统空闲 >= 90 秒时自动弹出
- 距离上次展示 >= 25 分钟时兜底触发
- 夜间 23:00-07:00 自动静默

### 复习阶段

- Stage 0: 10 分钟
- Stage 1: 1 天
- Stage 2: 3 天
- Stage 3: 7 天
- Stage 4: 14 天
- Mastered: 已掌握

## 项目结构

```
fragment-vocab/
├── src/                    # 前端源码
│   ├── app/               # 应用入口
│   ├── data/              # 数据层
│   ├── domain/            # 业务逻辑
│   ├── features/          # 功能模块
│   └── shared/            # 共享组件
├── src-tauri/             # Tauri 后端
│   └── src/               # Rust 源码
├── docs/                  # 项目文档
├── scripts/               # 构建脚本
└── assets/                # 静态资源
```

## 开发文档

- [MVP PRD](./docs/mvp-prd.md) - 产品需求文档
- [技术方案](./docs/technical-plan.md) - 技术架构与任务拆解

## 贡献指南

请查看 [CONTRIBUTING.md](./CONTRIBUTING.md)

## 许可证

MIT License - 详见 [LICENSE](./LICENSE)

## 路线图

### MVP (v0.1)
- [x] 数据库设计
- [ ] 菜单栏常驻
- [ ] 空闲检测
- [ ] 单词卡弹出
- [ ] SRS 复习算法
- [ ] 基础统计

### 后续版本
- [ ] 词库编辑
- [ ] 多词库支持
- [ ] 详细统计图表
- [ ] 成就系统
- [ ] 云同步

## 反馈与支持

- 提交 Issue: [GitHub Issues](https://github.com/Shaojie66/fragment-vocab/issues)
- 讨论: [GitHub Discussions](https://github.com/Shaojie66/fragment-vocab/discussions)
