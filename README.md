# Fragment Vocab

被动提醒式背词工具。当前产品形态已经不是“纯托盘小工具”，而是：

- 主页面：设置控制台，负责配置、推荐说明、诊断、反馈、模板和导入导出
- 托盘：辅助入口，保留暂停、今日不再提醒、打开主页面/统计页、退出
- 浮卡：真正的作答组件，负责认识 / 不认识 / 跳过和轻反馈

## 当前能力

- 首次使用引导
- 系统推荐提醒频率，且允许用户手动覆盖
- 工作日 / 周末不同推荐策略
- 静默时间、自定义学习计划、卡片偏好、启动行为配置
- 主页面反馈入口和浮卡轻反馈
- 团队模板
- 配置摘要导出、JSON 导出与导入
- 统计页查看今日状态、推荐和最近反馈

当前仍未支持：

- 自定义词库导入
- 多设备同步
- 自定义浮卡位置

## 本地数据

应用数据保存在本地，不上传云端。

- 数据库位置：`~/Library/Application Support/com.chenshaojie.fragment-vocab/fragment-vocab.db`

首次启动会导入内置词库：

- `assets/wordbooks/ielts-core-3000.json`

## 开发

```bash
npm install
npm run dev
```

常用命令：

```bash
npm run build
npm test
cd src-tauri && cargo test
npm run tauri build
npm run package:mac
```

## 当前交互原则

- 默认策略偏克制，优先减少打扰
- 系统会给推荐值，但不会强制覆盖用户自定义
- 无卡可学、暂停、静默等状态要在主页面解释清楚
- 托盘不是唯一入口

## 文档

- [被动提醒 PRD](docs/prd-passive-reminder-v0.2.md)
- [设置控制台 IA](docs/settings-console-ia.md)
- [Issue Backlog](docs/issue-backlog-v0.2.md)
