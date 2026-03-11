# 贡献指南

感谢你对 Fragment Vocab 的关注！我们欢迎各种形式的贡献。

## 如何贡献

### 报告 Bug

如果你发现了 bug，请在 [GitHub Issues](https://github.com/Shaojie66/fragment-vocab/issues) 中提交，并包含以下信息：

- 问题描述
- 复现步骤
- 预期行为
- 实际行为
- 系统环境（macOS 版本、应用版本）
- 相关日志或截图

### 提出新功能

在提交功能请求前，请先查看 [Issues](https://github.com/Shaojie66/fragment-vocab/issues) 和 [Discussions](https://github.com/Shaojie66/fragment-vocab/discussions) 确认是否已有类似讨论。

提交时请说明：
- 功能描述
- 使用场景
- 为什么需要这个功能
- 可能的实现方案（可选）

### 提交代码

1. **Fork 仓库**
   ```bash
   git clone https://github.com/YOUR_USERNAME/fragment-vocab.git
   cd fragment-vocab
   ```

2. **创建分支**
   ```bash
   git checkout -b feature/your-feature-name
   # 或
   git checkout -b fix/your-bug-fix
   ```

3. **开发与测试**
   ```bash
   npm install
   npm run tauri dev
   npm test
   ```

4. **提交代码**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   # 或
   git commit -m "fix: fix your bug description"
   ```

5. **推送并创建 PR**
   ```bash
   git push origin feature/your-feature-name
   ```

   然后在 GitHub 上创建 Pull Request。

## 代码规范

### Commit 消息格式

使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Type 类型：**
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式调整（不影响功能）
- `refactor`: 重构（不是新功能也不是 bug 修复）
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建工具或辅助工具的变动

**示例：**
```
feat(srs): add stage 5 for long-term review

Add a new review stage at 30 days interval for better retention.

Closes #123
```

### 代码风格

- TypeScript: 使用 ESLint + Prettier
- Rust: 使用 rustfmt
- 提交前运行 `npm run lint` 和 `npm run format`

### 测试要求

- 新功能必须包含单元测试
- Bug 修复应包含回归测试
- 确保所有测试通过：`npm test`

## 开发流程

### 本地开发

```bash
# 安装依赖
npm install

# 启动开发服务器（热重载）
npm run tauri dev

# 运行测试
npm test

# 运行测试（watch 模式）
npm run test:watch

# 代码检查
npm run lint

# 代码格式化
npm run format
```

### 调试

- 前端调试：在 Tauri 窗口中按 `Cmd+Option+I` 打开开发者工具
- Rust 调试：使用 `println!` 或配置 VS Code 调试器

### 构建测试

```bash
# 构建应用
npm run tauri build

# 测试构建产物
open src-tauri/target/release/bundle/macos/Fragment\ Vocab.app
```

## 项目结构

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
├── scripts/               # 构建脚本
└── tests/                 # 测试文件
```

## 发布流程

发布由维护者负责，流程如下：

1. 更新版本号（`package.json` 和 `src-tauri/Cargo.toml`）
2. 更新 CHANGELOG.md
3. 创建 Git tag：`git tag v0.1.0`
4. 推送 tag：`git push origin v0.1.0`
5. GitHub Actions 自动构建并发布 Release

## 行为准则

- 尊重所有贡献者
- 保持友好和建设性的讨论
- 接受建设性的批评
- 关注对项目最有利的事情

## 需要帮助？

- 查看 [文档](./docs/)
- 在 [Discussions](https://github.com/Shaojie66/fragment-vocab/discussions) 提问
- 查看现有 [Issues](https://github.com/Shaojie66/fragment-vocab/issues)

再次感谢你的贡献！🎉
