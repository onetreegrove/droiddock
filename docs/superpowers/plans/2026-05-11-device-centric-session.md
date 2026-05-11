# Device-Centric Session Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move DroidDock's primary session controls into the selected device detail view while keeping the sessions page as a multi-device overview.

**Architecture:** Keep the existing Tauri session backend unchanged. Add frontend session selection helpers in Pinia, surface session state in the device list, introduce a device-scoped session panel, and retarget the sessions page to overview and navigation duties.

**Tech Stack:** Vue 3, TypeScript, Pinia, Tauri invoke, existing Vitest suite.

---

## File Structure

- Modify: `src/stores/sessions.ts`
  - Add `latestSession(serial)` and `displaySession(serial)` getters.
- Modify: `src/stores/app.ts`
  - Stop navigating to the sessions page after `startMirror`.
  - Add a small `showDevice(serial)` action to open the devices page and select the device.
- Modify: `src/stores/ui.ts`
  - Add `showDevice(serial)` or equivalent action if UI navigation belongs there.
- Modify: `src/components/DeviceList.vue`
  - Add session status badge and visual state for devices with running sessions.
- Create: `src/components/DeviceSessionPanel.vue`
  - Show the selected device's current or latest session with actions.
  - Auto-expand logs for failed sessions and show unauthorized guidance in the empty state.
- Modify: `src/components/DeviceDetailPanel.vue`
  - Render `DeviceSessionPanel`.
  - Keep launch flow on the devices page.
- Modify: `src/components/SessionCard.vue`
  - Add an optional `show-device-action` prop and emit / call action for viewing the device.
- Modify: `src/components/SessionsView.vue`
  - Rename page copy to overview positioning.
  - Pass the view-device action to each session card.
- Modify: `src/components/AppSidebar.vue`
  - Rename navigation label from `投屏会话` to `投屏总览`.
- Modify: `src/styles.css`
  - Add compact styles for device session panel and session badges.
- Test: `src/stores/sessions.test.ts` if the current test setup can exercise Pinia getters cleanly.
  - If not, validate through `npm run test` and `npm run build` after the component changes.

## Task 1: Add Session Selection Helpers

**Files:**
- Modify: `src/stores/sessions.ts`

- [ ] **Step 1: Add getters for latest and display session**

Update the getters block in `src/stores/sessions.ts`:

```ts
  getters: {
    activeSession: (state) => (serial: string) =>
      state.sessions.find((session) => session.serial === serial && session.status === 'running') ?? null,
    latestSession: (state) => (serial: string) =>
      [...state.sessions]
        .filter((session) => session.serial === serial)
        .sort((left, right) => right.started_at - left.started_at)[0] ?? null,
    displaySession(): (serial: string) => SessionInfo | null {
      return (serial: string) => this.activeSession(serial) ?? this.latestSession(serial);
    },
  },
```

- [ ] **Step 2: Run frontend tests**

Run:

```bash
npm run test
```

Expected: existing Vitest tests pass.

## Task 2: Keep Launch Flow on Device Detail

**Files:**
- Modify: `src/stores/app.ts`
- Modify: `src/stores/ui.ts` if navigation helper is placed there

- [ ] **Step 1: Stop automatic sessions-page navigation after launch**

In `src/stores/app.ts`, update `startMirror` so it no longer calls `ui.openPage('sessions')`.

Expected shape:

```ts
  async function startMirror(serial: string, options?: ScrcpyOptions) {
    setBusy('sessions', true);
    try {
      const finalOptions = options ?? effectiveOptions(serial);
      const info = await sessionsStore.startMirror(serial, finalOptions);
      log(`已启动投屏: ${serial}`);
      return info;
    } catch (error) {
      log(`启动投屏失败: ${errorMessage(error)}`);
      throw error;
    } finally {
      setBusy('sessions', false);
    }
  }
```

- [ ] **Step 2: Add device navigation helper**

Prefer adding this to `src/stores/ui.ts`:

```ts
    showDevice(serial: string) {
      this.selectedSerial = serial;
      this.currentPage = 'devices';
    },
```

If `src/stores/ui.ts` already has a method with this responsibility, reuse it instead of adding a duplicate.

- [ ] **Step 3: Run type check through build**

Run:

```bash
npm run build
```

Expected: TypeScript build passes.

