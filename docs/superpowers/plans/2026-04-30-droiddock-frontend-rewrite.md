# DroidDock Frontend Rewrite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite the DroidDock frontend to match `docs/droiddock_ui_preview.html` and satisfy `docs/product-requirements.md`, including global default parameters, per-device parameters, and session parameter previews.

**Architecture:** Keep Vue 3 + TypeScript + Pinia + Tauri invoke. Move app types into `src/types/app.ts`, pure scrcpy option logic into `src/domain/scrcpyOptions.ts`, and keep Tauri calls plus UI state in `src/stores/app.ts`. Add only the minimum Rust config extensions required to persist global and per-device scrcpy options.

**Tech Stack:** Vue 3, TypeScript, Pinia, Tauri v2, Rust, Vitest for pure TypeScript option tests.

---

## File Structure

- Modify: `package.json` and `package-lock.json`
  - Add `test` script and Vitest dependency.
- Create: `src/types/app.ts`
  - Shared frontend types for tools, devices, sessions, config, pages, modals, and scrcpy options.
- Create: `src/domain/scrcpyOptions.ts`
  - Presets, defaults, option merge, command preview, parameter summary, and boolean-safe cleanup helpers.
- Create: `src/domain/scrcpyOptions.test.ts`
  - Vitest coverage for option merge and command preview.
- Modify: `src-tauri/src/lib.rs`
  - Extend `AppConfig` and add commands for saving global and per-device scrcpy options.
- Modify: `src/stores/app.ts`
  - Use shared types, fetch config, manage pages/modals/drafts, call new config commands, compute effective options.
- Modify: `src/App.vue`
  - Replace current layout with the UI-preview shell and page switching.
- Replace or create components:
  - `src/components/AppSidebar.vue`
  - `src/components/AppHeader.vue`
  - `src/components/StatusChip.vue`
  - `src/components/DeviceList.vue`
  - `src/components/DeviceDetailPanel.vue`
  - `src/components/ParameterEditor.vue`
  - `src/components/CommandPreview.vue`
  - `src/components/SessionsView.vue`
  - `src/components/SessionCard.vue`
  - `src/components/SetupView.vue`
  - `src/components/SettingsView.vue`
  - `src/components/PairModal.vue`
  - `src/components/WirelessModal.vue`
  - `src/components/LogPanel.vue`
- Modify: `src/styles.css`
  - Replace the current light theme with the dark control-console theme from the UI preview, adjusted for real Vue components.

## Task 1: Add Option Domain Types and Tests

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`
- Create: `src/types/app.ts`
- Create: `src/domain/scrcpyOptions.ts`
- Create: `src/domain/scrcpyOptions.test.ts`

- [ ] **Step 1: Add Vitest dependency and script**

Run:

```bash
npm install -D vitest
```

Then ensure `package.json` contains:

```json
{
  "scripts": {
    "dev": "vite --host 127.0.0.1",
    "build": "vue-tsc --noEmit && vite build",
    "test": "vitest run",
    "preview": "vite preview --host 127.0.0.1",
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build --target aarch64-apple-darwin"
  }
}
```

Expected: `package-lock.json` updates and `npm run test` is available.

- [ ] **Step 2: Create shared frontend types**

Create `src/types/app.ts`:

```ts
export type PageKey = 'devices' | 'sessions' | 'setup' | 'settings';

export type ModalKey = null | 'pair' | 'wireless' | 'logs';

export type ToolStatus = {
  adb_path: string | null;
  scrcpy_path: string | null;
  adb_version: string | null;
  scrcpy_version: string | null;
  adb_ok: boolean;
  scrcpy_ok: boolean;
};

export type Device = {
  serial: string;
  state: string;
  model: string | null;
  product: string | null;
  connection: 'usb' | 'wireless';
  alias: string | null;
  raw: string;
};

export type SessionLogLine = {
  timestamp: number;
  level: string;
  message: string;
};

export type SessionInfo = {
  session_id: string;
  serial: string;
  alias: string | null;
  pid: number;
  status: 'idle' | 'starting' | 'running' | 'stopped' | 'failed';
  started_at: number;
  connection: string;
  args: string[];
  last_message: string | null;
};

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

export type PairRequest = {
  host: string;
  pair_port: number;
  pairing_code: string;
  connect_port: number | null;
};
```

- [ ] **Step 3: Write failing option tests**

Create `src/domain/scrcpyOptions.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import {
  buildScrcpyArgs,
  buildScrcpyCommand,
  clearUndefinedOptions,
  defaultScrcpyOptions,
  mergeScrcpyOptions,
  optionSummaryTags,
  presetOptions,
} from './scrcpyOptions';

describe('scrcpy option domain', () => {
  it('merges global, device, and session options while preserving explicit false', () => {
    const result = mergeScrcpyOptions(
      { ...defaultScrcpyOptions, noAudio: true, stayAwake: true },
      { maxFps: 30, noAudio: false },
      { videoBitRate: '2M' },
    );

    expect(result).toEqual({
      ...defaultScrcpyOptions,
      noAudio: false,
      stayAwake: true,
      maxFps: 30,
      videoBitRate: '2M',
    });
  });

  it('removes undefined values without dropping false values', () => {
    expect(clearUndefinedOptions({ noAudio: false, maxFps: undefined, videoBitRate: '' })).toEqual({
      noAudio: false,
      videoBitRate: '',
    });
  });

  it('builds scrcpy args in stable UI order', () => {
    const args = buildScrcpyArgs('R9YT301WXXX', {
      maxSize: 1920,
      maxFps: 60,
      videoBitRate: '4M',
      videoCodec: 'h265',
      noAudio: true,
      stayAwake: true,
      alwaysOnTop: true,
    });

    expect(args).toEqual([
      '-s',
      'R9YT301WXXX',
      '--max-size=1920',
      '--max-fps=60',
      '--video-bit-rate=4M',
      '--video-codec=h265',
      '--no-audio',
      '--stay-awake',
      '--always-on-top',
    ]);
  });

  it('skips empty bit rate and default codec in command preview', () => {
    expect(buildScrcpyCommand('SERIAL', { maxSize: 1024, videoBitRate: '', videoCodec: 'default' })).toBe(
      'scrcpy -s SERIAL --max-size=1024',
    );
  });

  it('exposes PRD presets and summary tags', () => {
    expect(presetOptions.lowBandwidth).toEqual({
      maxSize: 1024,
      videoBitRate: '2M',
      maxFps: 30,
      noAudio: true,
    });
    expect(optionSummaryTags({ maxSize: 1024, maxFps: 30, videoBitRate: '2M', noAudio: true })).toEqual([
      '1024p',
      '30fps',
      '2M',
      'no-audio',
    ]);
  });
});
```

- [ ] **Step 4: Run test and confirm it fails before implementation**

Run:

```bash
npm run test
```

Expected: FAIL because `src/domain/scrcpyOptions.ts` does not exist yet.

- [ ] **Step 5: Implement option domain**

Create `src/domain/scrcpyOptions.ts`:

```ts
import type { PresetId, ScrcpyOptions } from '../types/app';

