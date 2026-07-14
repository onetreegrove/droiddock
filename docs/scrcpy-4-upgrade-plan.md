# scrcpy 4.0 特性接入实施方案

## 背景

DroidDock 当前已支持一组基础 scrcpy 参数，包括分辨率、帧率、码率、视频编码、音频开关、控制开关、保持唤醒、熄屏、显示触摸、置顶和全屏。

scrcpy 4.0 新增或改进了一些适合 DroidDock 接入的能力。本方案聚焦以下四项：

1. `--keep-active`
2. 内置 `scrcpy` / `adb` 工具升级
3. `--background-color`
4. `--no-window-aspect-ratio-lock`

## 实施目标

- 支持通过 UI 配置 `--keep-active`。
- 支持配置 scrcpy 窗口背景色 `--background-color`。
- 支持控制窗口比例锁，必要时生成 `--no-window-aspect-ratio-lock`。
- 升级 bundled `scrcpy` / `adb`，并验证现有镜像、连接、会话能力兼容。
- 保持前端命令预览与 Rust 后端实际启动参数一致。
- 保证旧配置文件可以正常反序列化和迁移。
- 外部旧版 scrcpy 不会因为新增默认参数而直接启动失败。

## 关键实施约束

### 版本门控必须前置

新增参数只在 scrcpy 4.0+ 可用，因此不能先把 `keepActive: true` 写入默认配置并直接生成参数，再在后续批次补版本门控。

推荐顺序是：

1. 先建立 scrcpy 版本解析和能力判断。
2. 参数预览和后端启动共用同一套能力判断结果。
3. 只有当前 scrcpy 版本支持对应能力时，才允许 UI 启用配置并生成参数。
4. 版本不足时，UI 禁用对应控件并显示 `需要 scrcpy 4.0+`；后端启动前仍要兜底校验并返回中文错误。

在版本门控完成前，不应把任何 4.0 专属参数设置为会自动生效的默认参数。

### 配置迁移语义

旧配置兼容不只是“缺字段可反序列化”。新增默认值会改变启动行为，因此需要明确迁移策略：

- `schema_version < 2` 的旧配置保持原有行为：缺失 `keepActive` 时不自动生成 `--keep-active`，原有 `stayAwake` 配置不被静默覆盖。
- 全新安装生成的默认配置可以使用新推荐值：`keepActive: true`、`stayAwake: false`、`windowAspectRatioLock: true`。
- 旧配置迁移到 schema 2 时只升级结构和 schema 号，不应把 `default_scrcpy_options` 重建为新默认值。
- 如果未来决定对旧配置启用新推荐默认，必须把它作为独立迁移策略明确记录，并补充迁移测试和用户可感知说明，避免用户原有投屏习惯被无提示改变。
- `windowAspectRatioLock` 的 `undefined` 只表示“遵循 scrcpy 4.0 默认锁定比例”，不需要为了旧配置主动写入配置文件。

建议本次升级将 `CURRENT_CONFIG_SCHEMA_VERSION` 提升到 `2`，并把“保持旧配置行为”和“新安装默认值”分别写测试覆盖。

旧配置迁移到 schema 2 时，不能通过新的 `ScrcpyOptions::default()` 重建旧用户的 `default_scrcpy_options`。保存别名、最近连接地址、设备记录等其他配置字段时，也不能把缺失的 4.0 专属参数静默写成启用状态。

### 预览与启动参数一致性

当前项目同时存在前端本地预览和后端实际启动参数构造：

- 前端：`src/domain/scrcpyOptions.ts` 的 `buildScrcpyArgs` / `buildScrcpyCommand`。
- 后端：`src-tauri/src/scrcpy.rs` 的 `build_scrcpy_args`。
- IPC：`src-tauri/src/lib.rs` 的 `preview_scrcpy_args`。

本次新增 4.0 参数后，预览必须和实际启动使用同一套能力判断结果。

