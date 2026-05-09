# DroidDock Gemini CLI 入口

本文件是 Gemini CLI 的项目入口。公共项目规则以根目录 `AGENTS.md` 为准；本文件只保留 Gemini 专属补充和当前架构索引，避免多套智能体文档漂移。

## 必读顺序

1. 先读取并遵守 `AGENTS.md`。
2. 再按任务读取 `docs/ai/skills-catalog.md`。
3. 如果任务命中项目本地技能，读取 `.codex/skills/<skill-name>/SKILL.md` 作为同源流程说明。
4. 最后读取任务相关代码和 `docs/` 文档。

## Gemini 专属约束

- 所有回复默认使用中文。
- 不直接覆盖 Codex 或用户已有的未提交改动；先检查 `git status --short`。
- 不把 `.codex/skills` 复制成另一套长期维护的 Gemini 专属技能，除非用户明确要求；优先复用同一份项目规则。
- 如果 Gemini CLI 支持技能激活，按 `docs/ai/skills-catalog.md` 中的名称选择对应流程。
- 如果 Gemini CLI 不支持直接激活本地技能，则把对应 `SKILL.md` 当作普通项目流程文档读取。

## 常用入口

- 公共规则：`AGENTS.md`
- 技能清单：`docs/ai/skills-catalog.md`
- 产品需求：`docs/product-requirements.md`
- 架构说明：`docs/architecture.md`
- UI 预览：`docs/droiddock_ui_preview.html`
- 前端入口：`src/App.vue`
- 后端入口：`src-tauri/src/lib.rs`

## 当前架构索引

- 前端 IPC 封装：`src/lib/ipc/client.ts`
- 前端错误归一化：`src/lib/ipc/errors.ts`
- 后端 DTO 类型：`src/lib/ipc/types.ts`
- 前端领域类型：`src/types/app.ts`
- Pinia 状态：按 `config`、`tools`、`devices`、`sessions`、`ui` 拆分；`src/stores/app.ts` 保留为兼容门面。
- 后端命令入口：`src-tauri/src/lib.rs`
- 外部命令执行：`src-tauri/src/command.rs`
- 配置管理：`src-tauri/src/config.rs`
- 设备解析：`src-tauri/src/devices.rs`
- 无线连接与配对：`src-tauri/src/wireless.rs`
- scrcpy 会话与日志：`src-tauri/src/sessions.rs`
- 工具检测和安装：`src-tauri/src/tools.rs`
- 工具下载来源：`src-tauri/src/tool_manifest.rs`