export const defaultScrcpyOptions: ScrcpyOptions = {
  maxSize: 1920,
  maxFps: 60,
  videoCodec: 'default',
  noAudio: true,
  stayAwake: true,
};

export const presetOptions: Record<PresetId, ScrcpyOptions> = {
  daily: { maxSize: 1920, maxFps: 60, noAudio: true, stayAwake: true },
  lowBandwidth: { maxSize: 1024, videoBitRate: '2M', maxFps: 30, noAudio: true },
  demo: { maxSize: 1920, maxFps: 60, showTouches: true, alwaysOnTop: true },
  batterySaver: { maxSize: 1920, maxFps: 60, noAudio: true, stayAwake: true, turnScreenOff: true },
  viewOnly: { maxSize: 1920, maxFps: 60, noControl: true },
};

export const presetLabels: Record<PresetId, string> = {
  daily: '日常使用',
  lowBandwidth: '低带宽无线',
  demo: '演示模式',
  batterySaver: '息屏省电',
  viewOnly: '只看不控',
};

export function clearUndefinedOptions(options: ScrcpyOptions): ScrcpyOptions {
  return Object.fromEntries(Object.entries(options).filter(([, value]) => value !== undefined)) as ScrcpyOptions;
}

export function mergeScrcpyOptions(
  globalOptions: ScrcpyOptions,
  deviceOptions?: ScrcpyOptions,
  sessionOptions?: ScrcpyOptions,
): ScrcpyOptions {
  return clearUndefinedOptions({
    ...globalOptions,
    ...(deviceOptions ?? {}),
    ...(sessionOptions ?? {}),
  });
}

export function buildScrcpyArgs(serial: string, options: ScrcpyOptions): string[] {
  const args = ['-s', serial];
  if (options.maxSize !== undefined) args.push(`--max-size=${options.maxSize}`);
  if (options.maxFps !== undefined) args.push(`--max-fps=${options.maxFps}`);
  if (options.videoBitRate?.trim()) args.push(`--video-bit-rate=${options.videoBitRate.trim()}`);
  if (options.videoCodec && options.videoCodec !== 'default') args.push(`--video-codec=${options.videoCodec}`);
  if (options.noAudio) args.push('--no-audio');
  if (options.noControl) args.push('--no-control');
  if (options.stayAwake) args.push('--stay-awake');
  if (options.turnScreenOff) args.push('--turn-screen-off');
  if (options.showTouches) args.push('--show-touches');
  if (options.alwaysOnTop) args.push('--always-on-top');
  if (options.fullscreen) args.push('--fullscreen');
  return args;
}

export function buildScrcpyCommand(serial: string, options: ScrcpyOptions): string {
  return ['scrcpy', ...buildScrcpyArgs(serial, options)].join(' ');
}

export function optionSummaryTags(options: ScrcpyOptions): string[] {
  const tags: string[] = [];
  if (options.maxSize !== undefined) tags.push(`${options.maxSize}p`);
  if (options.maxFps !== undefined) tags.push(`${options.maxFps}fps`);
  if (options.videoBitRate?.trim()) tags.push(options.videoBitRate.trim());
  if (options.videoCodec && options.videoCodec !== 'default') tags.push(options.videoCodec);
  if (options.noAudio) tags.push('no-audio');
  if (options.noControl) tags.push('no-control');
  if (options.stayAwake) tags.push('stay-awake');
  if (options.turnScreenOff) tags.push('screen-off');
  if (options.showTouches) tags.push('touches');
  if (options.alwaysOnTop) tags.push('top');
  if (options.fullscreen) tags.push('fullscreen');
  return tags;
}
```

- [ ] **Step 6: Verify tests pass**

Run:

```bash
npm run test
```

Expected: PASS for `src/domain/scrcpyOptions.test.ts`.

- [ ] **Step 7: Commit**

```bash
git add package.json package-lock.json src/types/app.ts src/domain/scrcpyOptions.ts src/domain/scrcpyOptions.test.ts
git commit -m "test: add scrcpy option domain"
```

## Task 2: Extend Config Persistence Commands

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/types/app.ts` only if Rust serialization shape requires a type correction

- [ ] **Step 1: Add Rust config structs**

Modify the Rust type section near `ScrcpyOptions`:

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceOptionEntry {
    preset_id: Option<String>,
    options: ScrcpyOptions,
    updated_at: u64,
}

impl Default for ScrcpyOptions {
    fn default() -> Self {
        Self {
            max_size: Some(1920),
            max_fps: Some(60),
            video_bit_rate: None,
            video_codec: Some("default".to_string()),
            no_audio: Some(true),
            no_control: None,
            stay_awake: Some(true),
            turn_screen_off: None,
            show_touches: None,
            always_on_top: None,
            fullscreen: None,
        }
    }
}
```

Then change `AppConfig` to:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    adb_path: Option<String>,
    scrcpy_path: Option<String>,
    device_aliases: HashMap<String, String>,
    recent_endpoints: Vec<String>,
    default_scrcpy_options: ScrcpyOptions,
    default_preset_id: String,
    device_scrcpy_options: HashMap<String, DeviceOptionEntry>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            adb_path: None,
            scrcpy_path: None,
            device_aliases: HashMap::new(),
            recent_endpoints: Vec::new(),
            default_scrcpy_options: ScrcpyOptions::default(),
            default_preset_id: "daily".to_string(),
            device_scrcpy_options: HashMap::new(),
        }
    }
}
```

- [ ] **Step 2: Add config save commands**

Add after `save_device_alias`:

```rust
#[tauri::command]
fn save_default_scrcpy_options(
    state: State<'_, AppState>,
    options: ScrcpyOptions,
    preset_id: String,
) -> Result<AppConfig, String> {
    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?;
    config.default_scrcpy_options = options;
    config.default_preset_id = preset_id;
    save_config(&config)?;
    Ok(config.clone())
}

#[tauri::command]
fn save_device_scrcpy_options(
    state: State<'_, AppState>,
    serial: String,
    options: ScrcpyOptions,
    preset_id: Option<String>,
) -> Result<AppConfig, String> {
    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?;
    config.device_scrcpy_options.insert(
        serial,
        DeviceOptionEntry {
            preset_id,
            options,
            updated_at: now_secs(),
        },
    );
    save_config(&config)?;
    Ok(config.clone())
}

#[tauri::command]
fn clear_device_scrcpy_options(
    state: State<'_, AppState>,
    serial: String,
) -> Result<AppConfig, String> {
    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?;
    config.device_scrcpy_options.remove(&serial);
    save_config(&config)?;
    Ok(config.clone())
}
```

- [ ] **Step 3: Register commands**

Add these names to `tauri::generate_handler!`:

```rust
save_default_scrcpy_options,
save_device_scrcpy_options,
clear_device_scrcpy_options,
```

- [ ] **Step 4: Verify Rust and frontend build**

Run:

```bash
npm run build
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: persist scrcpy option settings"
```

## Task 3: Rewrite Pinia Store Around Pages, Config, and Drafts

