# DroidDock AI 技能清单

本清单用于说明项目本地技能的职责边界。公共约束以根目录 `AGENTS.md` 为准，本文只做索引。

## 目录约定

```text
.codex/skills/
  droiddock-release/
    SKILL.md
  droiddock-preview-gap-review/
    SKILL.md
  droiddock-frontend-implementation/
    SKILL.md
```

## 技能清单

### droiddock-release

适用于版本号同步、打包、提交、打标签、推送、发布产物验证等发布工作。

重点读取：

- `AGENTS.md`
- `package.json`
- `src-tauri/tauri.conf.json`
- `docs/product-requirements.md`

### droiddock-preview-gap-review

适用于用户要求对照 PRD、UI 预览稿、当前前端实现检查功能或体验缺口。

重点读取：

- `AGENTS.md`
- `docs/product-requirements.md`
- `docs/droiddock_ui_preview.html`
- `docs/ui-design.md`
- `src/components/`
- `src/stores/app.ts`

### droiddock-frontend-implementation

适用于实现或调整 Vue 3 前端、Pinia 状态、scrcpy 参数界面、设备连接流程、会话视图和设置页。

重点读取：

- `AGENTS.md`
- `docs/product-requirements.md`
- `src/App.vue`
- `src/components/`
- `src/domain/`
- `src/stores/app.ts`
- `src/types/app.ts`

## 多智能体使用方式

- Codex：优先读取 `AGENTS.md`，再按任务触发 `.codex/skills/*/SKILL.md`。
- Gemini CLI：优先读取 `GEMINI.md`；`GEMINI.md` 会要求回到 `AGENTS.md`，并以本清单作为项目技能索引。
- 其他智能体：先读 `AGENTS.md`，再按任务读取本清单指向的技能。

## 维护规则

- 新增高频流程时，先判断它是否是项目专属流程；项目专属流程放 `.codex/skills`，通用规则放 `AGENTS.md`。
- 同一规则只保留一个权威来源。不要在 `GEMINI.md`、技能和 `AGENTS.md` 之间重复大段内容。
- 修改技能后，至少检查 `SKILL.md` frontmatter、触发描述和路径引用。
