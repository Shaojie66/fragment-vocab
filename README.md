# Fragment Vocab

<div align="center">

**碎片时间背单词工具 - macOS 菜单栏应用**

一款专为 macOS 设计的轻量级背单词应用，让你在工作间隙轻松记单词，无需专门安排学习时间。

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue.svg)](https://tauri.app/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.6-blue.svg)](https://www.typescriptlang.org/)

[功能特性](#功能特性) • [快速开始](#快速开始) • [使用说明](#使用说明) • [开发指南](#开发指南) • [贡献](#贡献)

</div>

---

## 项目简介

Fragment Vocab 是一款基于 **间隔重复算法（SRS）** 的智能背单词工具，专为利用碎片时间学习设计。

### 核心理念

传统背单词应用需要主动打开、进入学习页、开始任务，启动成本高，导致高频但短时的学习机会被浪费。

Fragment Vocab 将"背单词"从主动任务改成被动插入：
- 🎯 **智能触发**：系统检测到你空闲时自动弹出单词卡
- ⚡ **极速作答**：一次点击或快捷键即可完成学习
- 🔄 **科学复习**：基于 SRS 算法自动安排复习计划
- 🚫 **零打扰**：夜间静默、手动暂停、智能冷却

### 适用场景

✅ 工作间隙的短暂休息  
✅ 等待编译/构建的时间  
✅ 思考问题的间隙  
✅ 喝水、上厕所回来后  

## 功能特性

### 🎯 智能弹卡系统

- **自动触发**：检测到 90 秒空闲时自动弹出单词卡
- **兜底机制**：距离上次学习超过 25 分钟时提醒
- **免打扰模式**：夜间静默（23:00-07:00）、手动暂停、跳过冷却

### 📚 科学复习算法

采用固定间隔重复（SRS）算法：

```
新词 → 10分钟 → 1天 → 3天 → 7天 → 14天 → 已掌握
```

- ✅ 答对：进入下一复习阶段
- ❌ 答错：回退到更早阶段
- ⏭️ 跳过：不计入正确率，短时间后再次出现

### ⌨️ 快捷键操作

| 操作 | 快捷键 | 说明 |
|------|--------|------|
| 认识 | `Cmd + K` | 标记为认识，进入下一阶段 |
| 不认识 | `Cmd + J` | 标记为不认识，回退复习 |
| 跳过 | `Esc` | 跳过当前单词 |

### 📊 学习统计

- **今日数据**：已学词数、正确率、新词数量
- **累计数据**：待复习词数、已掌握词数、学习进度
- **详细统计页**：展示次数、认识/不认识/跳过次数

### 🎛️ 灵活控制

- ⏸️ **暂停 1 小时**：临时关闭弹卡
- 🌙 **今日不再提醒**：今天不再弹出
- 📈 **统计页面**：查看详细学习数据
- ⚙️ **自定义设置**：调整学习参数

## 快速开始

### 系统要求

- macOS 10.15 (Catalina) 或更高版本
- 约 50MB 可用磁盘空间

### 安装

1. 从 [Releases](https://github.com/Shaojie66/fragment-vocab/releases) 下载最新版本
2. 解压并拖动到 Applications 文件夹
3. 首次启动时允许系统权限请求
4. 应用会自动常驻菜单栏并设置开机启动

### 首次使用

1. 在菜单栏找到应用图标（📚）
2. 等待 90 秒不操作电脑
3. 屏幕右上角会弹出第一张单词卡
4. 选择"认识"、"不认识"或"跳过"

就这么简单！

## 使用说明

### 单词卡展示

每张单词卡包含：
- 英文单词
- 音标
- 词性
- 中文释义

示例：

```
abandon
/əˈbændən/
v. 放弃；抛弃

[认识] [不认识] [跳过]
```

### 菜单栏功能

点击菜单栏图标可以：

- 📊 查看今日统计（已学词数、正确率、新词数量）
- ⏸️ 暂停 1 小时
- 🌙 今日不再提醒
- 📈 打开统计页
- ⚙️ 设置
- ❌ 退出应用

### 学习建议

**每日新词量**
- 默认：15 个新词/天
- 建议：根据自己的时间调整，宁少勿多

**复习优先**
- 系统优先展示到期复习词
- 确保旧词不遗忘

**诚实作答**
- 不确定就选"不认识"
- 不要为了正确率而作弊

详细使用说明请查看 [用户文档](./docs/user-guide.md)。

## 技术栈

- **桌面框架**：[Tauri 2](https://tauri.app/) - 轻量级跨平台桌面应用框架
- **前端**：[Vite](https://vitejs.dev/) + [TypeScript](https://www.typescriptlang.org/)
- **后端**：Rust - 系统能力桥接（idle 检测、菜单栏、快捷键）
- **数据库**：SQLite - 本地数据存储
- **测试**：[Vitest](https://vitest.dev/) - 单元测试框架

### 核心模块

- **SRS 引擎**：间隔重复算法实现
- **卡片选择器**：智能选词逻辑
- **触发调度器**：idle 检测与弹卡控制
- **统计系统**：学习数据追踪与展示

## 开发指南

### 环境准备

1. **安装依赖**

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Node.js (推荐使用 nvm)
nvm install 22
nvm use 22

# 克隆仓库
git clone https://github.com/Shaojie66/fragment-vocab.git
cd fragment-vocab

# 安装项目依赖
npm install
```

2. **Tauri 依赖**

```bash
# macOS 需要安装 Xcode Command Line Tools
xcode-select --install
```

### 开发命令

```bash
# 启动开发服务器（热重载）
npm run tauri dev

# 运行测试
npm test

# 运行测试（watch 模式）
npm run test:watch

# 构建应用
npm run tauri build

# 代码检查
npm run lint

# 代码格式化
npm run format
```

### 项目结构

```
fragment-vocab/
├── src/                    # 前端源码
│   ├── app/               # 应用入口与路由
│   ├── data/              # 数据层（SQLite、API）
│   ├── domain/            # 业务逻辑（SRS、调度）
│   ├── features/          # 功能模块（单词卡、统计）
│   └── shared/            # 共享组件与工具
├── src-tauri/             # Tauri 后端
│   ├── src/               # Rust 源码
│   │   ├── main.rs       # 入口
│   │   ├── idle.rs       # 空闲检测
│   │   └── tray.rs       # 菜单栏
│   └── Cargo.toml        # Rust 依赖
├── docs/                  # 项目文档
│   ├── mvp-prd.md        # 产品需求文档
│   ├── technical-plan.md # 技术方案
│   ├── user-guide.md     # 用户指南
│   └── ...
├── scripts/               # 构建脚本
└── tests/                 # 测试文件
```

### 调试

- **前端调试**：在 Tauri 窗口中按 `Cmd+Option+I` 打开开发者工具
- **Rust 调试**：使用 `println!` 或配置 VS Code 调试器
- **日志查看**：`~/Library/Application Support/fragment-vocab/logs/`

### 测试

```bash
# 运行所有测试
npm test

# 运行特定测试文件
npm test -- src/domain/srs.test.ts

# 生成测试覆盖率报告
npm test -- --coverage
```

## 贡献

欢迎各种形式的贡献！

### 如何贡献

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'feat: add some amazing feature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

详细贡献指南请查看 [CONTRIBUTING.md](./CONTRIBUTING.md)。

### 报告问题

如果你发现了 bug 或有功能建议，请在 [Issues](https://github.com/Shaojie66/fragment-vocab/issues) 中提交。

## 路线图

### v0.1.0 (MVP) ✅
- ✅ 基础单词卡展示
- ✅ 三种回答方式
- ✅ 间隔重复算法
- ✅ 菜单栏常驻
- ✅ 今日统计
- ✅ 暂停与静默控制

### 计划中的功能
- 🔜 自定义词库导入
- 🔜 暗色模式
- 🔜 更丰富的统计图表
- 🔜 学习提醒通知
- 🔜 成就系统
- 🔜 多设备同步

## 常见问题

**Q: 为什么没有弹出单词卡？**

可能原因：
- 没有满足 90 秒 idle 条件
- 当前在夜间静默时段（23:00-07:00）
- 已手动暂停
- 今日新词配额已用完且无待复习词

**Q: 数据存储在哪里？**

所有数据存储在本地 SQLite 数据库：
- 位置：`~/Library/Application Support/fragment-vocab/`
- 包含：词库、学习记录、统计数据
- 不会上传到云端

**Q: 会影响电脑性能吗？**

不会。应用非常轻量：
- 内存占用：< 50MB
- CPU 占用：几乎为 0（idle 时）
- 磁盘占用：< 10MB

更多问题请查看 [FAQ](./docs/faq.md)。

## 许可证

本项目采用 [MIT License](./LICENSE) 开源。

## 致谢

感谢所有为本项目做出贡献的开发者和用户。

特别感谢：
- [Tauri](https://tauri.app/) - 优秀的桌面应用框架
- [Vite](https://vitejs.dev/) - 快速的前端构建工具
- 雅思词汇资源提供者
- 测试用户的宝贵反馈

---

<div align="center">

**祝你学习愉快！🎉**

如有问题，欢迎提交 [Issue](https://github.com/Shaojie66/fragment-vocab/issues) 或查阅[文档](./docs/)。

Made with ❤️ by [Shaojie Chen](https://github.com/Shaojie66)

</div>