**Files:**
- Modify: `src/stores/app.ts`

- [ ] **Step 1: Replace store imports and state**

Use imports:

```ts
import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import {
  AppConfig,
  Device,
  ModalKey,
  PageKey,
  PairRequest,
  PresetId,
  ScrcpyOptions,
  SessionInfo,
  SessionLogLine,
  ToolStatus,
} from '../types/app';
import {
  defaultScrcpyOptions,
  mergeScrcpyOptions,
  presetOptions,
} from '../domain/scrcpyOptions';
```

Set state:

```ts
state: () => ({
  toolStatus: null as ToolStatus | null,
  appConfig: null as AppConfig | null,
  devices: [] as Device[],
  sessions: [] as SessionInfo[],
  currentPage: 'devices' as PageKey,
  selectedSerial: null as string | null,
  modal: null as ModalKey,
  loading: false,
  busy: {} as Record<string, boolean>,
  logs: [] as string[],
  sessionLogs: {} as Record<string, SessionLogLine[]>,
  globalDraftOptions: { ...defaultScrcpyOptions } as ScrcpyOptions,
  globalDraftPresetId: 'daily' as PresetId,
  deviceDraftOptions: {} as Record<string, ScrcpyOptions>,
  sessionDraftOptions: {} as Record<string, ScrcpyOptions>,
}),
```

- [ ] **Step 2: Add getters**

```ts
getters: {
  selectedDevice: (state) => state.devices.find((device) => device.serial === state.selectedSerial) ?? null,
  isToolsReady: (state) => Boolean(state.toolStatus?.adb_ok && state.toolStatus?.scrcpy_ok),
  availableDeviceCount: (state) => state.devices.filter((device) => device.state === 'device').length,
  activeSession: (state) => (serial: string) => state.sessions.find((session) => session.serial === serial && session.status === 'running') ?? null,
  deviceOptionEntry: (state) => (serial: string) => state.appConfig?.device_scrcpy_options[serial] ?? null,
  effectiveOptions: (state) => (serial: string) => {
    const globalOptions = state.appConfig?.default_scrcpy_options ?? defaultScrcpyOptions;
    const deviceOptions = state.appConfig?.device_scrcpy_options[serial]?.options;
    const sessionOptions = state.sessionDraftOptions[serial];
    return mergeScrcpyOptions(globalOptions, deviceOptions, sessionOptions);
  },
},
```

- [ ] **Step 3: Add config actions**

```ts
async fetchAppConfig() {
  try {
    this.appConfig = await invoke<AppConfig>('get_app_config');
    this.globalDraftOptions = { ...(this.appConfig.default_scrcpy_options ?? defaultScrcpyOptions) };
    this.globalDraftPresetId = this.appConfig.default_preset_id ?? 'daily';
  } catch (error) {
    this.log(`读取配置失败: ${String(error)}`);
  }
},

async saveDefaultOptions(options: ScrcpyOptions, presetId: PresetId) {
  try {
    this.appConfig = await invoke<AppConfig>('save_default_scrcpy_options', { options, presetId });
    this.globalDraftOptions = { ...options };
    this.globalDraftPresetId = presetId;
    this.log('已保存全局默认参数');
  } catch (error) {
    this.log(`保存全局默认参数失败: ${String(error)}`);
    throw error;
  }
},

async saveDeviceOptions(serial: string, options: ScrcpyOptions, presetId: PresetId | null) {
  try {
    this.appConfig = await invoke<AppConfig>('save_device_scrcpy_options', { serial, options, presetId });
    delete this.sessionDraftOptions[serial];
    this.log(`已保存设备参数: ${serial}`);
  } catch (error) {
    this.log(`保存设备参数失败: ${String(error)}`);
    throw error;
  }
},

async clearDeviceOptions(serial: string) {
  try {
    this.appConfig = await invoke<AppConfig>('clear_device_scrcpy_options', { serial });
    delete this.deviceDraftOptions[serial];
    delete this.sessionDraftOptions[serial];
    this.log(`已恢复全局默认: ${serial}`);
  } catch (error) {
    this.log(`恢复全局默认失败: ${String(error)}`);
    throw error;
  }
},
```

- [ ] **Step 4: Update existing actions**

Update `refreshDevices` so the first available device becomes selected:

```ts
async refreshDevices() {
  this.loading = true;
  try {
    this.devices = await invoke<Device[]>('list_devices');
    if (!this.selectedSerial && this.devices.length > 0) {
      this.selectedSerial = this.devices[0].serial;
    }
    if (this.selectedSerial && !this.devices.some((device) => device.serial === this.selectedSerial)) {
      this.selectedSerial = this.devices[0]?.serial ?? null;
    }
  } catch (error) {
    this.log(`刷新设备失败: ${String(error)}`);
  } finally {
    this.loading = false;
  }
},
```

Update `startMirror`:

```ts
async startMirror(serial: string, options?: ScrcpyOptions) {
  try {
    const finalOptions = options ?? this.effectiveOptions(serial);
    const info = await invoke<SessionInfo>('start_scrcpy', { serial, options: finalOptions });
    await this.refreshSessions();
    this.currentPage = 'sessions';
    this.log(`已启动投屏: ${serial}`);
    return info;
  } catch (error) {
    this.log(`启动失败: ${String(error)}`);
    throw error;
  }
},
```

- [ ] **Step 5: Update app initialization later in `App.vue`**

This task only prepares the store. `App.vue` will call `fetchAppConfig` in Task 5.

- [ ] **Step 6: Verify typecheck**

Run:

```bash
npm run build
```

Expected: Vue template errors can exist until components are rewritten; TypeScript module errors in `src/stores/app.ts` must be resolved before proceeding.

- [ ] **Step 7: Commit**

```bash
git add src/stores/app.ts
git commit -m "feat: restructure app store for frontend shell"
```

## Task 4: Build Reusable UI Components and Dark Theme

**Files:**
- Modify: `src/styles.css`
- Create: `src/components/StatusChip.vue`
- Create: `src/components/AppSidebar.vue`
- Create: `src/components/AppHeader.vue`
- Create: `src/components/CommandPreview.vue`
- Create: `src/components/ParameterEditor.vue`

- [ ] **Step 1: Replace base theme**

Replace `src/styles.css` with dark shell tokens and shared classes derived from `docs/droiddock_ui_preview.html`. Keep selectors component-safe:

