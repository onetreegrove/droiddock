# DroidDock 发展路线规划

制定日期：2026-05-08

依据文档：

- `docs/product-requirements.md`
- `docs/qtscrcpy-research-report.md`
- `docs/linkandroid-research-report.md`

## 1. 总体判断

QtScrcpy 和 LinkAndroid 给 DroidDock 的共同启发是：Android 投屏工具的长期价值不只在“能启动 scrcpy”，而在于把 ADB、scrcpy、设备连接、参数配置、无线配对、会话状态和故障诊断做成稳定、可理解、可恢复的桌面工作流。

两者路线差异也很清晰：

- QtScrcpy 走 Qt/C++ 自研客户端内核路线，控制力强，但维护成本高。
- LinkAndroid 走桌面应用编排 ADB/scrcpy 路线，产品能力完整，更贴近 DroidDock 当前 Tauri/Vue/Rust 方向。

DroidDock 未来应坚持轻量编排路线：

- 不重写 scrcpy 视频解码和渲染链路。
- 不绑定第三方定制 scrcpy。
- 优先使用官方 scrcpy 和官方 Android Platform Tools。
- 用 Tauri/Rust 承担工具链、进程、配置、日志和安全边界。
- 用 Vue 承担设备连接、参数配置、会话管理和错误引导体验。

## 2. 产品定位演进

### 当前定位

面向 macOS Apple Silicon 的 Android 手机投屏与控制桌面应用。DroidDock 是连接和会话控制台，投屏窗口由 scrcpy 独立提供。

### 中期定位

面向非技术用户的 Android 设备连接助手。用户不需要理解 `adb`、`scrcpy`、serial、pair/connect、端口等概念，就能完成工具安装、USB 连接、无线调试、投屏、多设备管理和常见故障修复。

### 长期定位

面向个人和轻量多设备场景的 Android 桌面控制台。除投屏外，逐步补充设备诊断、文件传输、应用管理、截图录屏、命令行排障和多设备批量操作。

## 3. 技术路线原则

### 3.1 坚持官方 scrcpy

短中期只使用官方 scrcpy，不维护 fork。

原因：

- DroidDock 当前价值在体验编排，不在底层投屏协议。
- 维护 scrcpy fork 会引入视频协议、server 参数、跨平台构建和兼容性成本。
- 官方 scrcpy 的更新和 Android 兼容修复更可靠。

允许的增强方式：

- 管理 scrcpy 进程。
- 生成结构化启动参数。
- 捕获 stdout/stderr。
- 记录会话状态。
- 通过 ADB 发送独立控制命令，如 Home、Back、截屏等。

不做：

- 不实现 `--droiddock-*` 定制 scrcpy 参数。
- 不内嵌投屏画面。
- 不自研 H.264 解码/渲染。

### 3.2 Tauri/Rust 作为安全命令层

LinkAndroid 的 Electron 配置偏宽，DroidDock 应反向吸取经验，保持 Tauri 的安全优势：

- 前端不能执行任意 shell。
- Rust commands 提供白名单能力。
- ADB/scrcpy 参数必须结构化。
- 文件路径必须由用户选择或由 App 管理目录生成。
- 日志和诊断数据默认本地保存，不上传。

### 3.3 参数模型优先于自由文本

自定义参数可以存在，但不应成为主要配置方式。

优先级：

1. 结构化参数控件。
2. 预设。
3. 高级自定义参数文本。

启动前应生成“最终参数快照”，并保存到会话记录中，避免全局默认值变化后历史会话显示漂移。

### 3.4 无线配对作为核心体验

LinkAndroid 的调研说明，Android 11+ 无线调试配对是非技术用户最难理解、也最值得产品化的流程。DroidDock 应把无线配对作为一等能力，而不是附属入口。

## 4. 阶段路线

## P0：MVP 稳定化

目标：把现有第一版范围打磨成稳定、可分发、可排障的 macOS Apple Silicon 工具。

