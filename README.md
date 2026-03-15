# Fragment Vocab

> 利用碎片时间学习英语单词的桌面应用

Fragment Vocab 是一款智能单词学习工具，通过在你的空闲时间自动弹出单词卡片，帮助你利用碎片时间高效记忆单词。

## ✨ 主要特性

- 🎯 **智能调度** - 根据系统空闲时间自动弹出单词卡片
- ⏰ **灵活提醒** - 支持克制、平衡、强化和自定义提醒模式
- 📚 **词库管理** - 支持导入 JSON/CSV/TXT/XLSX 格式的自定义词库
- 🔄 **间隔重复** - 基于记忆曲线的智能复习算法
- 📊 **学习统计** - 实时追踪学习进度和正确率
- 🎨 **简洁界面** - 现代化的卡片式学习界面
- ⌨️ **快捷键支持** - Cmd/Ctrl+Shift+1/2/3/4 快速选择答案

## 🛠️ 技术栈

- **前端框架**: TypeScript + Vanilla JS
- **桌面框架**: Tauri 2.0
- **后端**: Rust
- **数据库**: SQLite
- **构建工具**: Vite

## 🚀 快速开始

### 环境要求

- Node.js 18+
- Rust 1.70+
- pnpm (推荐) 或 npm

### 安装依赖

```bash
pnpm install
```

### 开发模式

```bash
pnpm tauri dev
```

### 构建应用

```bash
pnpm tauri build
```

## 📖 使用说明

### 导入词库

支持以下格式的词库文件：

**JSON 格式**
```json
[
  {
    "word": "abandon",
    "meaning_zh": "放弃；抛弃"
  }
]
```

**CSV 格式**
```csv
word,meaning_zh
abandon,放弃；抛弃
```

**TXT 格式**
```
abandon 放弃；抛弃
```

### 提醒模式

- **克制模式**: 空闲 180 秒后提醒，兜底间隔 45 分钟
- **平衡模式**: 空闲 120 秒后提醒，兜底间隔 30 分钟
- **强化模式**: 空闲 90 秒后提醒，兜底间隔 20 分钟
- **自定义模式**: 自由设置空闲时间和兜底间隔

### 快捷键

- `Cmd/Ctrl + Shift + 1/2/3/4`: 选择答案选项
- `Cmd/Ctrl + Shift + Esc`: 跳过当前卡片

## 🔧 开发

### 项目结构

```
fragment-vocab/
├── src/                  # 前端源码
│   ├── main.ts          # 主窗口逻辑
│   ├── card.ts          # 卡片窗口逻辑
│   └── domain/          # 业务逻辑
├── src-tauri/           # Rust 后端
│   └── src/
│       ├── commands.rs  # Tauri 命令
│       ├── db/          # 数据库层
│       └── lib.rs       # 主入口
└── assets/              # 资源文件
```

### 运行测试

```bash
pnpm test
```

## 📝 License

MIT

