# 监控与日志方案

## 日志架构

### 日志层级

```
ERROR   - 错误，需要立即关注
WARN    - 警告，可能影响功能
INFO    - 关键业务事件
DEBUG   - 调试信息（仅开发环境）
TRACE   - 详细追踪（仅开发环境）
```

### 日志分类

1. **应用日志** - 应用生命周期、启动、退出
2. **业务日志** - 单词学习、复习、统计
3. **系统日志** - 空闲检测、窗口管理、快捷键
4. **错误日志** - 异常、崩溃、数据库错误
5. **性能日志** - 响应时间、资源占用

## 日志实现

### Rust 侧（Tauri）

使用 `tracing` 生态：

```toml
# src-tauri/Cargo.toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"
```

```rust
// src-tauri/src/main.rs
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

fn setup_logging() {
    let log_dir = app_data_dir().join("logs");
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        log_dir,
        "fragment-vocab.log"
    );

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(file_appender))
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(EnvFilter::from_default_env()
            .add_directive("fragment_vocab=info".parse().unwrap())
            .add_directive("tauri=warn".parse().unwrap()))
        .init();

    tracing::info!("Application started");
}
```

### TypeScript 侧

使用自定义 Logger：

```typescript
// src/shared/logger.ts
enum LogLevel {
  ERROR = 0,
  WARN = 1,
  INFO = 2,
  DEBUG = 3,
  TRACE = 4,
}

class Logger {
  private level: LogLevel = LogLevel.INFO;

  constructor(private context: string) {}

  error(message: string, meta?: any) {
    this.log(LogLevel.ERROR, message, meta);
  }

  warn(message: string, meta?: any) {
    this.log(LogLevel.WARN, message, meta);
  }

  info(message: string, meta?: any) {
    this.log(LogLevel.INFO, message, meta);
  }

  debug(message: string, meta?: any) {
    this.log(LogLevel.DEBUG, message, meta);
  }

  private log(level: LogLevel, message: string, meta?: any) {
    if (level > this.level) return;

    const timestamp = new Date().toISOString();
    const levelStr = LogLevel[level];
    const logEntry = {
      timestamp,
      level: levelStr,
      context: this.context,
      message,
      meta,
    };

    // 输出到控制台
    console.log(`[${timestamp}] ${levelStr} [${this.context}] ${message}`, meta || '');

    // 发送到 Rust 侧持久化
    if (level <= LogLevel.WARN) {
      window.__TAURI__?.invoke('log_event', { entry: logEntry });
    }
  }
}

export const createLogger = (context: string) => new Logger(context);
```

### 使用示例

```typescript
// src/features/word-card/WordCard.vue
import { createLogger } from '@/shared/logger';

const logger = createLogger('WordCard');

function handleAnswer(result: 'know' | 'dont_know' | 'skip') {
  logger.info('User answered', { result, wordId: currentWord.id });
  
  try {
    await submitAnswer(result);
  } catch (error) {
    logger.error('Failed to submit answer', { error, wordId: currentWord.id });
  }
}
```

## 日志存储

### 文件位置

```
macOS:
~/Library/Application Support/com.shaojie.fragment-vocab/logs/
├── fragment-vocab.log           # 当天日志
├── fragment-vocab.log.2025-03-11
├── fragment-vocab.log.2025-03-10
└── ...
```

### 轮转策略

- **按天轮转** - 每天生成新文件
- **保留 30 天** - 自动删除 30 天前的日志
- **单文件限制** - 最大 50MB，超过则强制轮转

### 实现

```rust
// src-tauri/src/logging.rs
use std::fs;
use std::time::{SystemTime, Duration};

pub fn cleanup_old_logs(log_dir: &Path, retention_days: u64) {
    let cutoff = SystemTime::now() - Duration::from_secs(retention_days * 86400);
    
    if let Ok(entries) = fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if modified < cutoff {
                        let _ = fs::remove_file(entry.path());
                        tracing::info!("Removed old log: {:?}", entry.path());
                    }
                }
            }
        }
    }
}
```

