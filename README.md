# Fragment Vocab

A smart desktop vocabulary learning application that automatically pops up word cards during your idle time, helping you memorize English words using fragmented time with zero interruption to your workflow.

[English](#features) | [中文介绍](#中文介绍)

---

## Features

### Intelligent Scheduling
- **System Idle Detection**: Monitors keyboard/mouse activity via Core Graphics (macOS)
- **Adaptive Reminders**:弹出时机基于你的空闲时间，而非固定间隔
- **Quiet Hours**: 设置勿扰时段，自动暂停提醒
- **Multiple Modes**: Gentle / Balanced / Intensive / Custom

### Spaced Repetition System (FSRS)
- Implements the [Free Spaced Repetition Scheduler Algorithm](https://github.com/open-spaced-repetition/fsrs) for optimal memory retention
- Automatically adjusts review intervals based on your performance
- Tracks memory strength and difficulty for each word

### Wordbook Management
- **Built-in Wordbook**: IELTS Core 3000 words included out of the box
- **Custom Import**: JSON / CSV / TXT / XLSX formats supported
- **Flexible Fields**: 自动识别多种字段名称（word/单词/英文、meaning_zh/中文/释义等）
- **Per-Wordbook Toggle**: Enable/disable specific wordbooks

### Pet Companion
- Cute pixel slime companion that grows with your learning progress
- 5 evolution stages (Egg → Hatchling → Juvenile → Adult → Fully-Evolved)
- Vitality multiplier system based on study streaks
- Visual health indicator affected by learning consistency
- Celebrates when you complete reviews

### Statistics & Progress
- Daily/weekly/monthly review stats
- Accuracy tracking
- Learning streak monitoring
- Achievement system (9 achievements)
- Data export/import for backup

### User Experience
- **Multi-Window Architecture**: Dashboard, flashcard popup, statistics - each in separate windows
- **Non-Intrusive**: Card window stays on top but doesn't steal focus
- **Keyboard Shortcuts**: `Cmd/Ctrl+Shift+1-4` for answers, `Esc` to skip
- **Auto-Pronunciation**: Optional text-to-speech for words
- **Dark/Light Mode**: Follows system preference or manual override
- **macOS Native**: Menu bar tray support, global shortcuts, autostart

---

## Architecture

```
fragment-vocab/
├── src/                          # Frontend (TypeScript + Vite)
│   ├── main/                     # Main window (dashboard, settings, wordbooks)
│   │   ├── dashboard.ts          # Dashboard rendering
│   │   ├── settings.ts          # Settings management
│   │   ├── wordbooks.ts         # Wordbook CRUD
│   │   └── events.ts           # Window event handling
│   ├── card/                     # Card window (flashcard popup)
│   │   └── card.ts              # Card logic + pet integration
│   ├── domain/                   # Business logic
│   │   ├── scheduler/           # TriggerScheduler (idle detection)
│   │   ├── srs/                # FSRS algorithm implementation
│   │   └── words/              # CardSelector (word prioritization)
│   ├── shared/                  # Shared utilities
│   │   ├── types/              # TypeScript interfaces
│   │   ├── theme.ts            # Theme management
│   │   └── update-checker.ts    # GitHub releases checker
│   └── pet.ts                   # Pet window entry
│
├── src-tauri/                    # Backend (Rust)
│   └── src/
│       ├── commands/            # Tauri command handlers
│       │   ├── review.rs        # Card selection, review submission
│       │   ├── config.rs        # App configuration
│       │   ├── pet.rs          # Pet state management
│       │   ├── achievements.rs # Achievement tracking
│       │   ├── wordbook.rs     # Wordbook import/export
│       │   └── ...
│       ├── db/                  # Database layer
│       │   ├── repositories/    # Repository pattern
│       │   ├── models.rs       # Data models
│       │   ├── migration.rs    # SQLite migrations
│       │   └── importer.rs     # Wordbook file importer
│       ├── pet/                 # Pet engine
│       │   └── engine.rs       # Pet calculations (evolution, health, XP)
│       └── idle.rs             # Platform-specific idle detection
│
├── assets/                      # Static assets
│   └── wordbooks/              # Built-in wordbooks (JSON)
│
└── docs/                       # Documentation
```

### Multi-Window Communication

Windows coordinate via Tauri events:

| Event | Direction | Purpose |
|-------|-----------|---------|
| `card-window-shown` | Backend → Frontend | Triggers card load + pet animation |
| `card-window-hidden` | Backend → Frontend | Triggers pet reveal |
| `card-hidden` | Frontend → Backend | Notifies card dismissed |
| `pet-state-updated` | Backend → Frontend | Updates pet visuals |
| `study-completed` | Backend → Frontend | Triggers pet celebration |

---

## Installation

### Requirements
- **Node.js**: 18+
- **Rust**: 1.70+
- **pnpm** (recommended) or npm
- **macOS**: Required for idle detection via Core Graphics

### From Source

```bash
# Clone the repository
git clone https://github.com/Shaojie66/fragment-vocab.git
cd fragment-vocab

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build

# Build macOS app bundle
pnpm run build:mac:app
```

### Pre-built Releases

Download from [GitHub Releases](https://github.com/Shaojie66/fragment-vocab/releases).

---

## Usage Guide

### First Launch

1. App opens to onboarding wizard
2. Set daily new word goal (default: 10)
3. Choose reminder mode (Gentle recommended for beginners)
4. App imports IELTS Core 3000 wordbook automatically

### Daily Learning Flow

1. App runs in background (menu bar tray icon)
2. When idle threshold reached, flashcard pops up
3. Choose correct answer or skip
4. Pet celebrates your progress
5. Repeat throughout the day

### Wordbook Import

**JSON Format**
```json
[
  {"word": "abandon", "meaning_zh": "放弃；抛弃", "phonetic": "/əˈbændən/"},
  {"word": "ability", "meaning_zh": "能力", "phonetic": "/əˈbɪləti/"}
]
```

**CSV Format**
```csv
word,meaning_zh,phonetic
abandon,放弃,/əˈbændən/
ability,能力,/əˈbɪləti/
```

**TXT Format** (tab or space separated)
```
abandon  放弃  /əˈbændən/
ability  能力  /əˈbɪləti/
```

Field name aliases supported:
- Word: `word`, `单词`, `词`, `英文`
- Meaning: `meaning_zh`, `meaning`, `释义`, `中文`, `翻译`
- Phonetic: `phonetic`, `音标`

### Reminder Modes

| Mode | Idle Trigger | Fallback Interval | Best For |
|------|-------------|-------------------|----------|
| Gentle | 180s | 45min | Light learners |
| Balanced | 120s | 30min | Regular learners |
| Intensive | 90s | 20min | Heavy learners |
| Custom | User-defined | User-defined | Power users |

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl + Shift + 1-4` | Select answer option |
| `Cmd/Ctrl + Shift + Esc` | Skip current card |

---

## Configuration

Config stored in SQLite database at:
```
~/Library/Application Support/com.chenshaojie.fragment-vocab/fragment-vocab.db
```

### Key Settings

| Setting | Description | Default |
|---------|-------------|---------|
| `daily_new_limit` | New words per day | 10 |
| `review_first` | Prioritize due cards | true |
| `allow_new_when_no_due` | Show new words when no reviews | true |
| `show_phonetic` | Display IPA pronunciation | true |
| `animations_enabled` | UI animations | true |
| `auto_pronounce` | Speak word on card show | false |

---

## Development

### Running Tests

```bash
# All tests (Rust + TypeScript)
pnpm test

# Watch mode
pnpm run test:watch

# Rust tests only
cd src-tauri && cargo test

# Frontend tests only
cd .. && npx vitest run
```

### Adding a New Feature

1. Database: Add migration in `src-tauri/migrations/`
2. Backend: Add command in `src-tauri/src/commands/`
3. Frontend: Add UI in appropriate window (`src/main/`, `src/card.ts`, etc.)
4. Tests: Add unit tests for domain logic

---

## FAQ

**Q: Does it work on Windows/Linux?**
A: Currently macOS only due to idle detection implementation. Linux support planned.

**Q: How is my data stored?**
A: Local SQLite database. No cloud sync.

**Q: Can I reset my progress?**
A: Yes, delete the database file to start fresh.

**Q: How does the pet affect learning?**
A: The pet is cosmetic but provides motivation. Health decreases if you don't study regularly.

**Q: What's the difference between "know" and "don't know"?**
A: "Know" schedules the card for later review. "Don't know" adds to wrong book and reschedules soon.

---

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) file.

---

## 中文介绍

Fragment Vocab 是一款智能背单词桌面应用，特点：

- **碎片时间学习**：检测系统空闲时间自动弹出单词卡片
- **间隔重复算法**：基于 FSRS 算法优化复习节奏
- **词库管理**：内置 IELTS 核心 3000 词，支持自定义导入
- **宠物陪伴**：可爱的像素史莱姆宠物陪你学习
- **静默运行**：后台运行，菜单栏托盘图标
- **macOS 原生**：支持全局快捷键、自动启动、深色模式

### 快速开始

```bash
pnpm install
pnpm tauri dev
```
