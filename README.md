# DroidDock

DroidDock 是一个面向 macOS Apple Silicon 的 Android 手机投屏与控制桌面应用。第一版采用 Tauri + Vue + TypeScript，基于 `adb` 管理连接，基于 `scrcpy` 打开独立投屏窗口。

## 安装与运行 (macOS)

由于应用目前尚未进行 Apple 开发者官方签名，安装后打开可能会提示 **“DroidDock 已损坏，打不开”** 或 **“无法验证开发者”**。这并非文件损坏，而是 macOS 的安全机制 (Gatekeeper) 的限制。

请尝试以下任一方法解决：

### 方法 1：执行终端命令（推荐）

1. 将 `DroidDock.app` 拖入 **应用程序 (Applications)** 文件夹。
2. 打开 **终端 (Terminal.app)**，复制并执行以下命令：

```bash
sudo xattr -rd com.apple.quarantine /Applications/DroidDock.app
```

### 方法 2：右键手动开启

1. 在 **访达 (Finder)** 中进入 **应用程序** 文件夹。
2. 找到 **DroidDock**，按住键盘上的 **Control (⌃) 键** 并点击应用（或右键点击）。
3. 在弹出的菜单中选择 **“打开”**。
4. 在随后的警告对话框中，再次点击 **“打开”**。

## 开发环境

当前项目需要：

- Node.js 24+
- npm 11+
- Rust stable
- Xcode Command Line Tools

安装依赖：

```bash
npm install
```

前端构建校验：

```bash
npm run build
```

Tauri 开发模式：

```bash
npm run tauri:dev
```

构建 Apple Silicon `.dmg`：

```bash
npm run tauri:build
```

如果本机 `hdiutil` 无法创建 DMG，可先构建可运行的 `.app`：

```bash
npm run tauri:build:app
```

## 文档

- [产品需求文档](docs/product-requirements.md)
- [架构说明](docs/architecture.md)
- [UI 设计稿说明](docs/ui-design.md)
- [UI 静态设计稿](docs/ui-mockup.html)
- [scrcpy 使用方式调研](docs/scrcpy-usage.md)