## 错误追踪

### 错误上下文

```typescript
// src/shared/error-handler.ts
import { createLogger } from './logger';

const logger = createLogger('ErrorHandler');

export class AppError extends Error {
  constructor(
    message: string,
    public code: string,
    public context?: Record<string, any>
  ) {
    super(message);
    this.name = 'AppError';
  }
}

export function handleError(error: unknown, context?: Record<string, any>) {
  if (error instanceof AppError) {
    logger.error(error.message, {
      code: error.code,
      context: { ...error.context, ...context },
      stack: error.stack,
    });
  } else if (error instanceof Error) {
    logger.error(error.message, {
      context,
      stack: error.stack,
    });
  } else {
    logger.error('Unknown error', { error, context });
  }
}
```

### 全局错误捕获

```typescript
// src/app/main.ts
window.addEventListener('error', (event) => {
  handleError(event.error, {
    type: 'uncaught',
    filename: event.filename,
    lineno: event.lineno,
    colno: event.colno,
  });
});

window.addEventListener('unhandledrejection', (event) => {
  handleError(event.reason, {
    type: 'unhandled-promise',
  });
});
```

## 性能监控

### 关键指标

1. **启动时间** - 从启动到菜单栏显示
2. **空闲检测延迟** - 检测周期与准确性
3. **卡片弹出延迟** - 触发到显示的时间
4. **数据库查询时间** - 各类查询的耗时
5. **内存占用** - 常驻内存使用量

### 性能日志

```typescript
// src/shared/performance.ts
export class PerformanceMonitor {
  private marks = new Map<string, number>();

  start(label: string) {
    this.marks.set(label, performance.now());
  }

  end(label: string) {
    const start = this.marks.get(label);
    if (!start) return;

    const duration = performance.now() - start;
    this.marks.delete(label);

    logger.info('Performance', { label, duration: `${duration.toFixed(2)}ms` });

    // 超过阈值时警告
    if (duration > 1000) {
      logger.warn('Slow operation detected', { label, duration });
    }
  }

  measure<T>(label: string, fn: () => T): T {
    this.start(label);
    try {
      return fn();
    } finally {
      this.end(label);
    }
  }

  async measureAsync<T>(label: string, fn: () => Promise<T>): Promise<T> {
    this.start(label);
    try {
      return await fn();
    } finally {
      this.end(label);
    }
  }
}

export const perf = new PerformanceMonitor();
```

### 使用示例

```typescript
// 同步操作
const result = perf.measure('select-word', () => {
  return selectNextWord();
});

// 异步操作
const result = await perf.measureAsync('submit-answer', async () => {
  return await submitAnswer(result);
});
```

## 统计数据收集

### 本地统计

```typescript
// src/data/analytics.ts
interface AnalyticsEvent {
  event: string;
  timestamp: string;
  properties?: Record<string, any>;
}

export class Analytics {
  private db: Database;

  async track(event: string, properties?: Record<string, any>) {
    const entry: AnalyticsEvent = {
      event,
      timestamp: new Date().toISOString(),
      properties,
    };

    await this.db.execute(
      'INSERT INTO analytics_events (event, timestamp, properties) VALUES (?, ?, ?)',
      [entry.event, entry.timestamp, JSON.stringify(entry.properties)]
    );
  }

  async getStats(startDate: string, endDate: string) {
    return await this.db.query(
      'SELECT event, COUNT(*) as count FROM analytics_events WHERE timestamp BETWEEN ? AND ? GROUP BY event',
      [startDate, endDate]
    );
  }
}
```

### 关键事件

