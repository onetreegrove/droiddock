# DroidDock 智能体约束规则

本文件是 DroidDock 仓库内所有 AI 智能体的公共约束入口。Codex、Gemini CLI 以及其他智能体进入本仓库后，都应先遵守本文，再读取各自入口文件和任务相关技能。

## 指令优先级

1. 用户当前明确指令。
2. 本文件 `AGENTS.md`。
3. 智能体专属入口文件，例如 `GEMINI.md`。
4. 项目本地技能，例如 `.codex/skills/*/SKILL.md`。
5. `docs/` 下的 PRD、SPEC、计划书、调研文档。
6. 当前代码实现。

当上述来源冲突时，按优先级处理，并在回复中说明冲突点和采用依据。

## 语言与沟通

- 默认使用中文回复。
- 先读取当前代码、文档和工作区状态，再给出结论或改动。
- 对不确定的信息明确标注，不把历史经验当作当前事实。
- 用户要求评审时，优先指出缺陷、风险、回归和缺失验证；未被要求时不要顺手改代码。

## 工作区纪律

- 允许存在用户未提交改动。不得回滚、覆盖或格式化无关文件。
- 修改前先用 `git status --short` 理解工作区状态。
- 搜索文件和文本优先使用 `rg` / `rg --files`。
- 手工编辑保持小范围、可解释、可验证。
- 不引入与任务无关的重构、依赖或目录迁移。

## 项目边界

- DroidDock 是面向 macOS Apple Silicon 的 Android 投屏与控制桌面应用。
- 技术栈为 Tauri v2、Vue 3、TypeScript、Pinia、Rust。
- 第一版接受 `scrcpy` 独立窗口投屏，DroidDock 作为连接、配置和会话控制台存在。
- 目标用户偏非技术用户，错误信息和流程文案应转化为可执行的下一步。
- 第一版不支持 Intel Mac、Windows、Linux、App Store 发布、内嵌投屏画面、完整 logcat 或文件管理器。

## 关键实现约束

- 前端源码位于 `src/`，主要状态入口是 `src/stores/app.ts`。
- 业务参数合并逻辑优先放在 `src/domain/`，并配套 Vitest。
- 后端源码位于 `src-tauri/`，Tauri command 主要在 `src-tauri/src/lib.rs` 暴露。
- 前端调用后端应通过 Tauri `invoke` 和 store 方法聚合，不在组件里散落命令细节。
- 后端错误需要转换成中文、用户可理解的错误信息。
- 应用管理 `adb` 和 `scrcpy` 路径，不假设用户系统 `PATH` 一定存在这些工具。
- 不修改用户 shell PATH，不要求 Homebrew，不要求 sudo。
- `scrcpy` 会话是独立子进程，日志需要由后端缓冲并由前端按需读取。

## 文档与设计来源

- 产品范围优先读取 `docs/product-requirements.md`。
- UI 对照优先读取 `docs/droiddock_ui_preview.html`、`docs/ui-design.md` 和相关计划文档。
- scrcpy 参数和命令行为优先读取 `docs/scrcpy-usage.md`。
- 发布、版本和打包任务要同时检查 `package.json`、`src-tauri/tauri.conf.json` 和当前 git 状态。

## 验证要求

- TypeScript / Vue 改动：优先运行 `npm run test`，必要时运行 `npm run build`。
- Rust / Tauri 后端改动：优先运行 `cd src-tauri && cargo test`。
- 打包或发布改动：优先运行 `npm run tauri:build:app`；只有任务需要 DMG 时才运行 `npm run tauri:build`。
- 如果 DMG 构建失败，先区分代码错误、签名配置错误和本机 `hdiutil` 环境错误，不直接判定为代码失败。
- 文档或 skill 改动至少验证文件存在、frontmatter 基本格式和关键链接路径。

## 技能规则

- 项目本地 Codex 技能存放在 `.codex/skills/<skill-name>/SKILL.md`。
- 技能名称使用小写字母、数字和连字符。
- 技能 frontmatter 必须包含 `name` 和 `description`。
- `description` 只描述触发场景，不总结工作流。
- 通用项目约束放在本文，不复制到每个技能；技能只写特定任务流程。
- 如果需要给 Gemini CLI 使用同一能力，优先让 `GEMINI.md` 指向同一份项目规则和技能清单，避免维护两套相互漂移的规则。