## Task 3: Add Device List Mirroring State

**Files:**
- Modify: `src/components/DeviceList.vue`
- Modify: `src/styles.css`

- [ ] **Step 1: Add helper functions in `DeviceList.vue`**

Add these helpers in the `<script setup>` block:

```ts
function mirrorSession(device: ManagedDevice) {
  return store.displaySession(device.serial);
}

function mirrorLabel(device: ManagedDevice) {
  const session = mirrorSession(device);
  if (!session) return '';
  if (session.status === 'running') return '投屏中';
  if (session.status === 'failed') return '投屏失败';
  if (session.status === 'stopped') return '最近停止';
  return '';
}

function mirrorTone(device: ManagedDevice): 'green' | 'yellow' | 'red' | 'gray' {
  const session = mirrorSession(device);
  if (session?.status === 'running') return 'green';
  if (session?.status === 'failed') return 'red';
  if (session?.status === 'stopped') return 'gray';
  return 'gray';
}
```

This assumes `useAppStore()` exposes `displaySession`. If not, expose it from `src/stores/app.ts` in the return object:

```ts
  const displaySession = (serial: string) => sessionsStore.displaySession(serial);
```

- [ ] **Step 2: Render the badge in each device card**

Inside the existing `.device-chips` block, after connection status:

```vue
            <StatusChip
              v-if="mirrorLabel(device)"
              :tone="mirrorTone(device)"
              :label="mirrorLabel(device)"
              dot
            />
```

Add a running class to the device card:

```vue
        :class="[
          'device-card',
          {
            selected: ui.selectedSerial === device.serial,
            mirroring: mirrorSession(device)?.status === 'running',
          },
        ]"
```

- [ ] **Step 3: Add CSS for running device cards**

Append focused styles to `src/styles.css` near existing `.device-card` rules:

```css
.device-card.mirroring {
  border-color: rgba(48, 209, 88, 0.42);
}

.device-card.mirroring .device-icon {
  color: #30d158;
}
```

- [ ] **Step 4: Run build**

Run:

```bash
npm run build
```

Expected: Vue template type checking passes.

## Task 4: Create Device Session Panel

**Files:**
- Create: `src/components/DeviceSessionPanel.vue`
- Modify: `src/styles.css`

- [ ] **Step 1: Create `DeviceSessionPanel.vue`**

Create `src/components/DeviceSessionPanel.vue`:

