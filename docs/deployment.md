# 部署与打包指南

## 版本管理策略

### 版本号规范

采用语义化版本 (Semantic Versioning)：`MAJOR.MINOR.PATCH`

- **MAJOR**: 不兼容的 API 变更
- **MINOR**: 向后兼容的功能新增
- **PATCH**: 向后兼容的问题修复

示例：
- `0.1.0` - MVP 首版
- `0.2.0` - 新增词库编辑功能
- `0.2.1` - 修复空闲检测 bug

### 版本号更新位置

发布前需同步更新以下文件：

1. `package.json` - `version` 字段
2. `src-tauri/Cargo.toml` - `version` 字段
3. `CHANGELOG.md` - 添加新版本记录

## 本地构建

### 开发构建

```bash
# 调试模式构建（快速，包含调试信息）
npm run tauri build -- --debug
```

### 生产构建

```bash
# 完整构建（优化，体积小）
npm run tauri build

# 指定目标架构
npm run tauri build -- --target aarch64-apple-darwin  # Apple Silicon
npm run tauri build -- --target x86_64-apple-darwin   # Intel
npm run tauri build -- --target universal-apple-darwin # 通用二进制
```

### 构建产物位置

```
src-tauri/target/
├── release/
│   └── bundle/
│       ├── macos/
│       │   └── Fragment Vocab.app      # macOS 应用包
│       └── dmg/
│           └── Fragment Vocab_*.dmg    # DMG 安装包
└── debug/
    └── bundle/
        └── macos/
            └── Fragment Vocab.app
```

## DMG 打包配置

### Tauri 配置

在 `src-tauri/tauri.conf.json` 中配置：

```json
{
  "bundle": {
    "identifier": "com.shaojie.fragment-vocab",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "macOS": {
      "minimumSystemVersion": "12.0",
      "entitlements": null,
      "exceptionDomain": "",
      "frameworks": [],
      "providerShortName": null,
      "signingIdentity": null
    },
    "dmg": {
      "background": "assets/dmg-background.png",
      "windowSize": {
        "width": 600,
        "height": 400
      },
      "appPosition": {
        "x": 180,
        "y": 170
      },
      "applicationFolderPosition": {
        "x": 420,
        "y": 170
      }
    }
  }
}
```

### 自定义 DMG 背景

1. 准备 600x400 的背景图：`assets/dmg-background.png`
2. 在 Tauri 配置中引用
3. 重新构建即可应用

## GitHub Actions 自动化

### CI 流程

触发条件：
- Push 到 `main` 或 `develop` 分支
- Pull Request 到 `main` 或 `develop` 分支

执行步骤：
1. 代码检查（lint）
2. 运行测试
3. 构建应用（debug 模式）
4. 上传构建产物（仅 main 分支）

### Release 流程

触发条件：
- 推送 `v*` 格式的 tag（如 `v0.1.0`）

执行步骤：
1. 创建 GitHub Release
2. 构建 Universal Binary（支持 Intel + Apple Silicon）
3. 生成 DMG 安装包
4. 计算 SHA256 校验和
5. 上传所有产物到 Release

### 发布新版本

```bash
# 1. 更新版本号
# 编辑 package.json 和 src-tauri/Cargo.toml

# 2. 更新 CHANGELOG.md
# 添加新版本的变更记录

# 3. 提交变更
git add .
git commit -m "chore: bump version to 0.1.0"

# 4. 创建并推送 tag
git tag v0.1.0
git push origin main
git push origin v0.1.0

# 5. GitHub Actions 自动构建并发布
# 访问 https://github.com/Shaojie66/fragment-vocab/releases 查看
```

## 签名与公证（可选）

### 代码签名

需要 Apple Developer 账号：

```bash
# 设置签名身份
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"

# 构建时自动签名
npm run tauri build
```

### 公证（Notarization）

```bash
# 需要 App-specific password
xcrun notarytool submit \
  "src-tauri/target/release/bundle/dmg/Fragment Vocab_0.1.0_universal.dmg" \
  --apple-id "your@email.com" \
  --password "app-specific-password" \
  --team-id "TEAM_ID" \
  --wait

# 装订公证票据
xcrun stapler staple \
  "src-tauri/target/release/bundle/dmg/Fragment Vocab_0.1.0_universal.dmg"
```

## 测试安装包

### 本地测试

```bash
# 打开 DMG
open src-tauri/target/release/bundle/dmg/*.dmg

# 或直接运行 .app
open src-tauri/target/release/bundle/macos/Fragment\ Vocab.app
```

### 首次运行

macOS 可能提示"无法验证开发者"：

1. 右键点击应用 → 选择"打开"
2. 或在"系统设置 → 隐私与安全性"中允许

## 分发渠道

### GitHub Releases（推荐）

- 自动化发布
- 版本管理清晰
- 支持下载统计

### 手动分发

```bash
# 生成校验和
shasum -a 256 Fragment\ Vocab_0.1.0_universal.dmg > checksums.txt

# 分发文件
- Fragment Vocab_0.1.0_universal.dmg
- checksums.txt
```

## 更新机制（未来）

Tauri 支持内置更新器：

```json
{
  "updater": {
    "active": true,
    "endpoints": [
      "https://github.com/Shaojie66/fragment-vocab/releases/latest/download/latest.json"
    ],
    "dialog": true,
    "pubkey": "YOUR_PUBLIC_KEY"
  }
}
```

需要配置：
1. 生成密钥对
2. 配置更新服务器
3. 在 Release 中包含 `latest.json`

## 故障排查

### 构建失败

```bash
# 清理缓存
rm -rf node_modules src-tauri/target
npm install

# 检查 Rust 工具链
rustc --version
cargo --version

# 检查 Tauri CLI
npm run tauri --version
```

### DMG 无法打开

- 检查 macOS 版本是否 >= 12.0
- 检查是否被 Gatekeeper 阻止
- 尝试右键 → 打开

### 应用闪退

- 查看控制台日志：`Console.app`
- 检查数据库文件权限
- 检查系统权限（辅助功能、自动化）

## 性能优化

### 减小包体积

```bash
# 启用 LTO（链接时优化）
# 在 src-tauri/Cargo.toml 中添加：
[profile.release]
lto = true
codegen-units = 1
opt-level = "z"  # 优化体积
strip = true     # 移除调试符号
```

### 加快构建速度

```bash
# 使用 sccache 缓存编译结果
cargo install sccache
export RUSTC_WRAPPER=sccache

# 并行编译
export CARGO_BUILD_JOBS=8
```

## 监控与反馈

### 崩溃报告

建议集成：
- Sentry
- Crashlytics
- 自建日志收集

### 使用统计

建议收集（需用户同意）：
- 应用启动次数
- 功能使用频率
- 崩溃率
- 系统版本分布

## 参考资源

- [Tauri 官方文档](https://tauri.app/v1/guides/)
- [macOS 代码签名指南](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [语义化版本规范](https://semver.org/)