```css
:root {
  color-scheme: dark;
  --bg: #0d0f12;
  --bg2: #141720;
  --bg3: #1a1e2a;
  --bg4: #1f2433;
  --bg5: #252b3b;
  --bg6: #2a3147;
  --border1: rgba(255, 255, 255, 0.06);
  --border2: rgba(255, 255, 255, 0.10);
  --border3: rgba(255, 255, 255, 0.18);
  --acc: #3dd9eb;
  --acc-d: rgba(61, 217, 235, 0.15);
  --green: #4ade80;
  --green-d: rgba(74, 222, 128, 0.12);
  --yellow: #fbbf24;
  --yellow-d: rgba(251, 191, 36, 0.12);
  --red: #f87171;
  --red-d: rgba(248, 113, 113, 0.12);
  --t1: #e8ecf4;
  --t2: #8892a4;
  --t3: #4f5a6e;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  overflow: hidden;
  background: #111318;
  color: var(--t1);
  font-family: "DM Sans", "Segoe UI", -apple-system, BlinkMacSystemFont, sans-serif;
  font-size: 13px;
  letter-spacing: 0;
  -webkit-font-smoothing: antialiased;
}

button,
input,
select {
  font: inherit;
}

button {
  border: 0;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  min-height: 28px;
  border-radius: 8px;
  padding: 5px 12px;
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
  cursor: pointer;
}

.btn:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.btn-primary {
  background: var(--acc);
  color: #0d0f12;
}

.btn-ghost {
  border: 1px solid var(--border2);
  background: transparent;
  color: var(--t2);
}

.btn-danger {
  border: 1px solid rgba(248, 113, 113, 0.2);
  background: var(--red-d);
  color: var(--red);
}

.mono {
  font-family: "JetBrains Mono", "Courier New", monospace;
}
```

Continue porting shared `.shell`, `.sidebar`, `.main`, `.page-header`, `.chip`, `.dot`, `.field-select`, `.toggle-card`, `.command-preview`, `.modal-overlay`, and `.modal-card` classes from the preview. Keep card radius at `8px` or `10px`.

- [ ] **Step 2: Create `StatusChip.vue`**

```vue
<script setup lang="ts">
defineProps<{
  tone: 'green' | 'yellow' | 'red' | 'blue' | 'gray';
  label: string;
  dot?: boolean;
}>();
</script>

<template>
  <span :class="['chip', `chip-${tone}`]">
    <span v-if="dot" :class="['dot', `dot-${tone}`]"></span>
    {{ label }}
  </span>
</template>
```

- [ ] **Step 3: Create `CommandPreview.vue`**

```vue
<script setup lang="ts">
defineProps<{ command: string }>();
</script>

<template>
  <div class="command-preview">
    <div class="command-label">命令预览</div>
    <div class="command-text mono">{{ command }}</div>
  </div>
</template>
```

- [ ] **Step 4: Create `AppHeader.vue`**

```vue
<script setup lang="ts">
defineProps<{
  title: string;
  subtitle?: string;
}>();
</script>

<template>
  <header class="page-header">
    <div>
      <div class="page-title">{{ title }}</div>
      <div v-if="subtitle" class="page-subtitle">{{ subtitle }}</div>
    </div>
    <div class="header-actions">
      <slot name="actions"></slot>
    </div>
  </header>
</template>
```

- [ ] **Step 5: Create `AppSidebar.vue`**

Use store `currentPage`, `isToolsReady`, session count, tool versions:

```vue
<script setup lang="ts">
import { computed } from 'vue';
import { useAppStore } from '../stores/app';

const store = useAppStore();
const runningCount = computed(() => store.sessions.filter((session) => session.status === 'running').length);
</script>

<template>
  <aside class="sidebar">
    <div class="titlebar"></div>
    <div class="brand">
      <div class="brand-icon">D</div>
      <span class="brand-name">DroidDock</span>
    </div>
    <div :class="['tool-pill', store.isToolsReady ? 'ok' : 'warn']">
      <span :class="['dot', store.isToolsReady ? 'dot-green' : 'dot-yellow']"></span>
      {{ store.isToolsReady ? 'Tools ready' : 'Tools missing' }}
    </div>
    <div class="sidebar-divider"></div>
    <nav class="nav">
      <button :class="['nav-item', { active: store.currentPage === 'devices' }]" @click="store.currentPage = 'devices'">设备</button>
      <button :class="['nav-item', { active: store.currentPage === 'sessions' }]" @click="store.currentPage = 'sessions'">
        <span>投屏会话</span><span v-if="runningCount" class="nav-badge">{{ runningCount }}</span>
      </button>
      <button :class="['nav-item', { active: store.currentPage === 'setup' }]" @click="store.currentPage = 'setup'">工具配置</button>
      <button :class="['nav-item', { active: store.currentPage === 'settings' }]" @click="store.currentPage = 'settings'">参数设置</button>
    </nav>
    <div class="sidebar-spacer"></div>
    <div class="sidebar-footer">
      <div class="tool-version"><span class="mono">adb</span><span class="mono">{{ store.toolStatus?.adb_version || '-' }}</span></div>
      <div class="tool-version"><span class="mono">scrcpy</span><span class="mono">{{ store.toolStatus?.scrcpy_version || '-' }}</span></div>
    </div>
  </aside>
</template>
```

- [ ] **Step 6: Create `ParameterEditor.vue`**

Implement with `v-model:options`, no internal persistence:

```vue
<script setup lang="ts">
import type { ScrcpyOptions } from '../types/app';

const props = defineProps<{ options: ScrcpyOptions }>();
const emit = defineEmits<{ 'update:options': [value: ScrcpyOptions] }>();

function patch(value: Partial<ScrcpyOptions>) {
  emit('update:options', { ...props.options, ...value });
}
</script>

<template>
  <div class="parameter-editor">
    <div class="param-row">
      <span class="param-label">最大分辨率</span>
      <select class="field-select" :value="options.maxSize ?? 1920" @change="patch({ maxSize: Number(($event.target as HTMLSelectElement).value) })">
        <option :value="1920">1920 (推荐)</option>
        <option :value="1280">1280</option>
        <option :value="1024">1024</option>
      </select>
    </div>
    <div class="param-row">
      <span class="param-label">最大帧率</span>
      <select class="field-select" :value="options.maxFps ?? 60" @change="patch({ maxFps: Number(($event.target as HTMLSelectElement).value) })">
        <option :value="60">60 fps</option>
        <option :value="45">45 fps</option>
        <option :value="30">30 fps</option>
      </select>
    </div>
    <div class="param-row">
      <span class="param-label">视频编码</span>
      <select class="field-select" :value="options.videoCodec ?? 'default'" @change="patch({ videoCodec: ($event.target as HTMLSelectElement).value as ScrcpyOptions['videoCodec'] })">
        <option value="default">默认</option>
        <option value="h264">H.264</option>
        <option value="h265">H.265</option>
      </select>
    </div>
    <div class="param-row">
      <span class="param-label">视频码率</span>
      <select class="field-select" :value="options.videoBitRate ?? ''" @change="patch({ videoBitRate: ($event.target as HTMLSelectElement).value })">
        <option value="">默认</option>
        <option value="16M">16 Mbps</option>
        <option value="8M">8 Mbps</option>
        <option value="4M">4 Mbps</option>
        <option value="2M">2 Mbps</option>
      </select>
    </div>
    <div class="toggle-grid">
      <button :class="['toggle-card', { on: options.noAudio }]" @click="patch({ noAudio: !options.noAudio })">禁用音频<span>--no-audio</span></button>
      <button :class="['toggle-card', { on: options.noControl }]" @click="patch({ noControl: !options.noControl })">只看不控<span>--no-control</span></button>
      <button :class="['toggle-card', { on: options.stayAwake }]" @click="patch({ stayAwake: !options.stayAwake })">保持亮屏<span>--stay-awake</span></button>
      <button :class="['toggle-card', { on: options.turnScreenOff }]" @click="patch({ turnScreenOff: !options.turnScreenOff })">息屏投屏<span>--turn-screen-off</span></button>
      <button :class="['toggle-card', { on: options.showTouches }]" @click="patch({ showTouches: !options.showTouches })">显示触摸<span>--show-touches</span></button>
      <button :class="['toggle-card', { on: options.alwaysOnTop }]" @click="patch({ alwaysOnTop: !options.alwaysOnTop })">置顶窗口<span>--always-on-top</span></button>
      <button :class="['toggle-card', { on: options.fullscreen }]" @click="patch({ fullscreen: !options.fullscreen })">全屏<span>--fullscreen</span></button>
    </div>
  </div>
</template>
```