推荐方案：将 `preview_scrcpy_args` 改为能力感知接口，接收 `State<AppState>` 并返回 `Result<Vec<String>, String>`。预览时必须复用启动路径的工具选择逻辑，先通过 `resolve_tool("scrcpy", &config)` 或等价的 `diagnose_tool` 选出实际会用于启动的 scrcpy 可执行文件，再基于该路径解析版本，执行与启动一致的 4.0 能力判断、背景色校验和参数构造。前端命令预览优先使用该 IPC 结果，并处理预览错误。

如果仍保留前端本地预览，则 `buildScrcpyArgs` 必须显式传入 `scrcpyCapabilities` 或等价能力对象，并与 Rust 侧使用同一组测试 case 覆盖，避免预览和启动逻辑漂移。

### 前后端校验一致

背景色不能只在前端校验。后端会接收 Tauri invoke 参数和本地配置文件内容，因此 Rust 侧也需要做同等校验和规范化。

建议抽象两个同名能力：

- 前端：`normalizeBackgroundColor(value: string): string | null`
- 后端：`normalize_background_color(value: &str) -> Option<String>`

非法颜色值不得进入最终启动参数。后端可以选择在启动前返回明确错误，也可以丢弃非法值并记录诊断；为了避免用户以为配置已生效，推荐返回错误：

```text
背景色格式不正确，请使用 #RGB 或 #RRGGBB。
```

### 工具来源策略

当前项目不是在仓库中提交固定 bundled 二进制，而是通过安装逻辑下载到：

```text
~/Library/Application Support/DroidDock/tools/
```

因此“升级 bundled scrcpy / adb”应落实为更新下载 manifest 和安装验证，而不是简单替换仓库文件。

本次目标固定为：

- `src-tauri/src/tool_manifest.rs` 中 scrcpy 下载来源固定到 `v4.0`，不继续依赖 GitHub latest。
- scrcpy macOS aarch64 包必须记录并校验 sha256。
- platform-tools 优先固定到 37.0.0 的可审计下载 URL，并记录 sha256。
- 如果确认 Google 没有可长期审计的 37.0.0 固定 URL，才允许继续使用 `platform-tools-latest-darwin.zip` 作为 fallback；fallback 必须显式修改 manifest/校验策略，不能继续依赖“latest + 空 sha256 + release 禁止空 sha256”的组合。
- release 构建不允许固定下载项缺少必要 sha256；缺少 sha256 时应停止安装或阻止发布。

## 批次一：新增 scrcpy 参数能力

### 1. 支持 `--keep-active`

`scrcpy 4.0` 新增 `--keep-active`，用于周期性向系统发送用户活跃信号，避免设备因无操作而休眠。

相比 `--stay-awake`：

- `--keep-active` 不修改 Android 全局设置。
- `--keep-active` 不依赖设备是否插电。
- 更适合 DroidDock 的无线连接和长时间控制场景。

#### 数据结构

前端新增：

```ts
keepActive?: boolean
```

Rust 后端新增：

```rust
keep_active: Option<bool>
```

涉及文件：

- `src/types/app.ts`
- `src/lib/ipc/types.ts`
- `src/domain/scrcpyOptions.ts`
- `src-tauri/src/scrcpy.rs`
- `src-tauri/src/config.rs`

#### 参数生成

当 `keepActive === true` 时生成：

```bash
--keep-active
```

Rust 侧示例：

```rust
if options.keep_active.unwrap_or(false) {
    args.push("--keep-active".into());
}
```

注意：该逻辑只能在确认当前 scrcpy 版本 `>= 4.0` 后执行。版本不足时不能静默传参。

#### 默认值建议

建议将默认策略调整为：

```ts
{
  keepActive: true,
  stayAwake: false,
}
```

保留 `stayAwake` 字段和 UI 配置能力，但不再作为推荐默认项。

默认值调整仅适用于全新安装生成的配置。旧配置迁移到 schema 2 时，应保持旧行为，不自动启用 4.0 参数。

#### UI 文案

新增开关：

- 标题：`保持设备活跃`
- 说明：`运行期间防止设备因无操作而休眠，不修改系统全局设置。`

原 `stayAwake` 建议调整文案：

- 标题：`插电时保持唤醒`
- 说明：`使用 scrcpy --stay-awake，可能影响设备全局设置。`