```vue
<script setup lang="ts">
import { computed, watch } from 'vue';
import LogPanel from './LogPanel.vue';
import StatusChip from './StatusChip.vue';
import { optionSummaryTagsFromArgs } from '../domain/scrcpyOptions';
import type { ManagedDevice, SessionInfo } from '../types/app';
import { useAppStore } from '../stores/app';
import { useUiStore } from '../stores/ui';

const props = defineProps<{
  device: ManagedDevice;
  session: SessionInfo | null;
}>();

const store = useAppStore();
const ui = useUiStore();

const running = computed(() => props.session?.status === 'running');
const failed = computed(() => props.session?.status === 'failed');
const tags = computed(() => (props.session ? optionSummaryTagsFromArgs(props.session.args) : []));
const unauthorized = computed(() => props.device.state === 'unauthorized');
const title = computed(() => {
  if (!props.session) return '当前未投屏';
  if (running.value) return '当前会话';
  if (failed.value) return '最近投屏失败';
  return '最近投屏已停止';
});
const statusLabel = computed(() => {
  if (!props.session) return '未投屏';
  if (running.value) return '投屏中';
  if (failed.value) return '失败';
  return '已停止';
});
const statusTone = computed(() => {
  if (running.value) return 'green';
  if (failed.value) return 'red';
  return 'gray';
});
const emptyMessage = computed(() => {
  if (unauthorized.value) return '请在手机上允许 USB 调试授权，授权后再启动投屏。';
  return '这台设备当前没有投屏会话。调整参数后可直接启动投屏。';
});

watch(
  () => props.session,
  async (session) => {
    if (session?.status === 'failed' && ui.selectedLogSessionId !== session.session_id) {
      await store.openSessionLogs(session.session_id);
    }
  },
  { immediate: true },
);

async function relaunch() {
  if (props.device.connection === 'wireless' && props.device.presence === 'offline' && props.device.endpoint) {
    ui.openReconnectModal(props.device.serial, props.device.endpoint, props.device.wireless_source, true);
    return;
  }
  await store.startMirror(props.device.serial);
}
</script>

<template>
  <section class="detail-section device-session-panel">
    <div class="section-head">
      <div>
        <div class="section-title">{{ title }}</div>
        <StatusChip :tone="statusTone" :label="statusLabel" dot />
      </div>
      <div v-if="session" class="inline-actions">
        <button
          v-if="running"
          class="btn btn-danger compact-button"
          :disabled="store.busy.sessions"
          @click="store.stopMirror(session.session_id)"
        >
          停止投屏
        </button>
        <button
          v-else
          class="btn btn-ghost compact-button"
          :disabled="store.busy.sessions || (device.presence === 'offline' && device.connection === 'usb')"
          @click="relaunch"
        >
          {{ device.connection === 'wireless' && device.presence === 'offline' ? '重连投屏' : '重新投屏' }}
        </button>
        <button class="btn btn-ghost compact-button" @click="store.openSessionLogs(session.session_id)">
          {{ ui.selectedLogSessionId === session.session_id ? '收起日志' : '查看日志' }}
        </button>
      </div>
    </div>

    <div v-if="session" class="device-session-body">
      <div class="metadata-grid compact">
        <span>Serial</span><span class="mono">{{ session.serial }}</span>
        <span>PID</span><span class="mono">{{ session.pid }}</span>
        <span>连接</span><span>{{ session.connection === 'usb' ? 'USB' : '无线' }}</span>
      </div>
      <div v-if="session.last_message" :class="['session-message', { error: failed }]">
        {{ session.last_message }}
      </div>
      <div class="tag-row">
        <span v-for="tag in tags" :key="tag" class="param-tag mono">{{ tag }}</span>
      </div>
      <LogPanel
        v-if="ui.selectedLogSessionId === session.session_id"
        :lines="store.sessionLogs[session.session_id] ?? []"
      />
    </div>

    <div v-else class="empty-panel compact-empty">
      {{ emptyMessage }}
    </div>
  </section>
</template>
```

- [ ] **Step 2: Add minimal styles**

Add to `src/styles.css`:

```css
.device-session-panel {
  margin-bottom: 12px;
}

.device-session-body {
  display: grid;
  gap: 10px;
}

.metadata-grid.compact {
  grid-template-columns: max-content minmax(0, 1fr);
}

.compact-empty {
  padding: 12px;
}

.session-message.error {
  border-color: rgba(255, 69, 58, 0.38);
  color: #ffb4ad;
}
```

- [ ] **Step 3: Run build**
- [ ] **Step 3: Verify failure and unauthorized behaviors in the component code**

Confirm `src/components/DeviceSessionPanel.vue` includes:

```ts
watch(
  () => props.session,
  async (session) => {
    if (session?.status === 'failed' && ui.selectedLogSessionId !== session.session_id) {
      await store.openSessionLogs(session.session_id);
    }
  },
  { immediate: true },
);
```

Confirm the empty state uses:

```ts
const emptyMessage = computed(() => {
  if (unauthorized.value) return '请在手机上允许 USB 调试授权，授权后再启动投屏。';
  return '这台设备当前没有投屏会话。调整参数后可直接启动投屏。';
});
```

- [ ] **Step 4: Run build**

Run:

```bash
npm run build
```

Expected: new component compiles.

## Task 5: Embed Session Panel in Device Detail

**Files:**
- Modify: `src/components/DeviceDetailPanel.vue`
- Modify: `src/stores/app.ts`

- [ ] **Step 1: Expose `displaySession` from app store**

In `src/stores/app.ts`, add:

```ts
  const displaySession = (serial: string) => sessionsStore.displaySession(serial);
```

Return it with the other public helpers:

```ts
    displaySession,
```

- [ ] **Step 2: Import and compute the selected device session**

In `src/components/DeviceDetailPanel.vue`, import the panel:

```ts
import DeviceSessionPanel from './DeviceSessionPanel.vue';
```

Add computed state:

```ts
const displaySession = computed(() => (device.value ? store.displaySession(device.value.serial) : null));
const isMirroring = computed(() => displaySession.value?.status === 'running');
```

- [ ] **Step 3: Render the panel below device metadata**

In the template, after the device hero block and before the parameter section:

