# DroidDock Frontend Rewrite Design

日期：2026-04-30

## 1. 背景

当前 Vue 前端已经能调用 Tauri 命令读取工具状态、设备列表、会话列表，并能启动/停止 `scrcpy`。但现有界面仍是早期结构：左侧只区分“发现与配对 / 已连接设备 / 设置”，设备详情只支持预设选择，缺少 `docs/droiddock_ui_preview.html` 中的四页控制台、深色工具型视觉、会话管理页、工具配置页、全局默认参数页，以及 PRD 已补充的“全局默认参数 / 设备独立参数 / 会话临时参数”规则。

本次重写目标是以 `docs/droiddock_ui_preview.html` 为视觉和交互参考，以 `docs/product-requirements.md` 为功能边界，把前端改造成 DroidDock MVP 控制台。投屏画面仍由 `scrcpy` 独立窗口承载，DroidDock 只负责设备发现、连接、参数配置、会话控制和日志排障。

## 2. 目标

- 重写前端为四个主视图：设备、投屏会话、工具配置、参数设置。
- 保留现有 Vue 3 + TypeScript + Pinia + Tauri invoke 技术路线。
- 将 UI 预览稿的深色控制台视觉落到真实组件中，不继续使用当前浅色卡片风格。
- 支持设备列表、设备详情、ADB Pair、USB 转无线、会话列表、工具状态、日志和错误说明。
- 支持全局默认参数、设备独立参数和会话启动临时参数的前端模型与持久化。
- 启动投屏前按“全局默认参数 -> 设备独立参数 -> 会话启动参数”合并，生成传给 `start_scrcpy` 的 `ScrcpyOptions`。
- 保证非技术用户不需要理解底层命令；命令预览只作为透明度信息，不作为主操作路径。

## 3. 非目标

- 不嵌入 `scrcpy` 投屏画面。
- 不实现录屏、文件管理器、完整 logcat。
- 不做 App Store、签名、公证和自动更新。
- 不在本次重写中实现真实工具下载器；工具配置页可以展示自动安装入口，但若后端命令缺失，应显示“待接入”或禁用态。
- 不修改用户 shell PATH，不要求 Homebrew，不要求 sudo。
- 不引入路由系统；MVP 使用 Pinia 中的 `currentPage` 管理四个页面即可。

## 4. 推荐方案

采用“分层组件 + 轻量状态模型”方案。

方案对比：

1. 推荐方案：按页面和领域拆组件，Pinia 统一承接 Tauri 数据、参数合并、表单状态。
   - 优点：适合当前单窗口工具应用，改动范围可控，后续接真实命令也清晰。
   - 缺点：Pinia store 会比现在更厚，需要把类型和参数工具函数拆出去。

2. 完整路由方案：引入 Vue Router，把四个主视图做成路由。
   - 优点：页面模型清楚。
   - 缺点：当前 Tauri 单窗口应用不需要 URL 历史，增加依赖和维护面。

3. 单文件快速复刻：把 UI 预览稿直接搬进 `App.vue`。
   - 优点：短期速度快。
   - 缺点：状态、表单、Tauri 调用和样式会混在一起，后续维护成本高。

结论：使用方案 1。页面拆分保持直接，避免过度封装；参数合并和预设逻辑必须放到纯 TypeScript 模块，方便测试和后续复用。

## 5. 信息架构

主导航固定四页：

- 设备：设备列表、连接方式入口、选中设备详情、设备级参数、命令预览、启动投屏。
- 投屏会话：运行中和已停止会话、参数摘要、停止/重连/日志入口、停止全部。
- 工具配置：adb/scrcpy 状态、版本、路径、自动安装入口、手动选择入口、新手入门步骤。
- 参数设置：全局默认参数、预设选择、画面参数、控制参数、常见错误说明。

导航状态使用枚举：

```ts
export type PageKey = 'devices' | 'sessions' | 'setup' | 'settings';
```

设备页内部保留 `selectedSerial`，初始值选择第一个设备；无设备时展示空状态和连接指引。

## 6. 前端状态模型

现有 `src/stores/app.ts` 需要拆出类型和参数工具函数。建议新增：

- `src/types/app.ts`：Tauri 返回类型、页面类型、参数类型、配置类型。
- `src/domain/scrcpyOptions.ts`：默认参数、预设、合并、命令预览、摘要标签。
- `src/stores/app.ts`：保留 Pinia store，负责 Tauri invoke、轮询、页面状态和调用 domain 函数。