### 2. 支持 `--background-color`

`scrcpy 4.0` 支持通过 `--background-color` 设置窗口内容之外的背景色。

#### 数据结构

前端新增：

```ts
backgroundColor?: string
```

Rust 后端新增：

```rust
background_color: Option<String>
```

#### 参数生成

当存在合法颜色值时生成：

```bash
--background-color=#234567
```

Rust 侧示例：

```rust
if let Some(background_color) = options.background_color.as_deref() {
    if !supports_scrcpy_4 {
        return Err("当前 scrcpy 版本低于 4.0，暂不支持背景色设置。请在工具配置中升级 scrcpy。".into());
    }
    let color = normalize_background_color(background_color)
        .ok_or_else(|| "背景色格式不正确，请使用 #RGB 或 #RRGGBB。".to_string())?;
    args.push(format!("--background-color={color}"));
}
```

注意：只有当前 scrcpy 版本 `>= 4.0` 时才允许生成该参数。版本不足且用户配置了背景色时，应提示用户升级 scrcpy，而不是静默丢弃配置或等 scrcpy 进程失败。

#### 校验规则

前端允许：

```text
#abc
abc
#aabbcc
aabbcc
```

前端不允许：

```text
red
#12
#abcd
#12345g
```

建议在前端统一规范化为 `#rrggbb`，并统一输出小写：

```text
567     -> #556677
234567  -> #234567
#AABBCC -> #aabbcc
```

Rust 侧必须执行同等规范化。不要直接拼接 `background_color` 原始字符串。

#### 默认值建议

不建议在配置中主动写入默认背景色。

```ts
backgroundColor: undefined
```

未配置时交给 scrcpy 4.0 使用自身默认背景色。

#### UI 文案

放入高级参数区：

- 标题：`背景色`
- 占位：`#234567`
- 说明：`窗口内容之外的背景颜色，支持 #RGB 或 #RRGGBB。`

可选预设：

- 默认
- 深灰
- 纯黑
- 自定义

### 3. 支持窗口比例锁控制

`scrcpy 4.0` 默认在调整窗口大小时锁定内容比例，避免黑边。旧行为可通过 `--no-window-aspect-ratio-lock` 恢复。

#### 数据结构

建议使用正向语义字段：

```ts
windowAspectRatioLock?: boolean
```

Rust 后端新增：

```rust
window_aspect_ratio_lock: Option<bool>
```

#### 参数生成

由于 scrcpy 4.0 默认锁定窗口比例，所以只有用户关闭该能力时才生成：

```bash
--no-window-aspect-ratio-lock
```

前端/Rust 逻辑：

```ts
if (options.windowAspectRatioLock === false) {
  args.push('--no-window-aspect-ratio-lock')
}
```

```rust
if options.window_aspect_ratio_lock == Some(false) {
    args.push("--no-window-aspect-ratio-lock".into());
}
```

注意：只有当前 scrcpy 版本 `>= 4.0` 时才允许生成该参数。版本不足且用户关闭比例锁时，应提示用户升级 scrcpy。

#### 默认值建议

```ts
windowAspectRatioLock: true
```

为了兼容旧配置，`undefined` 应视为 `true`。

该默认值可以作为内存中的 effective options 参与 UI 展示和参数合并，但不需要主动持久化到配置文件。只有用户显式关闭比例锁时，才需要保存 `windowAspectRatioLock: false`。

#### UI 文案

新增开关：

- 标题：`锁定窗口比例`
- 说明：`调整窗口大小时保持画面比例，避免黑边。`

默认开启。

## 批次一涉及文件

### 前端

#### `src/domain/scrcpyOptions.ts`

需要更新：

- 默认值。
- option merge 逻辑。
- 参数预览逻辑。
- scrcpy 版本能力门控。
- summary tags。
- `optionSummaryTagsFromArgs` 反解析逻辑。
- presets 展示。
- 背景色规范化与校验逻辑。

新增 summary 示例：

- `保持活跃`
- `背景 #234567`
- `自由缩放窗口`

#### `src/components/ParameterEditor.vue`