### 核心能力

- 工具检测：`adb version`、`scrcpy --version`、可执行权限、Apple Silicon 架构匹配。
- 工具安装：自动安装 adb 和 scrcpy 到 App 管理目录。
- 设备发现：`adb devices -l`，识别 USB / Wi-Fi / unauthorized / offline。
- 投屏会话：启动、停止、失败、日志、参数快照。
- 参数配置：全局默认、设备覆盖、会话临时参数。
- 多设备投屏：至少两台设备可同时启动独立 scrcpy 窗口。
- 打包：Apple Silicon `.dmg`。

### 体验重点

- 不要求用户打开终端。
- 设备状态必须可解释。
- 投屏失败必须给出下一步操作。
- 会话列表展示本次实际生效参数。

### 验收标准

- 新机器安装后可完成工具安装、USB 投屏、无线投屏。
- `unauthorized`、`offline`、`more than one device/emulator`、`adb: unknown command pair` 都有清晰提示。
- App 退出时能询问是否关闭由 DroidDock 启动的 scrcpy 会话。

### 不纳入 P0

- 文件管理。
- 应用管理。
- 录屏。
- 设备预览。
- 群控。
- 自研投屏窗口。

## P1：连接体验强化

目标：把 DroidDock 做成比普通 scrcpy GUI 更可靠的连接助手，重点补齐 Android 11+ 无线调试。

### 1. 手动配对码配对

提供单独流程：

- 连接 IP。
- 连接端口。
- 配对 IP。
- 配对端口。
- 6 位配对码。

执行顺序：

```text
adb pair <pair_ip>:<pair_port>
输入 pairing code
adb connect <connect_ip>:<connect_port>
刷新设备列表
```

界面必须明确说明：

- 配对端口和连接端口通常不同。
- 配对码会过期，不能保存。
- 配对成功不代表连接成功，必须继续连接。

### 2. 二维码无线配对

参考 LinkAndroid 的路线，生成 ADB Wi-Fi 二维码：

```text
WIFI:T:ADB;S:ADBQR-connectPhoneOverWifi;P:<password>;;
```

再通过 mDNS 扫描：

- `adb-tls-pairing`
- `_adb-tls-pairing._tcp`
- `adb-tls-connect`
- `_adb-tls-connect._tcp`

状态流转：

- 等待手机扫码。
- 发现配对服务。
- 配对中。
- 等待连接服务。
- 连接中。
- 连接成功。
- 失败并展示可执行建议。

### 3. USB 转无线连接优化

保留传统流程：

```text
adb -s <serial> tcpip 5555
adb connect <ip>:5555
```

增强点：

- 自动尝试读取 `wlan0` IP。
- 保存最近成功 endpoint。
- 提供“重新连接”入口。
- 明确说明手机重启后可能需要重新开启无线调试或重新 USB 初始化。

### 验收标准

- Android 11+ 手机可通过手动配对码连接。
- 支持扫码配对的设备可通过二维码流程连接。
- pairing 端口和 connect 端口在 UI 中明确分离。
- 所有无线连接尝试都有结构化日志。

## P2：会话与诊断中台

目标：让 DroidDock 不只是启动器，而是可观测、可恢复的 scrcpy 会话管理工具。

### 1. 会话状态机

将投屏启动拆成明确阶段：

- `checking_tools`
- `checking_device`
- `building_args`
- `starting_process`
- `waiting_output`
- `running`
- `stopping`
- `stopped`
- `failed`

每个阶段记录：

- 开始时间。
- 结束时间。
- 命令。
- 参数。
- stdout。
- stderr。
- 退出码。
- 失败分类。

### 2. 错误诊断规则

建立错误规则库，覆盖：

- 设备未授权。
- 设备离线。
- 多设备冲突。
- 无线端口拒绝。
- adb pair 不支持。
- scrcpy server 启动失败。
- scrcpy binary 不可执行。
- 工具架构不匹配。
- macOS 权限或隔离属性导致不可执行。