- [ ] **Step 7: Verify style and type baseline**

Run:

```bash
npm run build
```

Expected: build may fail for unused old component imports until Task 5 rewires `App.vue`; no syntax errors in new files.

- [ ] **Step 8: Commit**

```bash
git add src/styles.css src/components/AppSidebar.vue src/components/AppHeader.vue src/components/StatusChip.vue src/components/CommandPreview.vue src/components/ParameterEditor.vue
git commit -m "feat: add DroidDock shell UI primitives"
```

## Task 5: Rewrite App Shell and Device Page

**Files:**
- Modify: `src/App.vue`
- Create: `src/components/DeviceList.vue`
- Create: `src/components/DeviceDetailPanel.vue`

- [ ] **Step 1: Rewrite `App.vue`**

Use this structure:

```vue
<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import AppSidebar from './components/AppSidebar.vue';
import DeviceList from './components/DeviceList.vue';
import DeviceDetailPanel from './components/DeviceDetailPanel.vue';
import SessionsView from './components/SessionsView.vue';
import SetupView from './components/SetupView.vue';
import SettingsView from './components/SettingsView.vue';
import PairModal from './components/PairModal.vue';
import WirelessModal from './components/WirelessModal.vue';
import { useAppStore } from './stores/app';

const store = useAppStore();
let poller: number | undefined;

onMounted(async () => {
  await store.fetchAppConfig();
  await store.fetchToolStatus();
  await store.refreshDevices();
  await store.refreshSessions();
  poller = window.setInterval(async () => {
    await store.refreshDevices();
    await store.refreshSessions();
    for (const session of store.sessions) {
      if (session.status === 'running') {
        await store.fetchSessionLogs(session.session_id);
      }
    }
  }, 3000);
});

onUnmounted(() => {
  if (poller) window.clearInterval(poller);
});
</script>

<template>
  <main class="app-shell">
    <AppSidebar />
    <section class="main">
      <div v-if="store.currentPage === 'devices'" class="page active">
        <DeviceList />
        <DeviceDetailPanel />
      </div>
      <SessionsView v-else-if="store.currentPage === 'sessions'" />
      <SetupView v-else-if="store.currentPage === 'setup'" />
      <SettingsView v-else />
    </section>
    <PairModal v-if="store.modal === 'pair'" />
    <WirelessModal v-if="store.modal === 'wireless'" />
  </main>
</template>
```

- [ ] **Step 2: Create `DeviceList.vue`**

Implement device cards and connection actions:

```vue
<script setup lang="ts">
import { computed } from 'vue';
import AppHeader from './AppHeader.vue';
import StatusChip from './StatusChip.vue';
import { useAppStore } from '../stores/app';

const store = useAppStore();
const subtitle = computed(() => `${store.devices.length} 台已发现 · ${store.availableDeviceCount} 台可用`);

function stateLabel(state: string) {
  if (state === 'device') return '可用';
  if (state === 'unauthorized') return '待授权';
  if (state === 'offline') return '离线';
  return state;
}

function stateTone(state: string) {
  if (state === 'device') return 'green';
  if (state === 'unauthorized') return 'yellow';
  if (state === 'offline') return 'red';
  return 'gray';
}
</script>

<template>
  <div class="devices-layout">
    <section class="device-column">
      <AppHeader title="设备" :subtitle="subtitle">
        <template #actions>
          <button class="btn btn-ghost" @click="store.modal = 'pair'">ADB Pair</button>
          <button class="btn btn-ghost" :disabled="store.loading" @click="store.refreshDevices">
            {{ store.loading ? '刷新中...' : '刷新' }}
          </button>
        </template>
      </AppHeader>
      <div class="device-list">
        <button
          v-for="device in store.devices"
          :key="device.serial"
          :class="['device-card', { selected: store.selectedSerial === device.serial }]"
          @click="store.selectedSerial = device.serial"
        >
          <div class="device-card-main">
            <div class="device-icon">▯</div>
            <div class="device-info">
              <div class="device-name">{{ device.alias || device.model || '未知设备' }}</div>
              <div class="device-model mono">{{ device.model || device.product || device.serial }}</div>
            </div>
            <div class="device-chips">
              <StatusChip :tone="stateTone(device.state)" :label="stateLabel(device.state)" dot />
              <StatusChip tone="gray" :label="device.connection === 'usb' ? 'USB' : '无线'" />
            </div>
          </div>
          <div class="device-serial mono">{{ device.serial }}</div>
          <div v-if="device.state === 'unauthorized'" class="device-warning">请在手机上允许 USB 调试授权</div>
        </button>
        <div class="connection-actions">
          <div class="section-label">连接方式</div>
          <button class="connection-button" @click="store.modal = 'wireless'">USB 转无线连接</button>
          <button class="connection-button" @click="store.modal = 'pair'">ADB Pair 无线配对</button>
        </div>
      </div>
    </section>
    <slot></slot>
  </div>
</template>
```

- [ ] **Step 3: Create `DeviceDetailPanel.vue`**

Use effective options and command preview:

```vue
<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import CommandPreview from './CommandPreview.vue';
import ParameterEditor from './ParameterEditor.vue';
import StatusChip from './StatusChip.vue';
import { buildScrcpyCommand, presetLabels, presetOptions } from '../domain/scrcpyOptions';
import type { PresetId, ScrcpyOptions } from '../types/app';
import { useAppStore } from '../stores/app';

const store = useAppStore();
const editorOptions = ref<ScrcpyOptions>({});
const activePreset = ref<PresetId>('daily');
const device = computed(() => store.selectedDevice);
const hasDeviceOptions = computed(() => Boolean(device.value && store.deviceOptionEntry(device.value.serial)));
const command = computed(() => device.value ? buildScrcpyCommand(device.value.serial, editorOptions.value) : 'scrcpy');
const canLaunch = computed(() => device.value?.state === 'device' && store.isToolsReady);
const launchHint = computed(() => {
  if (!device.value) return '请选择设备';
  if (!store.isToolsReady) return '请先完成工具配置';
  if (device.value.state === 'unauthorized') return '请先在手机上允许 USB 调试授权';
  if (device.value.state === 'offline') return '设备已离线，请重新连接';
  return '';
});

watch(() => store.selectedSerial, (serial) => {
  if (!serial) return;
  editorOptions.value = { ...store.effectiveOptions(serial) };
  activePreset.value = store.deviceOptionEntry(serial)?.presetId ?? store.appConfig?.default_preset_id ?? 'daily';
}, { immediate: true });

function applyPreset(presetId: PresetId) {
  activePreset.value = presetId;
  editorOptions.value = { ...presetOptions[presetId] };
}

async function saveForDevice() {
  if (!device.value) return;
  await store.saveDeviceOptions(device.value.serial, editorOptions.value, activePreset.value);
}

async function resetToGlobal() {
  if (!device.value) return;
  await store.clearDeviceOptions(device.value.serial);
  editorOptions.value = { ...store.effectiveOptions(device.value.serial) };
}

async function launch() {
  if (!device.value) return;
  store.sessionDraftOptions[device.value.serial] = { ...editorOptions.value };
  await store.startMirror(device.value.serial, editorOptions.value);
}
</script>

<template>
  <section v-if="device" class="device-detail">
    <div class="device-hero">
      <div class="hero-icon">▯</div>
      <div>
        <div class="hero-title">{{ device.alias || device.model || '未知设备' }}</div>
        <div class="hero-chips">
          <StatusChip :tone="device.state === 'device' ? 'green' : device.state === 'unauthorized' ? 'yellow' : 'red'" :label="device.state === 'device' ? '可用' : device.state === 'unauthorized' ? '待授权' : '离线'" dot />
          <StatusChip tone="gray" :label="device.connection === 'usb' ? 'USB' : '无线'" />
        </div>
      </div>
    </div>
    <div class="metadata-grid">
      <span>Serial</span><span class="mono">{{ device.serial }}</span>
      <span>型号</span><span>{{ device.model || '-' }}</span>
      <span>产品</span><span>{{ device.product || '-' }}</span>
    </div>
    <section class="detail-section">
      <div class="section-head">
        <div>
          <div class="section-title">投屏参数</div>
          <StatusChip :tone="hasDeviceOptions ? 'blue' : 'gray'" :label="hasDeviceOptions ? '使用设备独立设置' : '使用全局默认'" />
        </div>
        <div class="inline-actions">
          <select class="field-select" :value="activePreset" @change="applyPreset(($event.target as HTMLSelectElement).value as PresetId)">
            <option v-for="(label, id) in presetLabels" :key="id" :value="id">{{ label }}</option>
          </select>
          <button class="btn btn-ghost" @click="saveForDevice">保存为此设备设置</button>
          <button class="btn btn-ghost" :disabled="!hasDeviceOptions" @click="resetToGlobal">恢复全局默认</button>
        </div>
      </div>
      <ParameterEditor v-model:options="editorOptions" />
      <CommandPreview :command="command" />
    </section>
    <footer class="launch-bar">
      <button class="btn btn-primary launch-button" :disabled="!canLaunch" @click="launch">启动投屏</button>
      <span class="hint-text">{{ launchHint }}</span>
    </footer>
  </section>
  <section v-else class="device-detail empty-detail">
    <div class="empty-title">未发现设备</div>
    <div class="empty-copy">请连接 USB 设备，或使用 ADB Pair 添加无线设备。</div>
  </section>
</template>
```

- [ ] **Step 4: Remove old component imports**

Delete old references to `Sidebar`, `DiscoverView`, and old `DeviceDetailView` from `App.vue`. The files can remain until the final cleanup task.

- [ ] **Step 5: Verify device page build**

Run:

```bash
npm run build
```

Expected: PASS or only errors from not-yet-created `SessionsView`, `SetupView`, `SettingsView`, `PairModal`, `WirelessModal`. Resolve missing imports in Task 6 before final build.

- [ ] **Step 6: Commit**

```bash
git add src/App.vue src/components/DeviceList.vue src/components/DeviceDetailPanel.vue
git commit -m "feat: rewrite device control page"
```

## Task 6: Implement Sessions, Setup, Settings, and Modals

**Files:**
- Create: `src/components/SessionsView.vue`
- Create: `src/components/SessionCard.vue`
- Create: `src/components/SetupView.vue`
- Replace: `src/components/SettingsView.vue`
- Create: `src/components/PairModal.vue`
- Create: `src/components/WirelessModal.vue`
- Create: `src/components/LogPanel.vue`

- [ ] **Step 1: Create `SessionCard.vue`**

```vue
<script setup lang="ts">
import { computed } from 'vue';
import { optionSummaryTags } from '../domain/scrcpyOptions';
import type { SessionInfo } from '../types/app';
import { useAppStore } from '../stores/app';

const props = defineProps<{ session: SessionInfo }>();
const store = useAppStore();
const tags = computed(() => optionSummaryTags(store.effectiveOptions(props.session.serial)));
const running = computed(() => props.session.status === 'running');
</script>

<template>
  <article :class="['session-card', session.status]">
    <div :class="['session-strip', session.status]"></div>
    <div class="session-body">
      <div class="session-main">
        <div class="session-title-row">
          <span class="session-title">{{ session.alias || session.serial }}</span>
          <span class="chip" :class="running ? 'chip-green' : session.status === 'failed' ? 'chip-red' : 'chip-gray'">{{ running ? '运行中' : session.status === 'failed' ? '失败' : '已停止' }}</span>
          <span class="chip chip-gray">{{ session.connection === 'usb' ? 'USB' : '无线' }}</span>
        </div>
        <div class="session-meta mono">{{ session.serial }} · PID {{ session.pid }}</div>
        <div v-if="session.last_message" class="session-message">{{ session.last_message }}</div>
        <div class="tag-row"><span v-for="tag in tags" :key="tag" class="param-tag mono">{{ tag }}</span></div>
      </div>
      <div class="session-actions">
        <button v-if="running" class="btn btn-danger" @click="store.stopMirror(session.session_id)">停止</button>
        <button v-else class="btn btn-ghost" @click="store.startMirror(session.serial)">重连</button>
        <button class="btn btn-ghost" @click="store.fetchSessionLogs(session.session_id)">日志</button>
      </div>
    </div>
  </article>
</template>
```

- [ ] **Step 2: Create `SessionsView.vue`**

```vue
<script setup lang="ts">
import { computed } from 'vue';
import AppHeader from './AppHeader.vue';
import SessionCard from './SessionCard.vue';
import { useAppStore } from '../stores/app';

const store = useAppStore();
const runningCount = computed(() => store.sessions.filter((session) => session.status === 'running').length);
</script>

<template>
  <section class="page active">
    <AppHeader title="投屏会话" :subtitle="`${store.sessions.length} 个会话 · ${runningCount} 个运行中`">
      <template #actions>
        <button class="btn btn-danger" :disabled="runningCount === 0" @click="store.stopAllSessions">停止全部</button>
        <button class="btn btn-ghost" @click="store.currentPage = 'devices'">新建投屏</button>
      </template>
    </AppHeader>
    <div class="session-list">
      <SessionCard v-for="session in store.sessions" :key="session.session_id" :session="session" />
      <div v-if="store.sessions.length === 0" class="empty-panel">暂无投屏会话，请到设备页启动投屏。</div>
    </div>
  </section>
</template>
```

