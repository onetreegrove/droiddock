# scrcpy 使用方式调研

调研日期：2026-04-25

## 结论

scrcpy 是 Android 设备投屏与控制工具，可以通过 USB 或 TCP/IP 连接设备，把 Android 画面和音频转发到电脑，并用电脑键鼠控制设备。它不要求 root，也不需要在手机上安装应用。

本机已验证的命令来源：

```bash
command -v scrcpy
# /Users/justonetree/.local/bin/scrcpy

scrcpy --version
# scrcpy 3.3.4

command -v adb
# /Users/justonetree/Library/Android/sdk/platform-tools/adb

adb version
# Android Debug Bridge version 1.0.41
# Version 37.0.0-14910828
```

官方资料入口：

- 官方站点：https://scrcpyapp.org/en/quickstart/
- 官方仓库：https://github.com/Genymobile/scrcpy
- 连接方式：https://scrcpyapp.org/en/guides/connection/
- 视频参数：https://scrcpyapp.org/en/guides/video/
- 音频参数：https://scrcpyapp.org/en/guides/audio/
- 录制参数：https://scrcpyapp.org/en/guides/recording/
- 快捷键：https://scrcpyapp.org/en/guides/shortcuts/

## 前置条件

- Android 设备系统至少 Android 5.0。
- 常规投屏需要在设备上开启 USB 调试，并在首次连接时允许电脑调试授权。
- 音频转发需要 Android 11+。Android 10 或更低版本会自动禁用音频。
- 部分小米设备如果能投屏但无法键鼠控制，需要额外开启开发者选项里的 `USB debugging (Security Settings)`，开启后通常需要重启设备。
- OTG 模式可以不启用 USB 调试，但只负责键鼠/手柄控制，不显示画面，也不转发音频。

## 快速开始

先确认设备可见：

```bash
adb devices
```

只有一台设备在线时，直接运行：

```bash
scrcpy
```

多台设备时指定序列号：

```bash
scrcpy -s 0123456789abcdef
```

只选择 USB 设备或 TCP/IP 设备：

```bash
scrcpy -d  # USB
scrcpy -e  # TCP/IP
```

## 常用场景

### 日常开发投屏

适合调试 App、看日志时保持手机亮屏：

```bash
scrcpy --max-size=1920 --max-fps=60 --no-audio --stay-awake
```

短参数版本：

```bash
scrcpy -m1920 --max-fps=60 --no-audio -w
```

### 低延迟或低带宽

降低分辨率、码率和帧率，能明显减少卡顿：

```bash
scrcpy -m1024 -b2M --max-fps=30 --no-audio
```

如果设备支持 H.265，可以尝试提升画质：

```bash
scrcpy --video-codec=h265 -m1920 --max-fps=60 --no-audio
```

H.264 通常更稳、延迟更低；H.265 更偏画质和压缩率。

### 演示和录屏

显示真实触摸点，并录制投屏内容：

```bash
scrcpy -m1920 --max-fps=60 --show-touches --record=demo.mp4
```

只录制，不显示窗口：

```bash
scrcpy --no-window --record=demo.mp4
```

只录视频：

```bash
scrcpy --no-audio --record=video.mp4
```

只录音频：

```bash
scrcpy --no-video --record=audio.opus
```

### 手机熄屏投屏

让手机物理屏幕关闭，但电脑继续显示和控制：

```bash
scrcpy --turn-screen-off --stay-awake
```

短参数版本：

```bash
scrcpy -Sw
```

### 固定窗口体验

```bash
scrcpy --window-title='Android Device' --window-width=800 --window-height=600
scrcpy --fullscreen
scrcpy --always-on-top
```

### 只看不控制

适合演示、监看，避免误触：

```bash
scrcpy --no-control
```

### 只控制不投屏

设备已开 USB 调试，但不需要画面和音频：

```bash
scrcpy --no-video --no-audio --mouse=uhid --keyboard=uhid
```

短参数版本：

```bash
scrcpy --no-video --no-audio -MK
```

### 无 USB 调试控制设备

OTG 模式不依赖 adb/USB 调试，但只能控制，不能投屏：

```bash
scrcpy --otg
```

如果有多台 USB 设备：

```bash
scrcpy --otg -s 0123456789abcdef
```

### 无线连接

自动方式：先用 USB 连接手机，并确保手机和电脑在同一网络，然后运行：

```bash
scrcpy --tcpip
```

如果设备已经监听 5555 端口：

```bash
scrcpy --tcpip=192.168.1.1
scrcpy --tcpip=192.168.1.1:5555
```

手动方式：

```bash
adb shell ip route | awk '{print $9}'
adb tcpip 5555
adb connect DEVICE_IP:5555
scrcpy
adb disconnect
```