```typescript
// 应用生命周期
analytics.track('app_started');
analytics.track('app_quit');

// 用户行为
analytics.track('word_shown', { wordId, stage });
analytics.track('word_answered', { wordId, result, responseTime });
analytics.track('word_skipped', { wordId });

// 功能使用
analytics.track('pause_activated', { duration: '1h' });
analytics.track('stats_viewed');
analytics.track('settings_changed', { key, value });
```

## 崩溃报告

### Rust 侧 Panic 捕获

```rust
// src-tauri/src/main.rs
use std::panic;

fn setup_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        let payload = panic_info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());

        tracing::error!("PANIC: {} at {}", message, location);

        // 写入崩溃报告
        let crash_report = format!(
            "Crash Report\nTime: {}\nMessage: {}\nLocation: {}\n",
            chrono::Utc::now(),
            message,
            location
        );

        let crash_file = app_data_dir().join("crash.log");
        std::fs::write(crash_file, crash_report).ok();
    }));
}
```

### 崩溃恢复

```rust
// 启动时检查崩溃报告
fn check_crash_report() {
    let crash_file = app_data_dir().join("crash.log");
    if crash_file.exists() {
        if let Ok(report) = std::fs::read_to_string(&crash_file) {
            tracing::warn!("Previous crash detected:\n{}", report);
            
            // 可选：上传到服务器或显示给用户
            
            // 清理崩溃报告
            std::fs::remove_file(crash_file).ok();
        }
    }
}
```

## 日志查看工具

### 内置日志查看器

```typescript
// src/features/debug/LogViewer.vue
<template>
  <div class="log-viewer">
    <div class="controls">
      <select v-model="levelFilter">
        <option value="all">All</option>
        <option value="error">Error</option>
        <option value="warn">Warn</option>
        <option value="info">Info</option>
      </select>
      <button @click="refresh">Refresh</button>
      <button @click="clear">Clear</button>
      <button @click="exportLogs">Export</button>
    </div>
    <div class="logs">
      <div v-for="log in filteredLogs" :key="log.id" :class="['log-entry', log.level]">
        <span class="timestamp">{{ log.timestamp }}</span>
        <span class="level">{{ log.level }}</span>
        <span class="context">{{ log.context }}</span>
        <span class="message">{{ log.message }}</span>
      </div>
    </div>
  </div>
</template>
```

### CLI 工具

```bash
# 查看最新日志
tail -f ~/Library/Application\ Support/com.shaojie.fragment-vocab/logs/fragment-vocab.log

# 搜索错误
grep ERROR ~/Library/Application\ Support/com.shaojie.fragment-vocab/logs/*.log

# 统计错误数量
grep -c ERROR ~/Library/Application\ Support/com.shaojie.fragment-vocab/logs/fragment-vocab.log
```

## 隐私与合规

### 数据脱敏

```typescript
function sanitize(data: any): any {
  if (typeof data === 'string') {
    // 移除邮箱
    data = data.replace(/[\w.-]+@[\w.-]+\.\w+/g, '[EMAIL]');
    // 移除路径
    data = data.replace(/\/Users\/[^\/]+/g, '/Users/[USER]');
  }
  return data;
}
```

### 用户控制

```typescript
// 设置页面
interface LoggingSettings {
  enabled: boolean;
  level: 'error' | 'warn' | 'info' | 'debug';
  includePerformance: boolean;
  includeAnalytics: boolean;
}
```

## 建议的监控方案

### MVP 阶段（本地）
- 文件日志（按天轮转）
- 本地崩溃报告
- 基础性能监控
- 本地统计数据

### 成熟阶段（可选）
- Sentry 错误追踪
- 自建日志服务器
- 实时性能监控
- 用户行为分析

## 参考配置

```toml
# src-tauri/Cargo.toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-appender = "0.2"
sentry = { version = "0.31", optional = true }

[features]
sentry-integration = ["sentry"]
```

```json
// package.json
{
  "devDependencies": {
    "@sentry/browser": "^7.0.0",
    "@sentry/tracing": "^7.0.0"
  }
}
```