Add `stopAllSessions` action to store:

```ts
async stopAllSessions() {
  try {
    this.sessions = await invoke<SessionInfo[]>('stop_all_sessions');
    this.log('已停止全部投屏会话');
  } catch (error) {
    this.log(`停止全部失败: ${String(error)}`);
  }
},
```

- [ ] **Step 3: Create `SetupView.vue`**

Include tool cards, disabled install/manual buttons if no command exists:

```vue
<script setup lang="ts">
import AppHeader from './AppHeader.vue';
import StatusChip from './StatusChip.vue';
import { useAppStore } from '../stores/app';

const store = useAppStore();
</script>

<template>
  <section class="page active">
    <AppHeader title="工具配置" subtitle="adb 与 scrcpy 路径管理" />
    <div class="setup-body">
      <div class="section-label">已安装工具</div>
      <div class="tool-card">
        <div><strong class="mono">adb</strong><div class="tool-path mono">{{ store.toolStatus?.adb_path || '未找到' }}</div></div>
        <StatusChip :tone="store.toolStatus?.adb_ok ? 'green' : 'red'" :label="store.toolStatus?.adb_version || '缺失'" />
      </div>
      <div class="tool-card">
        <div><strong class="mono">scrcpy</strong><div class="tool-path mono">{{ store.toolStatus?.scrcpy_path || '未找到' }}</div></div>
        <StatusChip :tone="store.toolStatus?.scrcpy_ok ? 'green' : 'red'" :label="store.toolStatus?.scrcpy_version || '缺失'" />
      </div>
      <div class="install-panel">
        <div><div class="panel-title">自动安装</div><div class="hint-text">工具将安装到 ~/Library/Application Support/DroidDock/tools/</div></div>
        <button class="btn btn-ghost" disabled>自动安装待接入</button>
      </div>
      <div class="guide-panel">
        <div class="section-label">新手入门</div>
        <ol class="guide-list">
          <li>安装工具：点击自动安装或手动配置路径。</li>
          <li>开启开发者选项：设置中连续点击版本号 7 次。</li>
          <li>开启 USB 调试：进入开发者选项打开 USB 调试。</li>
          <li>连接并授权：USB 连接后在手机弹窗中允许调试。</li>
          <li>启动投屏：在设备页选择设备并点击启动。</li>
        </ol>
      </div>
    </div>
  </section>
</template>
```

- [ ] **Step 4: Replace `SettingsView.vue`**

Use global draft and presets:

```vue
<script setup lang="ts">
import ParameterEditor from './ParameterEditor.vue';
import AppHeader from './AppHeader.vue';
import { presetLabels, presetOptions } from '../domain/scrcpyOptions';
import type { PresetId } from '../types/app';
import { useAppStore } from '../stores/app';

const store = useAppStore();

function applyPreset(id: PresetId) {
  store.globalDraftPresetId = id;
  store.globalDraftOptions = { ...presetOptions[id] };
}
</script>

<template>
  <section class="page active">
    <AppHeader title="参数设置" subtitle="全局默认投屏参数，适用于所有未单独配置的设备">
      <template #actions>
        <button class="btn btn-primary" @click="store.saveDefaultOptions(store.globalDraftOptions, store.globalDraftPresetId)">保存全局默认</button>
      </template>
    </AppHeader>
    <div class="settings-body">
      <div class="section-label">快速应用预设</div>
      <div class="preset-row">
        <button v-for="(label, id) in presetLabels" :key="id" :class="['preset-chip', { on: store.globalDraftPresetId === id }]" @click="applyPreset(id as PresetId)">
          {{ label }}
        </button>
      </div>
      <div class="settings-panel">
        <div class="section-label">画面与控制参数</div>
        <ParameterEditor v-model:options="store.globalDraftOptions" />
      </div>
      <div class="error-table">
        <div><span class="mono">unauthorized</span><span>请解锁手机，并在弹窗中允许 USB 调试</span></div>
        <div><span class="mono">offline</span><span>设备已离线，请重新插拔或重连无线调试</span></div>
        <div><span class="mono">Connection refused</span><span>无线调试端口不可用，请检查 IP 和端口</span></div>
        <div><span class="mono">failed to authenticate</span><span>配对失败，请重新生成配对码</span></div>
        <div><span class="mono">device not found</span><span>设备不存在或已断开，请刷新设备列表</span></div>
      </div>
    </div>
  </section>
</template>
```

- [ ] **Step 5: Create `PairModal.vue`**

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { useAppStore } from '../stores/app';

const store = useAppStore();
const host = ref('');
const pairPort = ref('');
const pairingCode = ref('');
const connectPort = ref('');

async function submit() {
  await store.adbPair({
    host: host.value,
    pair_port: Number(pairPort.value),
    pairing_code: pairingCode.value,
    connect_port: connectPort.value ? Number(connectPort.value) : null,
  });
  host.value = '';
  pairPort.value = '';
  pairingCode.value = '';
  connectPort.value = '';
  store.modal = null;
}
</script>

<template>
  <div class="modal-overlay" @click.self="store.modal = null">
    <section class="modal-card">
      <header class="modal-header"><div><div class="modal-title">ADB Pair 无线配对</div><div class="modal-subtitle">适用于 Android 11+ 无线调试</div></div><button class="btn btn-ghost" @click="store.modal = null">关闭</button></header>
      <div class="modal-body">
        <div class="modal-note">配对端口和连接端口通常不同，请按手机屏幕分别填写。</div>
        <div class="form-grid">
          <label>配对 IP<input v-model="host" class="field-input" placeholder="192.168.1.100" /></label>
          <label>配对端口<input v-model="pairPort" class="field-input" placeholder="38521" /></label>
        </div>
        <label>配对码（6 位）<input v-model="pairingCode" class="field-input code-input" maxlength="6" placeholder="123456" /></label>
        <div class="form-grid">
          <label>连接 IP<input :value="host" class="field-input" disabled /></label>
          <label>连接端口<input v-model="connectPort" class="field-input" placeholder="39845" /></label>
        </div>
      </div>
      <footer class="modal-footer"><button class="btn btn-ghost" @click="store.modal = null">取消</button><button class="btn btn-primary" :disabled="!host || !pairPort || !pairingCode" @click="submit">执行配对并连接</button></footer>
    </section>
  </div>
</template>
```

- [ ] **Step 6: Create `WirelessModal.vue`**

```vue
<script setup lang="ts">
import { computed, ref } from 'vue';
import { useAppStore } from '../stores/app';

const store = useAppStore();
const selectedSerial = ref(store.devices.find((device) => device.connection === 'usb' && device.state === 'device')?.serial ?? '');
const host = ref('');
const port = ref('5555');
const usbDevices = computed(() => store.devices.filter((device) => device.connection === 'usb' && device.state === 'device'));

