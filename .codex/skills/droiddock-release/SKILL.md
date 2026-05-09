---
name: droiddock-release
description: 当需要准备、验证、打包、打 tag、推送或发布 DroidDock 版本与发布产物时使用。
---

# DroidDock 发布

## 必读上下文

先读取这些内容：

- `AGENTS.md`
- `package.json`
- `src-tauri/tauri.conf.json`
- `docs/product-requirements.md`
- `git status --short`

## 工作流

1. 确认范围：只改版本号、只构建、提交/推送、打 tag、发布产物，还是完整发布。
2. 检查 `package.json` 和 Tauri 配置中的版本号是否一致。
3. 检查当前未提交改动；除非用户明确要求，不要把无关用户改动放进发布提交。
4. 打包前运行有针对性的验证：
   - 前端和领域逻辑检查：`npm run test`
   - 后端有改动时检查 Rust：`cd src-tauri && cargo test`
5. 按用户要求构建发布产物：
   - `.app` 包：`npm run tauri:build:app`
   - DMG：`npm run tauri:build`
6. 如果 DMG 打包失败，先区分代码错误和本机 macOS `hdiutil` 环境错误，再决定是否修改代码。
7. 如果用户要求发布，按明确范围提交、创建 tag、推送分支和 tag，并验证远端状态。

## 发布边界

- 不要静默包含无关的未跟踪文档或生成文件。
- 不要在发布清理过程中改变产品范围。
- 只有命令实际生成 DMG 且路径已验证时，才能声称 DMG 已生成。
- 如果只能产出 `.app` 作为兜底，不要把它说成 DMG 发布。

## 常用检查

```bash
npm run test
npm run build
npm run tauri:build:app
git status --short
git tag --list 'v*'
```
