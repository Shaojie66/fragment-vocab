# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

```bash
# Development
npm run tauri dev          # Start dev server with hot reload
npm test                   # Run all tests once
npm run test:watch         # Run tests in watch mode

# Build
npm run tauri build        # Build production app
npm run build:mac:app      # Build and sign macOS app
```

## Architecture Overview

Fragment Vocab is a Tauri 2.0 desktop app for learning vocabulary during idle time. The app intelligently schedules word cards based on system idle detection and spaced repetition algorithms.

### Multi-Window Architecture

The app uses three independent Tauri windows, each with its own HTML/TS entry point:

- **main** (`index.html` / `main.ts`) - Main dashboard, settings, wordbook management
- **card** (`card.html` / `card.ts`) - Pop-up quiz card window
- **stats** (`stats.html` / `stats.ts`) - Statistics and progress tracking

Window coordination happens through Tauri events (`card-hidden`, `card-window-shown`, etc.) and shared backend state.

### Frontend Domain Layer

**TriggerScheduler** (`src/domain/scheduler/triggerScheduler.ts`)
- Runs in main window, polls every 5 seconds
- Checks: system idle time, quiet hours, pause state, main window focus
- Uses `document.hasFocus()` to detect if main window is active (not just visible)
- Triggers card display when conditions are met

**SRS Engine** (`src/domain/srs/srsEngine.ts`)
- Implements spaced repetition algorithm
- Calculates next review intervals based on performance
- Manages card stages and difficulty adjustments

**Card Selector** (`src/domain/words/cardSelector.ts`)
- Prioritizes due cards over new cards (configurable)
- Respects daily new word limits
- Filters by enabled/disabled wordbook sources

### Backend Architecture (Rust)

**Commands Layer** (`src-tauri/src/commands.rs`)
- Tauri commands exposed to frontend via `#[tauri::command]`
- Handles all business logic: card selection, review submission, config management
- Key commands: `get_next_card`, `submit_review`, `get_today_stats`

**Database Layer** (`src-tauri/src/db/`)
- SQLite with repository pattern
- Repositories: `CardsRepository`, `WordsRepository`, `LogsRepository`, `StateRepository`
- `WordbookImporter` supports JSON/CSV/TXT/XLSX formats with flexible field aliases
- Migrations run automatically on startup

**Idle Detection** (`src-tauri/src/idle.rs`)
- Platform-specific: uses Core Graphics on macOS
- Returns seconds since last user input (mouse/keyboard)

## Key Behaviors

### Card Display Logic

When a card is shown:
1. Card window positions near mouse cursor with 1000px margin from screen edges
2. Main window hides automatically
3. Card window stays on top, unfocused (doesn't steal keyboard focus)

When a card is answered:
- Correct: Window hides immediately
- Wrong: Shows correct answer, auto-hides after 5 seconds

### Daily Goal Completion

When daily new word limit is reached and no cards are due:
- Shows congratulations screen with two options:
  - "调高目标继续挑战" - Increases daily limit by 10 and continues
  - "今天就到这里" - Closes card window

### Wordbook Import

Supports flexible field names for compatibility:
- Word field: `word`, `单词`, `词`, `英文`
- Meaning field: `meaning_zh`, `meaning`, `释义`, `中文`, `翻译`

## Testing

Tests use Vitest with jsdom environment. Domain logic (scheduler, SRS, card selector) has comprehensive unit tests.

## gstack

Use the `/browse` skill from gstack for all web browsing, never use `mcp__claude-in-chrome__*` tools.

### Available skills

- `/office-hours` - YC Office Hours: reframe your product before you write code
- `/plan-ceo-review` - CEO/Founder: rethink the problem, find the 10-star product
- `/plan-eng-review` - Eng Manager: lock in architecture, data flow, diagrams, edge cases
- `/plan-design-review` - Senior Designer: rate design dimensions, AI slop detection
- `/design-consultation` - Design Partner: build a complete design system from scratch
- `/review` - Staff Engineer: find production bugs, auto-fix obvious issues
- `/ship` - Release Engineer: sync main, run tests, push, open PR
- `/browse` - QA Engineer: real Chromium browser automation (~100ms/command)
- `/qa` - QA Lead: test app, find bugs, fix with atomic commits, generate regression tests
- `/qa-only` - QA Reporter: same methodology as /qa but report only
- `/design-review` - Designer Who Codes: audit design then fix what it finds
- `/setup-browser-cookies` - Session Manager: import cookies from real browser
- `/retro` - Eng Manager: team-aware weekly retro with per-person breakdowns
- `/investigate` - Debugger: systematic root-cause debugging
- `/document-release` - Technical Writer: update all docs to match shipped changes
- `/codex` - Second Opinion: independent review from OpenAI Codex CLI
- `/careful` - Safety Guardrails: warns before destructive commands
- `/freeze` - Edit Lock: restrict file edits to one directory
- `/guard` - Full Safety: `/careful` + `/freeze` combined
- `/unfreeze` - Unlock: remove the `/freeze` boundary
- `/gstack-upgrade` - Self-Updater: upgrade gstack to latest version

If gstack skills aren't working, run `cd .claude/skills && ./setup` to build the binary and register skills.