核心类型：

```ts
export type ScrcpyOptions = {
  maxSize?: number;
  maxFps?: number;
  videoBitRate?: string;
  videoCodec?: 'default' | 'h264' | 'h265';
  noAudio?: boolean;
  noControl?: boolean;
  stayAwake?: boolean;
  turnScreenOff?: boolean;
  showTouches?: boolean;
  alwaysOnTop?: boolean;
  fullscreen?: boolean;
};

export type PresetId = 'daily' | 'lowBandwidth' | 'demo' | 'batterySaver' | 'viewOnly';

export type DeviceOptionEntry = {
  presetId: PresetId | null;
  options: ScrcpyOptions;
  updatedAt: number;
};

export type AppConfig = {
  adb_path: string | null;
  scrcpy_path: string | null;
  device_aliases: Record<string, string>;
  recent_endpoints: string[];
  default_scrcpy_options: ScrcpyOptions;
  default_preset_id: PresetId;
  device_scrcpy_options: Record<string, DeviceOptionEntry>;
};
```

Store 需要新增状态：

- `currentPage: PageKey`
- `selectedSerial: string | null`
- `appConfig: AppConfig | null`
- `deviceDraftOptions: Record<string, ScrcpyOptions>`
- `sessionDraftOptions: Record<string, ScrcpyOptions>`
- `globalDraftOptions: ScrcpyOptions`
- `globalDraftPresetId: PresetId`
- `modal: null | 'pair' | 'wireless' | 'logs'`
- `busy: Record<string, boolean>`

## 7. 参数规则

预设保持 PRD 中五个：

- `daily`：`--max-size=1920 --max-fps=60 --no-audio --stay-awake`
- `lowBandwidth`：`--max-size=1024 --video-bit-rate=2M --max-fps=30 --no-audio`
- `demo`：`--max-size=1920 --max-fps=60 --show-touches --always-on-top`
- `batterySaver`：`--max-size=1920 --max-fps=60 --no-audio --stay-awake --turn-screen-off`
- `viewOnly`：`--max-size=1920 --max-fps=60 --no-control`

参数合并规则：

```ts
effectiveOptions = {
  ...default_scrcpy_options,
  ...device_scrcpy_options[serial]?.options,
  ...sessionDraftOptions[serial],
}
```

布尔值必须支持显式 `false`，不能用 truthy 判断丢失用户关闭开关的意图。清除设备独立设置时删除 `device_scrcpy_options[serial]`，而不是保存空对象。

命令预览由前端生成，仅用于展示：

```text
scrcpy -s <serial> --max-size=1920 --max-fps=60 --no-audio --stay-awake
```

真正启动仍调用：

```ts
invoke<SessionInfo>('start_scrcpy', { serial, options: effectiveOptions })
```

## 8. Tauri 命令边界

前端重写需要最小后端配置扩展，否则无法满足“App 重启后保留全局默认参数和设备独立参数”。

保留现有命令：

- `get_tool_status`
- `list_devices`
- `list_sessions`
- `get_session_logs`
- `start_scrcpy`
- `stop_scrcpy`
- `stop_all_sessions`
- `adb_tcpip`
- `adb_connect`
- `adb_pair`
- `get_app_config`
- `set_tool_paths`
- `save_device_alias`

新增或扩展命令：

- 扩展 `AppConfig` 字段：`default_scrcpy_options`、`default_preset_id`、`device_scrcpy_options`。
- 新增 `save_default_scrcpy_options(options, presetId)`：保存全局默认参数。
- 新增 `save_device_scrcpy_options(serial, options, presetId)`：保存单台设备独立参数。
- 新增 `clear_device_scrcpy_options(serial)`：删除单台设备独立参数，恢复全局默认。

工具自动安装和手动选择文件对话框可在 UI 中保留入口；如果没有后端命令，本轮实现为禁用态或展示明确提示，不伪造成功。

## 9. 组件设计

建议文件结构：

```text
src/
  App.vue
  styles.css
  types/app.ts
  domain/scrcpyOptions.ts
  stores/app.ts
  components/
    AppSidebar.vue
    AppHeader.vue
    StatusChip.vue
    DeviceList.vue
    DeviceDetailPanel.vue
    ParameterEditor.vue
    CommandPreview.vue
    SessionsView.vue
    SessionCard.vue
    SetupView.vue
    SettingsView.vue
    PairModal.vue
    WirelessModal.vue
    LogPanel.vue
```