新增控件：

1. `keepActive` toggle。
2. `backgroundColor` input 或 select。
3. `windowAspectRatioLock` toggle。

当当前 scrcpy 版本低于 4.0 或版本未知时，以上 4.0 专属控件需要禁用，并显示 `需要 scrcpy 4.0+`。如果版本未知但工具可用，应引导用户重新检测工具状态。

#### `src/components/DeviceDetailPanel.vue`

检查命令预览是否自动展示新增参数。

#### `src/components/SettingsView.vue`

如果该页面复用 `ParameterEditor.vue`，全局默认配置应自动覆盖新增参数。

#### `src/types/app.ts`

同步新增 `ScrcpyOptions` 字段。

#### `src/lib/ipc/types.ts`

同步新增 IPC 类型字段。

### 后端

#### `src-tauri/src/scrcpy.rs`

新增字段和参数构造逻辑：

```rust
if supports_scrcpy_4 && options.keep_active.unwrap_or(false) {
    args.push("--keep-active".into());
}

if let Some(background_color) = options.background_color.as_deref() {
    if !supports_scrcpy_4 {
        return Err("当前 scrcpy 版本低于 4.0，暂不支持背景色设置。请在工具配置中升级 scrcpy。".into());
    }
    let color = normalize_background_color(background_color)
        .ok_or_else(|| "背景色格式不正确，请使用 #RGB 或 #RRGGBB。".to_string())?;
    args.push(format!("--background-color={color}"));
}

if supports_scrcpy_4 && options.window_aspect_ratio_lock == Some(false) {
    args.push("--no-window-aspect-ratio-lock".into());
}
```

如果 `supports_scrcpy_4 == false` 且用户配置了任一 4.0 专属参数，后端启动前应返回明确错误，而不是等 scrcpy 进程失败：

```text
当前 scrcpy 版本低于 4.0，暂不支持保持活跃、背景色或窗口比例锁设置。请在工具配置中升级 scrcpy。
```

#### `src-tauri/src/config.rs`

新增字段时使用 `Option<T>` 或 `#[serde(default)]`，保证旧配置反序列化不失败。

同时需要补充 schema 迁移逻辑，明确旧配置是否保持原行为或迁移到新默认值。建议新增：

- `CURRENT_CONFIG_SCHEMA_VERSION = 2`
- `migrate_config(config: AppConfig) -> AppConfig`
- `config_preserves_legacy_scrcpy_defaults_when_fields_are_missing`
- `config_uses_new_scrcpy_defaults_for_fresh_install`

#### `src-tauri/src/tools.rs`

需要新增或暴露 scrcpy 版本能力判断：

- 解析 `scrcpy --version` 第一行中的版本号。
- 将 `scrcpy 4.0`、`scrcpy 4.0.1` 判断为支持 4.0 能力。
- 将 `scrcpy 3.3.4`、版本解析失败、版本未知判断为不支持 4.0 专属参数。
- 将能力结果提供给 UI，或至少由后端在启动时兜底判断。

#### `src-tauri/src/tool_manifest.rs`

如果本次要保证内置安装到 scrcpy 4.0，应从“latest 动态下载”调整为可审计的固定来源：

- 固定 scrcpy `v4.0` macOS aarch64 下载 URL。
- 填写 scrcpy 包 sha256。
- 明确 platform-tools 37.0.0 的固定 URL 和 sha256，或说明继续使用 Google latest 并以版本检测作为验收。

## 批次一测试计划

### 单元测试

重点覆盖参数构造和配置合并逻辑：

1. `keepActive=true` 生成 `--keep-active`。
2. `keepActive=false` 或 `undefined` 不生成 `--keep-active`。
3. `backgroundColor=#234567` 生成 `--background-color=#234567`。
4. `backgroundColor=234567` 规范化后生成 `--background-color=#234567`。
5. `backgroundColor=567` 规范化后生成 `--background-color=#556677`。
6. 非法背景色不应进入最终参数。
7. `windowAspectRatioLock=false` 生成 `--no-window-aspect-ratio-lock`。
8. `windowAspectRatioLock=true` 或 `undefined` 不生成 `--no-window-aspect-ratio-lock`。
9. 新字段不影响旧 preset 和旧默认配置。
10. scrcpy 3.3.4 下不会生成任何 4.0 专属参数。
11. scrcpy 3.3.4 下用户显式启用 4.0 参数时，后端返回中文错误。
12. scrcpy 4.0 / 4.0.1 下允许生成 4.0 专属参数。
13. 旧 schema 配置缺失新字段时，不会被静默改成 `keepActive: true`。
14. 新安装默认配置使用新推荐值。