```vue
    <DeviceSessionPanel :device="device" :session="displaySession" />
```

- [ ] **Step 4: Avoid duplicate primary launch action while running**

Update `canLaunch` to return false when `isMirroring` is true:

```ts
const canLaunch = computed(
  () =>
    !isMirroring.value &&
    (Boolean(device.value?.presence === 'online' && device.value?.state === 'device' && store.isToolsReady) ||
      canReconnectAndLaunch.value),
);
```

- [ ] **Step 5: Run tests and build**

Run:

```bash
npm run test
npm run build
```

Expected: tests and build pass.

## Task 6: Retarget Sessions Page to Overview

**Files:**
- Modify: `src/components/AppSidebar.vue`
- Modify: `src/components/SessionsView.vue`
- Modify: `src/components/SessionCard.vue`
- Modify: `src/stores/app.ts` or `src/stores/ui.ts`

- [ ] **Step 1: Rename sidebar label**

In `src/components/AppSidebar.vue`, change:

```vue
        <span>投屏会话</span>
```

to:

```vue
        <span>投屏总览</span>
```

- [ ] **Step 2: Add view-device action to `SessionCard.vue`**

Update props:

```ts
const props = withDefaults(defineProps<{ session: SessionInfo; showDeviceAction?: boolean }>(), {
  showDeviceAction: false,
});
```

Add a button in `.session-actions`:

```vue
        <button
          v-if="showDeviceAction"
          class="btn btn-ghost compact-button"
          @click="ui.showDevice(session.serial)"
        >
          查看设备
        </button>
```

If `showDevice` is implemented on `appStore` instead of `uiStore`, call that store action consistently.

- [ ] **Step 3: Update sessions page title and card usage**

In `src/components/SessionsView.vue`, change header text:

```vue
    <AppHeader title="投屏总览" :subtitle="`${store.sessions.length} 个会话 · ${runningCount} 个运行中`">
```

Pass the new prop:

```vue
        <SessionCard :session="session" show-device-action />
```

Update empty state:

```vue
      <div v-if="store.sessions.length === 0" class="empty-panel">暂无投屏会话，请到设备页选择设备并启动投屏。</div>
```

- [ ] **Step 4: Run build**

Run:

```bash
npm run build
```

Expected: all Vue components compile.

## Task 7: Final Verification

**Files:**
- No code changes expected unless verification finds a defect.

- [ ] **Step 1: Run frontend test suite**

Run:

```bash
npm run test
```

Expected: all tests pass.

- [ ] **Step 2: Run production build**

Run:

```bash
npm run build
```

Expected: TypeScript and Vite build pass.

- [ ] **Step 3: Optional local UI smoke check**

If a dev server is needed, run:

```bash
npm run dev -- --host 127.0.0.1
```

Open:

```text
http://127.0.0.1:1420/
```

Manual checks:

- Device list shows `投屏中` for a running session.
- Launching from device detail keeps the app on the devices page.
- Device detail shows current session controls and logs.
- Failed sessions auto-expand logs in the device detail panel.
- Unauthorized devices with no session show the phone authorization hint.
- Sessions overview still supports stop all.
- `查看设备` returns to the selected device.

- [ ] **Step 4: Review changed files**

Run:

```bash
git diff --stat
git diff -- src/stores/sessions.ts src/stores/app.ts src/stores/ui.ts src/components/DeviceList.vue src/components/DeviceDetailPanel.vue src/components/DeviceSessionPanel.vue src/components/SessionCard.vue src/components/SessionsView.vue src/components/AppSidebar.vue src/styles.css
```

Expected: changes are limited to frontend state, components, and styles. No Rust backend changes should appear.

## Self-Review Checklist

- Spec coverage:
  - Device list mirroring status: Task 3.
  - Device detail current session panel: Tasks 4 and 5.
  - Launch stays on device detail: Task 2.
  - Sessions page becomes overview: Task 6.
  - No backend session refactor: all tasks avoid `src-tauri`.

- Placeholder scan:
  - No placeholder keywords or vague implementation-only steps remain.

- Type consistency:
  - `displaySession(serial)` returns `SessionInfo | null`.
  - `DeviceSessionPanel` receives `ManagedDevice` and `SessionInfo | null`.
  - UI navigation uses `showDevice(serial)` consistently once implemented.
