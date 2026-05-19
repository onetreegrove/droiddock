# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

DroidDock is a macOS Apple Silicon desktop app for Android screen mirroring and control. It uses a Vue 3 + TypeScript + Pinia frontend inside Tauri 2, with Rust commands managing `adb`, wireless pairing/connect flows, and `scrcpy` sessions.

## Common commands

- `npm run dev` — run the Vite frontend dev server on `127.0.0.1:1420`.
- `npm run tauri:dev` — run the full Tauri desktop app in development.
- `npm run build` — run TypeScript/Vue type checking with `vue-tsc --noEmit`, then build the frontend with Vite.
- `npm run test` — run Vitest once against `src`.
- `npm run test -- <path-or-pattern>` — run a focused Vitest subset.
- `npm run preview` — preview the built frontend.
- `npm run tauri` — invoke the Tauri CLI.
- `npm run tauri:build` — build the Apple Silicon Tauri app target.
- `npm run tauri:build:app` — build only the macOS app bundle.

There is no lint script and no standalone typecheck script; use `npm run build` for type checking.

## Tooling and requirements

- Node.js 24+ and npm 11+.
- Rust stable and Xcode Command Line Tools.
- Vite 6, Vue 3, Pinia, TypeScript strict mode, Vitest.
- Tauri uses the macOS private API feature for transparent window behavior.
- The app depends on `adb` and `scrcpy`; backend code can locate, validate, and install bundled tools.

## High-level architecture

### Frontend shell and state

- `src/main.ts` creates the Vue app, installs Pinia, imports global CSS, and mounts `App.vue`.
- `src/App.vue` is the root shell. It loads config and tool status, polls devices/sessions every 3 seconds, listens for session log events, handles close-with-running-sessions confirmation, and switches between devices, sessions, setup, and settings views.
- `src/stores/app.ts` is the main orchestration store. It composes config, devices, sessions, tools, and UI stores and exposes high-level actions for installing tools, refreshing runtime state, starting/stopping mirroring, wireless adb flows, saving aliases/options, and forgetting devices.
- `src/stores/config.ts` manages persisted app config, global scrcpy defaults, per-device options, tool paths, and aliases.
- `src/stores/devices.ts`, `src/stores/sessions.ts`, `src/stores/tools.ts`, and `src/stores/ui.ts` hold device selection, scrcpy session state/log listeners, adb/scrcpy readiness, navigation/modals/reconnect state, and toasts.

### Frontend domain and IPC layer

- `src/lib/ipc/client.ts` wraps Tauri `invoke` calls and normalizes IPC errors.
- `src/lib/ipc/types.ts` mirrors backend command payloads and result types.
- `src/domain/scrcpyOptions.ts` owns scrcpy presets, option merging, argument/command previews, and summary tags.
- `src/domain/deviceDetail.ts`, `src/domain/wireless.ts`, and `src/domain/reconnectFeedback.ts` contain UI-facing decision logic and form parsing separate from Vue components.

### Rust/Tauri backend

- `src-tauri/src/main.rs` delegates to `droiddock_lib::run`.
- `src-tauri/src/lib.rs` defines shared `AppState`, registers all Tauri commands, persists config updates, resolves adb/scrcpy tools, and bridges frontend commands to backend modules.
- `src-tauri/src/config.rs` stores app config under macOS Application Support, including tool paths, aliases, recent endpoints, device records, default scrcpy options, and per-device options.
- `src-tauri/src/tools.rs` locates and validates adb/scrcpy, checks executable architecture/version, and installs bundled tools.
- `src-tauri/src/command.rs` is the central helper for running external commands with stdout/stderr capture, optional stdin, and timeouts.
- `src-tauri/src/devices.rs` runs and parses `adb devices -l`, merges live devices with saved records, tracks offline known devices, aliases, and forget behavior.
- `src-tauri/src/wireless.rs` implements adb TCP/IP mode, connect, disconnect, and pair flows, including remembered endpoints and migration when wireless ports change.
- `src-tauri/src/scrcpy.rs` defines Rust scrcpy options and stable argument construction.
- `src-tauri/src/sessions.rs` launches scrcpy child processes, prevents duplicate active sessions per serial, tracks status/PIDs/args, kills sessions, captures logs, and emits `session-log` events to Vue.

## Development notes

- For UI changes, run `npm run tauri:dev` and exercise the app in the desktop window when possible; frontend type checks do not validate the native Tauri/scrcpy workflows.
- The frontend and backend maintain mirrored IPC types; update `src/lib/ipc/types.ts` when changing command payloads or response structs in Rust.
- Scrcpy option behavior is split between frontend preview/merging logic and backend argument construction; keep `src/domain/scrcpyOptions.ts` and `src-tauri/src/scrcpy.rs` aligned.
- Device identity spans USB serials, wireless endpoints, saved aliases, and per-device options; changes in `devices.rs` or `wireless.rs` often need corresponding store/domain updates.