async function submit() {
  if (!selectedSerial.value || !host.value) return;
  await store.adbTcpip(selectedSerial.value);
  await store.adbConnect(`${host.value}:${port.value || '5555'}`);
  store.modal = null;
}
</script>

<template>
  <div class="modal-overlay" @click.self="store.modal = null">
    <section class="modal-card">
      <header class="modal-header"><div><div class="modal-title">USB 转无线连接</div><div class="modal-subtitle">通过 USB 建立无线调试连接</div></div><button class="btn btn-ghost" @click="store.modal = null">关闭</button></header>
      <div class="modal-body">
        <label>USB 设备<select v-model="selectedSerial" class="field-select"><option v-for="device in usbDevices" :key="device.serial" :value="device.serial">{{ device.model || device.serial }}</option></select></label>
        <div class="form-grid">
          <label>手机 IP<input v-model="host" class="field-input" placeholder="192.168.1.100" /></label>
          <label>端口<input v-model="port" class="field-input" /></label>
        </div>
      </div>
      <footer class="modal-footer"><button class="btn btn-ghost" @click="store.modal = null">取消</button><button class="btn btn-primary" :disabled="!selectedSerial || !host" @click="submit">连接</button></footer>
    </section>
  </div>
</template>
```

- [ ] **Step 7: Create `LogPanel.vue`**

```vue
<script setup lang="ts">
import type { SessionLogLine } from '../types/app';
defineProps<{ lines: SessionLogLine[] }>();
</script>

<template>
  <div class="log-panel">
    <div v-for="line in lines" :key="`${line.timestamp}-${line.message}`" :class="['log-line', line.level]">
      [{{ new Date(line.timestamp * 1000).toLocaleTimeString() }}] {{ line.message }}
    </div>
  </div>
</template>
```

- [ ] **Step 8: Verify full frontend build**

Run:

```bash
npm run build
```

Expected: PASS.

- [ ] **Step 9: Commit**

```bash
git add src/components/SessionsView.vue src/components/SessionCard.vue src/components/SetupView.vue src/components/SettingsView.vue src/components/PairModal.vue src/components/WirelessModal.vue src/components/LogPanel.vue src/stores/app.ts
git commit -m "feat: add sessions setup settings and modals"
```

## Task 7: Polish Responsive Layout and Remove Old Components

**Files:**
- Modify: `src/styles.css`
- Delete: `src/components/Sidebar.vue`
- Delete: `src/components/DiscoverView.vue`
- Delete: `src/components/DeviceDetailView.vue`

- [ ] **Step 1: Add layout CSS for all new components**

Complete `src/styles.css` classes referenced by tasks 4-6:

```css
.app-shell {
  display: flex;
  height: 100vh;
  min-width: 900px;
  background: var(--bg);
}

.main {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  display: flex;
}

.page.active {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
}

.devices-layout {
  display: grid;
  grid-template-columns: 320px 1fr;
  width: 100%;
  height: 100%;
  min-width: 0;
}

.device-column {
  display: flex;
  flex-direction: column;
  min-width: 0;
  border-right: 1px solid var(--border1);
}

.device-list,
.session-list,
.setup-body,
.settings-body {
  overflow-y: auto;
  padding: 14px 22px;
}

.device-detail {
  min-width: 0;
  overflow-y: auto;
}

.toggle-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 6px;
}

@media (max-width: 960px) {
  .app-shell {
    min-width: 760px;
  }
  .devices-layout {
    grid-template-columns: 280px 1fr;
  }
  .toggle-grid {
    grid-template-columns: 1fr;
  }
}
```

Then port remaining exact class styling from the preview for visual fidelity.

- [ ] **Step 2: Delete unused old components**

Run:

```bash
rm src/components/Sidebar.vue src/components/DiscoverView.vue src/components/DeviceDetailView.vue
```

- [ ] **Step 3: Verify no stale imports**

Run:

```bash
rg "Sidebar|DiscoverView|DeviceDetailView" src
```

Expected: no output.

- [ ] **Step 4: Verify build and tests**

Run:

```bash
npm run test
npm run build
```

Expected: both PASS.

- [ ] **Step 5: Commit**

```bash
git add src/styles.css src/components
git add -u src/components
git commit -m "feat: polish DroidDock frontend layout"
```

## Task 8: Manual UI Verification

**Files:**
- No source files unless verification finds defects.

- [ ] **Step 1: Start Vite dev server**

Run:

```bash
npm run dev
```

Expected: Vite serves at `http://127.0.0.1:5173/` or the next available port.

- [ ] **Step 2: Check the device page**

Open the local URL in the in-app browser or a browser. Verify:

- Sidebar uses DroidDock brand and four navigation items.
- Device page shows header, device list, connection actions, selected device detail, parameter editor, command preview, and launch button.
- Unauthorized device disables launch and shows the USB authorization hint.
- Changing parameters updates command preview.

- [ ] **Step 3: Check sessions page**

Navigate to 投屏会话. Verify:

- Empty state appears when no sessions exist.
- If sessions exist, cards show status, connection, PID, parameter summary, stop/reconnect buttons.
- Stop all button disables when no running sessions exist.

- [ ] **Step 4: Check setup page**

Navigate to 工具配置. Verify:

- adb and scrcpy path/version render from `get_tool_status`.
- Missing tools show red/missing state.
- Auto install button is disabled with clear text.
- New user guide is visible.

- [ ] **Step 5: Check settings page**

Navigate to 参数设置. Verify:

- Preset chips update draft options.
- Save global default calls the store action.
- Error table displays PRD messages.

- [ ] **Step 6: Stop dev server**

Stop the running Vite process with `Ctrl-C`.

- [ ] **Step 7: Commit verification fixes only if needed**

If manual verification required source fixes:

```bash
git add src
git commit -m "fix: resolve frontend verification issues"
```

If no fixes were needed, do not create an empty commit.

## Task 9: Final Verification

**Files:**
- No source files unless verification finds defects.

- [ ] **Step 1: Run tests**

```bash
npm run test
```

Expected: PASS.

- [ ] **Step 2: Run frontend build**

```bash
npm run build
```

Expected: PASS.

- [ ] **Step 3: Inspect git status**

```bash
git status --short
```

Expected: only intentional files changed, no generated build artifacts such as `dist/` staged unless the repo already tracks them.

- [ ] **Step 4: Summarize implementation result**

Include:

- Files changed.
- Tests run and output summary.
- Any UI behavior intentionally left disabled because backend commands are absent.
- Any follow-up needed for real tool auto-install or file picker integration.

## Plan Self-Review

- Spec coverage: covered four-page UI shell, global/default/device/session parameters, persistence commands, device flow, sessions, tools, settings, modals, logs, testing, and validation.
- Red-flag scan: no incomplete marker, incomplete step, or undefined task dependency is intentionally left in the plan.
- Type consistency: frontend uses `ScrcpyOptions`, `PresetId`, `AppConfig`, `SessionInfo`, and `Device` from `src/types/app.ts`; Rust config fields serialize to the same snake_case keys expected by frontend `AppConfig`.