输出形式：

- 原始错误。
- 用户可读解释。
- 推荐下一步。
- 可选“一键修复”动作。

### 3. 诊断导出

新增本地诊断包：

- DroidDock 版本。
- macOS 版本。
- adb 路径和版本。
- scrcpy 路径和版本。
- 最近设备列表。
- 最近会话日志。
- 最近无线连接日志。

隐私约束：

- 不包含配对码。
- 不包含截图。
- 不自动上传。

### 验收标准

- 投屏失败时，用户能知道失败发生在哪一步。
- 日志可复制，可导出。
- 常见问题能给出明确下一步。

## P3：轻量设备管理扩展

目标：在不偏离投屏主线的前提下，补充高频 Android 管理能力。

### 1. 截图

实现方式：

```text
adb -s <serial> exec-out screencap -p
```

产品体验：

- 保存到用户选择目录。
- 支持复制到剪贴板。
- 不默认保存历史截图。

### 2. ADB Shell

提供高级入口：

- 当前设备 shell。
- 当前 App 使用的 adb 路径。
- 输出可复制。
- 命令历史本地保存，默认关闭或限制条数。

限制：

- 明确标记为高级功能。
- 不在新手主流程中出现。

### 3. 文件传输轻量版

先做最小能力：

- 从电脑推送文件到设备下载目录。
- 从设备拉取指定路径文件。
- 不做完整文件管理器。

原因：

- 完整文件管理器复杂度高，容易偏离 DroidDock 主线。
- LinkAndroid 已证明这类能力有价值，但也会带来大量边界处理。

### 4. 应用安装

提供 APK 安装：

```text
adb -s <serial> install <apk_path>
```

暂不做应用市场、批量卸载或复杂应用管理。

### 验收标准

- 用户可对已连接设备截图。
- 高级用户可打开设备 shell。
- 可推送单个文件。
- 可安装 APK。

## P4：预览与多设备增强

目标：面向多设备场景提升效率，但保持可选和低资源占用。

### 1. 手动刷新预览

优先实现手动预览，不默认启动后台 scrcpy：

```text
adb exec-out screencap -p
```

设备卡片显示最近一次预览图。

配置：

- 全局开关：是否显示设备预览。
- 设备开关：是否为某台设备显示预览。
- 清除预览图。

### 2. 低频自动预览

作为可选能力：

- 每 30 秒 / 60 秒刷新一次。
- 仅对已启用设备生效。
- App 后台或低电量时暂停。
- 多设备时限制并发。

### 3. 批量操作

从低风险操作开始：

- 批量刷新设备。
- 批量停止投屏。
- 批量应用参数预设。
- 批量重连无线设备。

暂不做：

- 触摸事件群控。
- 键鼠同步操作。
- 定制 scrcpy 随动模式。

### 验收标准

- 多设备用户能快速识别设备画面。
- 预览功能不会默认消耗大量资源。
- 批量操作有明确确认。

## P5：高级投屏体验探索

目标：在前四阶段稳定后，再评估是否需要更深的投屏窗口控制。

### 可探索方向

- 投屏窗口布局管理。
- 启动后自动排列窗口。
- 记录每台设备的窗口位置。
- 通过 AppleScript 或系统窗口 API 做轻量窗口管理。
- scrcpy 版本更新策略。
- 可选实验版定制 scrcpy。

### 决策门槛

只有满足以下条件，才考虑定制 scrcpy 或更深集成：

- 官方 scrcpy 无法满足明确的高价值需求。
- 需求已被真实用户反复验证。
- 有可维护的 fork 构建链路。
- 有回退到官方 scrcpy 的机制。
- 不影响 P0-P4 的稳定体验。

## 5. 能力优先级

### 必须优先

1. 工具链可靠安装和检测。
2. USB 设备发现和授权提示。
3. 投屏会话状态机。
4. 全局默认 + 设备覆盖 + 会话临时参数。
5. Android 11+ 手动 `adb pair`。
6. 错误诊断和日志。

