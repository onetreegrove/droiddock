# DroidDock

DroidDock 是一个面向 macOS Apple Silicon 的 Android 手机投屏与控制桌面应用。第一版采用 Tauri + Vue + TypeScript，基于 `adb` 管理连接，基于 `scrcpy` 打开独立投屏窗口。

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
- [UI 设计稿说明](docs/ui-design.md)
- [UI 静态设计稿](docs/ui-mockup.html)
- [scrcpy 使用方式调研](docs/scrcpy-usage.md)