### 启动指定 App

列出设备上的 App：

```bash
scrcpy --list-apps
```

启动指定包名：

```bash
scrcpy --start-app=org.mozilla.firefox
```

启动前先强制停止：

```bash
scrcpy --start-app=+org.mozilla.firefox
```

### 摄像头采集

Android 12+ 支持采集设备摄像头：

```bash
scrcpy --video-source=camera
scrcpy --video-source=camera --camera-facing=front
scrcpy --video-source=camera --camera-size=1920x1080 --camera-fps=60
```

### 虚拟显示

在独立虚拟显示里启动 App：

```bash
scrcpy --new-display=1920x1080 --start-app=org.videolan.vlc
```

## 快捷键

`MOD` 默认是左 `Alt` 或左 `Super`。在 macOS 上 `Super` 通常是 `Cmd`。

| 操作 | 快捷键 |
| --- | --- |
| 全屏切换 | `MOD+f` |
| 返回 | 右键，或 `MOD+b` |
| Home | 中键，或 `MOD+h` |
| 最近任务 | `MOD+s` |
| 旋转设备屏幕 | `MOD+r` |
| 旋转显示画面 | `MOD+左/右方向键` |
| 关闭设备屏幕并继续投屏 | `MOD+o` |
| 打开设备屏幕 | `MOD+Shift+o` |
| 展开通知栏 | `MOD+n` |
| 展开快捷设置 | 按住 `MOD` 后双击 `n` |
| 复制/剪切/粘贴 | `MOD+c` / `MOD+x` / `MOD+v` |
| 注入电脑剪贴板文本 | `MOD+Shift+v` |
| 显示/隐藏 FPS 输出 | `MOD+i` |
| 缩放窗口到 1:1 | `MOD+g` |
| 移除黑边适配窗口 | `MOD+w`，或双击黑边 |
| 模拟双指缩放/旋转 | `Ctrl+左键拖动` |

可以调整快捷键修饰符：

```bash
scrcpy --shortcut-mod=rctrl
scrcpy --shortcut-mod=lctrl,lsuper
```

## 文件投递

- 把 `.apk` 文件拖到 scrcpy 窗口：安装 APK。
- 把非 APK 文件拖到 scrcpy 窗口：默认推送到 `/sdcard/Download/`。

修改推送目录：

```bash
scrcpy --push-target=/sdcard/Movies/
```

## 排障

### `adb devices` 看不到设备

```bash
adb kill-server
adb start-server
adb devices
```

同时检查：

- 数据线是否支持数据传输。
- 手机是否开启 USB 调试。
- 手机上是否弹出并允许了调试授权。

### 设备状态是 `unauthorized`

解锁手机，重新插拔数据线，允许 RSA 调试授权。仍然不行时：

```bash
adb kill-server
adb start-server
adb devices
```

### 多台设备时报错

先看序列号：

```bash
adb devices
```

再指定设备：

```bash
scrcpy -s SERIAL
```

### 可以投屏但不能控制

优先检查设备端权限。小米等设备需要额外开启 `USB debugging (Security Settings)`，并在开启后重启。

临时只看画面：

```bash
scrcpy --no-control
```

### 音频不可用

- Android 10 或更低版本不支持音频转发。
- Android 11 启动 scrcpy 时需要保持设备解锁。
- 如果 Opus 编码失败，可以改用 AAC：

```bash
scrcpy --audio-codec=aac
```

不需要音频时直接关闭：

```bash
scrcpy --no-audio
```

### 无线连接失败

确认电脑和手机在同一网络，先用 USB 方式确认 adb 可用，再执行：

```bash
scrcpy --tcpip
```

手动流程完成后，收尾执行：

```bash
adb disconnect
```

### 性能差、卡顿或发热

从低配置开始逐步加码：

```bash
scrcpy -m1024 -b2M --max-fps=30 --no-audio
```

常见调优顺序：

1. 降低 `--max-size`。
2. 降低 `--video-bit-rate` / `-b`。
3. 降低 `--max-fps`。
4. 关闭音频 `--no-audio`。
5. 在 H.264 和 H.265 之间切换测试。

## 推荐预设

日常开发：

```bash
scrcpy -m1920 --max-fps=60 --no-audio -w
```

演示录屏：

```bash
scrcpy -m1920 --max-fps=60 --show-touches --record=demo.mp4
```

低带宽无线：

```bash
scrcpy --tcpip -m1024 -b2M --max-fps=30 --no-audio
```

息屏省电：

```bash
scrcpy -Sw -m1920 --max-fps=60 --no-audio
```

只控制：

```bash
scrcpy --no-video --no-audio -MK
```