可能涉及测试文件：

- `src/domain/scrcpyOptions.test.ts`
- Rust 侧 `scrcpy.rs` 参数构造测试

### 手动验证

执行：

```bash
npm run test
npm run build
npm run tauri:dev
```

手动检查：

- 全局默认参数可以保存。
- 单设备参数可以保存。
- 命令预览正确。
- 启动 scrcpy 时参数正确。
- 老配置文件启动不报错。

## 批次二：升级内置 scrcpy / adb

### 1. 确认当前工具来源

先确认项目中的工具安装逻辑。当前代码会下载工具到应用支持目录，而不是从仓库内复制固定二进制。

重点检查：

- `src-tauri/src/tools.rs`
- `src-tauri/src/tool_manifest.rs`
- `src-tauri/tauri.conf.json`
- 构建脚本或 release 脚本
- 可能存在的 `src-tauri/binaries`、`resources` 或类似目录

需要确认：

- 当前 adb 存放位置。
- 当前 scrcpy 存放位置。
- macOS aarch64 bundle 如何打包。
- 工具版本校验逻辑是否写死版本。
- 安装逻辑是否使用 GitHub latest，是否需要固定为 `v4.0`。
- release 模式下 sha256 是否完整。

### 2. 固定工具下载来源

引入 scrcpy 4.0 对应 macOS Apple Silicon 包来源：

```text
scrcpy-macos-aarch64-v4.0.tar.gz
```

本次建议在 `src-tauri/src/tool_manifest.rs` 中固定：

- scrcpy v4.0 macOS aarch64 下载 URL。
- scrcpy v4.0 macOS aarch64 sha256。
- platform-tools 37.0.0 下载 URL 与 sha256。

platform-tools 处理策略按优先级执行：

1. 优先查找 Google 官方固定版本下载 URL，例如 `platform-tools_r37.0.0-darwin.zip` 对应的稳定地址；找到后将 URL 和 sha256 写入 manifest，release 安装继续执行固定 sha256 校验。
2. 如果找不到可长期审计的固定 URL，保留 `platform-tools-latest-darwin.zip` 时必须把它建模为“动态来源”，例如在 manifest 中增加 `checksum_policy` 或 `dynamic_latest` 标记。
3. 动态来源不能填写固定 sha256，也不能走当前 release 模式下“空 sha256 直接失败”的路径；需要改为下载后校验工具身份、Apple Silicon 兼容性、可执行权限，并要求 `adb version >= 37.0.0`。
4. 动态来源方案需要在文档和安装日志中明确安全取舍：安装结果可能随 Google latest 更新变化，不具备固定 sha256 的完全可复现性。

不建议继续使用 GitHub latest 作为 scrcpy 安装来源。latest 会让安装结果随上游变化，增加 sha256 审计和回归验证成本。

安装或替换后检查：

```bash
scrcpy --version
adb version
file scrcpy
file adb
```

需要确认：

- 可执行文件为 `arm64`。
- 可执行权限存在。
- 动态库路径正确。
- Tauri bundle 后仍可运行。

### 3. 更新版本检测逻辑

`tools.rs` 当前会记录工具版本字符串，但还没有面向参数能力的语义版本判断。需要同步更新。

建议策略：

- 最低支持版本保持当前下限，避免阻断用户使用外部旧版本。
- bundled 推荐版本更新为 `4.0`。
- adb 推荐版本更新为 `37.0.0`。
- 新增 `scrcpyCapabilities` 或等价字段，至少包含 `supportsKeepActive`、`supportsBackgroundColor`、`supportsWindowAspectRatioLock`。