### 高价值增强

1. 二维码无线配对。
2. mDNS 自动发现。
3. 最近无线连接管理。
4. 诊断包导出。
5. 截图。
6. ADB shell。

### 谨慎推进

1. 文件管理。
2. 应用管理。
3. 设备预览。
4. 批量操作。
5. 投屏窗口布局管理。

### 暂不推进

1. 自研 scrcpy 客户端内核。
2. 维护定制 scrcpy fork。
3. 内嵌投屏画面。
4. 游戏键鼠映射。
5. 群控触摸事件同步。
6. Windows / Linux 版本。

## 6. 架构演进建议

### Rust 后端模块

建议长期保持以下边界：

- `ToolManager`：下载、安装、检测、版本、架构、可执行权限。
- `DeviceManager`：设备发现、状态解析、USB/Wi-Fi 判断、watch 或轮询。
- `PairingManager`：`adb pair`、`adb connect`、mDNS 扫描、无线连接历史。
- `ScrcpyOptions`：结构化参数、预设、合并、命令参数生成。
- `SessionManager`：scrcpy 子进程、状态机、日志、停止、重连。
- `DiagnosticManager`：错误分类、诊断包导出。
- `ConfigStore`：工具路径、设备别名、参数、无线 endpoint、用户偏好。

### 前端页面

建议长期拆成：

- `SetupView`：工具安装和检测。
- `DevicesView`：设备列表、连接入口、设备状态。
- `DeviceDetailView`：设备参数、设备操作、无线信息。
- `SessionsView`：投屏会话和日志。
- `SettingsView`：全局默认参数、工具路径、高级选项。
- `PairingWizard`：无线配对引导。
- `DiagnosticsPanel`：错误解释和日志导出。

### 数据模型

核心模型需要稳定：

- `ToolStatus`
- `DeviceRecord`
- `WirelessEndpoint`
- `PairingAttempt`
- `ScrcpyOptions`
- `EffectiveScrcpyOptions`
- `SessionRecord`
- `CommandLog`
- `DiagnosticFinding`

其中 `SessionRecord` 必须保存 `EffectiveScrcpyOptions` 快照。

## 7. 风险控制

### 工具版本风险

策略：

- 固定默认下载版本。
- 显示版本号。
- 允许用户手动选择路径。
- 版本升级走显式确认。

### GitHub 下载不稳定

策略：

- 下载失败给出手动安装路径。
- 支持重试。
- 后续可考虑镜像源，但必须展示来源。

### ADB 行为差异

策略：

- 记录 adb 原始输出。
- 错误规则库可迭代。
- 所有无线流程保存每一步日志。

### 多设备资源占用

策略：

- 投屏会话由用户显式启动。
- 预览默认关闭或手动刷新。
- 批量操作需确认。

### 安全边界

策略：

- 不提供任意 shell 给普通流程。
- 高级命令行入口明确标识风险。
- 配对码不保存。
- 诊断包不含截图和隐私内容。

## 8. 推荐执行顺序

```text
P0 MVP 稳定化
  -> P1 无线连接体验强化
  -> P2 会话与诊断中台
  -> P3 轻量设备管理扩展
  -> P4 预览与多设备增强
  -> P5 高级投屏体验探索
```

其中 P0-P2 是 DroidDock 的核心竞争力，必须优先投入。P3-P5 是产品广度增强，应在基础连接和投屏稳定后推进。

## 9. 近期建议

下一阶段最建议先做三件事：

1. 完整重审当前实现与 P0 验收项的差距，形成缺口清单。
2. 将 Android 11+ 无线配对从“能执行命令”升级为“完整引导流程”。
3. 建立会话状态机和错误诊断规则库，让每次失败都能被解释和复盘。

这三件事完成后，DroidDock 会从“scrcpy 启动器”进入“可靠 Android 连接控制台”的阶段。