组件职责：

- `App.vue`：布局、页面切换、轮询生命周期。
- `AppSidebar.vue`：品牌、工具状态摘要、四页导航、版本信息。
- `DeviceList.vue`：设备卡片、状态提示、连接方式入口。
- `DeviceDetailPanel.vue`：选中设备信息、参数来源、启动投屏。
- `ParameterEditor.vue`：画面参数和控制参数编辑，支持全局和设备两种上下文。
- `CommandPreview.vue`：展示 `scrcpy` 命令预览。
- `SessionsView.vue` / `SessionCard.vue`：会话状态、参数摘要、停止/重连。
- `SetupView.vue`：工具配置和新手入门。
- `SettingsView.vue`：全局默认参数和错误说明。
- `PairModal.vue`：ADB Pair 表单，明确区分配对端口和连接端口。
- `WirelessModal.vue`：USB 转无线表单。
- `LogPanel.vue`：系统日志和会话日志。

## 10. 交互细节

设备页：

- 顶部显示发现设备数量和可用设备数量。
- `device` 状态允许启动投屏。
- `unauthorized` 禁用启动按钮，并显示“请在手机上允许 USB 调试授权”。
- `offline` 禁用启动按钮，并显示重新插拔或重连无线调试提示。
- 设备参数来源显示“使用全局默认”或“使用设备独立设置”。
- 修改设备参数后，用户可以选择“保存为此设备设置”或仅作为本次会话临时参数启动。
- “恢复全局默认”删除该设备独立配置，并刷新命令预览。

会话页：

- 运行中会话显示绿色状态条和运行时间。
- 停止/失败会话保留最近日志和参数摘要。
- 重连按钮使用该会话 `args` 反推或复用当前设备有效参数；实现阶段优先复用当前设备有效参数，避免从命令行字符串反解析。
- 停止全部调用 `stop_all_sessions`。

工具配置页：

- 展示 `adb` 和 `scrcpy` 路径、版本、架构/状态。
- 自动安装入口如果后端未接入，禁用并提示“自动安装待接入”。
- 手动选择入口如果后端未接入文件对话框，禁用并提示“手动选择待接入”。
- 新手入门步骤固定展示，不依赖后端。

参数设置页：

- 保存全局默认参数后调用 `save_default_scrcpy_options`。
- 应用预设后只是更新草稿，用户点击保存后才持久化。
- 常见错误说明来自 PRD 固定列表。

## 11. 错误处理

- Tauri invoke 抛错统一写入 `logs`，同时在当前页面显示用户可执行提示。
- 参数保存失败时保留用户草稿，不回滚输入。
- 启动投屏失败时留在设备页，显示错误提示，不跳转到会话页。
- ADB Pair 的配对码只保存在 modal 局部状态；关闭或成功后立即清空。
- 轮询失败不清空上一次成功数据，只记录错误，避免界面闪空。

## 12. 测试与验证

必须验证：

- `npm run build` 通过。
- 参数合并函数覆盖：全局默认、设备覆盖、会话临时覆盖、显式 false、清除设备配置。
- 命令预览函数覆盖：数值参数、空码率、默认 codec、布尔 flag。
- Vue 类型检查覆盖新组件 props 和 store 类型。
- 浏览器手动检查四页：设备、投屏会话、工具配置、参数设置。
- 手动检查小窗口宽度下文本不溢出、不互相遮挡。

建议新增 Vitest，仅测试纯 TypeScript 参数逻辑；组件层先用类型检查和手动 UI 验证控制成本。

## 13. 验收标准

- 前端主界面视觉和信息架构与 `docs/droiddock_ui_preview.html` 保持一致。
- PRD 中主界面、参数设置、多会话列表、工具配置和新手引导要求在前端都有对应入口。
- 用户能设置全局默认投屏/控制参数。
- 用户能为单台设备保存独立投屏/控制参数。
- 用户能恢复单台设备为全局默认。
- 启动投屏使用合并后的最终参数。
- 会话卡片展示本次会话生效参数摘要。
- ADB Pair 表单明确区分配对端口和连接端口，不持久化配对码。
- 构建通过，参数核心逻辑测试通过。