版本解析测试需要覆盖：

- `scrcpy 4.0`：支持 4.0 能力。
- `scrcpy 4.0.1`：支持 4.0 能力。
- `scrcpy v4.0`：支持 4.0 能力。
- `scrcpy 3.3.4`：不支持 4.0 能力。
- 空字符串、非 scrcpy 输出、解析失败：不支持 4.0 能力。

如果 UI 显示内置工具版本，也同步更新文案。

### 4. 兼容性风险

新增参数只在 scrcpy 4.0+ 可用。

如果用户配置的是外部旧版 scrcpy，直接传递以下参数可能导致启动失败：

```bash
--keep-active
--background-color
--no-window-aspect-ratio-lock
```

#### 方案 A：强依赖 scrcpy 4.0+

优点：实现简单。

缺点：会影响使用外部旧版 scrcpy 的用户。

#### 方案 B：按版本启用参数

推荐采用该方案。

工具检测时解析 `scrcpy --version`，得到版本号。

规则：

- `keepActive` 需要 `>= 4.0`。
- `backgroundColor` 需要 `>= 4.0`。
- `windowAspectRatioLock` 需要 `>= 4.0`。

UI 行为：

- scrcpy 版本低于 4.0 时，禁用对应选项。
- 显示提示：`需要 scrcpy 4.0+`。

后端行为：

- 启动前兜底校验。
- 如果版本不足且用户配置了 4.0 参数，返回明确错误。
- 避免命令预览和实际启动参数不一致。

## 推荐实施顺序

1. 新增 TS/Rust 数据结构字段。
2. 新增 scrcpy 版本解析和 4.0 能力判断。
3. 将 `preview_scrcpy_args` 改为能力感知接口，确保预览和启动使用同一套校验与参数构造规则。
4. 更新配置反序列化与 schema 迁移，明确旧配置语义。
5. 更新前端 preview 展示逻辑，并接入版本门控或后端预览 IPC。
6. 更新 Rust 后端实际启动参数构造，并增加启动前兜底校验。
7. 更新 `ParameterEditor.vue` UI 控件和禁用提示。
8. 更新 summary tags、`optionSummaryTagsFromArgs` 和预设展示。
9. 补充前端和 Rust 单元测试。
10. 固定 scrcpy / adb 安装来源与 sha256。
11. 更新工具版本检测、能力展示和 UI 文案。
12. 执行完整验证。

## 最终默认配置建议

建议全新安装的 effective 默认配置为：

```ts
{
  maxSize: 1920,
  maxFps: 60,
  videoCodec: 'default',
  noAudio: true,
  keepActive: true,
  stayAwake: false,
  windowAspectRatioLock: true,
  backgroundColor: undefined,
}
```

其中 `windowAspectRatioLock: true` 是合并后的有效默认值，不要求主动写入持久化配置。旧配置的缺省行为按“配置迁移语义”执行，不应因为缺少字段就自动套用上述新默认。

## 验收标准

- 命令预览可以正确展示新增参数。
- 实际启动 scrcpy 时参数与预览一致。
- `preview_scrcpy_args` 和 `start_scrcpy` 对 4.0 能力、背景色校验、错误提示的行为一致。
- 全局默认配置和单设备配置均支持新增字段。
- 旧配置文件不需要手动迁移即可正常启动。
- 旧配置保存其他字段时，不会把缺失的 4.0 参数静默写成启用状态。
- scrcpy 4.0 bundled 工具可正常安装、检测和启动。
- 工具安装来源和 sha256 可审计，release 模式不会因为 manifest 缺失 sha256 而中断安装。
- 如果 platform-tools 使用动态 latest fallback，manifest 明确标记动态来源，安装后校验 `adb version >= 37.0.0`，且安装日志说明不可完全复现的安全取舍。
- 外部旧版 scrcpy 不会因新增参数直接启动失败，应有禁用提示或明确错误。
- 非法背景色不会进入后端最终启动参数。
- `npm run test` 通过。
- `npm run build` 通过。
- `npm run tauri:dev` 下手动验证通过。
