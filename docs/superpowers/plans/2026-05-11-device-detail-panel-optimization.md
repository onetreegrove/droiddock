# Device Detail Panel Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor the Device Detail Panel so device identity, connection medium, status guidance, and launch actions are easier to understand for non-technical users.

**Architecture:** Split the current top detail area in `DeviceDetailPanel.vue` into `DeviceHero.vue` and `DeviceStatusBanner.vue`. Keep launch behavior owned by `DeviceDetailPanel.vue`: the banner provides status guidance and recovery entry points, while the bottom Launch Bar remains the only "launch mirroring" action.

**Tech Stack:** Vue 3 (Composition API), Pinia, TypeScript, Vanilla CSS, Vitest.

---

## Scope Notes

- Keep the parameter editor, command preview, and session panel behavior unchanged.
- Keep wireless reconnect modal behavior unchanged.
- Preserve wireless IP / endpoint visibility after removing the old `metadata-grid`.
- Do not add a placeholder help modal or browser popup for authorization guidance. The unauthorized banner should use complete inline guidance in this iteration.
- Commit steps are optional checkpoints. If the user has not asked for commits, complete the implementation and verification without committing.

---

### Task 1: Create DeviceHero Component

**Files:**
- Create: `src/components/DeviceHero.vue`

- [ ] **Step 1: Add the focused Hero component**

Create `src/components/DeviceHero.vue` with the full device identity layout. Wireless devices must keep IP / endpoint visible in the secondary row; USB devices can omit the network field.

```vue
<script setup lang="ts">
import { computed } from 'vue';
import type { ManagedDevice } from '../types/app';
import StatusChip from './StatusChip.vue';

const props = defineProps<{
  device: ManagedDevice;
  ipAddress: string | null;
  connectionLabel: string;
}>();

const emit = defineEmits<{
  (e: 'edit-alias'): void;
}>();

const displayName = computed(() => props.device.alias || props.device.display_name || props.device.model || '未知设备');
const networkLabel = computed(() => {
  if (props.device.connection !== 'wireless') return null;
  return props.device.endpoint || props.ipAddress || null;
});
const stateLabel = computed(() => {
  if (props.device.presence === 'offline') return '离线';
  if (props.device.state === 'device') return '可用';
  if (props.device.state === 'unauthorized') return '待授权';
  return '离线';
});
const stateTone = computed(() => {
  if (props.device.presence === 'offline') return 'red';
  if (props.device.state === 'device') return 'green';
  if (props.device.state === 'unauthorized') return 'yellow';
  return 'red';
});
</script>

<template>
  <div class="device-hero-new">
    <div class="hero-icon-box" :title="connectionLabel">
      <svg width="28" height="28" viewBox="0 0 20 20" fill="none" aria-hidden="true">
        <rect x="3.5" y="1.5" width="13" height="17" rx="2.5" stroke="currentColor" stroke-width="1.3" />
        <circle cx="10" cy="15.5" r="1.2" fill="currentColor" />
        <rect x="6.5" y="4" width="7" height="1" rx=".5" fill="currentColor" opacity=".4" />
      </svg>
      <div class="connection-badge" aria-hidden="true">
        <svg v-if="device.connection === 'usb'" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
          <path d="M12 2v20M2 12h20" />
        </svg>
        <svg v-else width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
          <path d="M5 12.55a11 11 0 0 1 14.08 0M1.42 9a16 16 0 0 1 21.16 0M8.59 16.11a6 6 0 0 1 6.82 0M12 20h.01" />
        </svg>
      </div>
    </div>

    <div class="hero-info">
      <div class="hero-primary-row">
        <h1 class="hero-alias" :title="`${displayName}，点击修改别名`" @click="emit('edit-alias')">
          {{ displayName }}
        </h1>
        <button class="btn-icon-sm" title="编辑别名" @click="emit('edit-alias')">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
            <path d="M2.5 11.5v-2l6-6 2 2-6 6h-2zM9.5 2.5l2 2" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          </svg>
        </button>
        <StatusChip :tone="stateTone" :label="stateLabel" dot />
      </div>

      <div class="hero-secondary-row">
        <span class="secondary-item">{{ device.model || '-' }}</span>
        <span class="dot-separator">·</span>
        <span class="secondary-item">{{ connectionLabel }}</span>
        <template v-if="networkLabel">
          <span class="dot-separator">·</span>
          <span class="secondary-item mono">{{ networkLabel }}</span>
        </template>
        <span class="dot-separator">·</span>
        <span class="secondary-item mono">{{ device.serial }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.device-hero-new {
  display: flex;
  align-items: center;
  gap: 20px;
  margin-bottom: 20px;
}

.hero-icon-box {
  width: 56px;
  height: 56px;
  background: var(--bg3);
  border: 1px solid var(--border2);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  flex: 0 0 auto;
  color: var(--acc);
}

.connection-badge {
  position: absolute;
  bottom: -4px;
  right: -4px;
  background: var(--bg5);
  border: 2px solid var(--bg);
  border-radius: 999px;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--t2);
}

.hero-info {
  flex: 1;
  min-width: 0;
}

.hero-primary-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 5px;
  min-width: 0;
}

.hero-alias {
  margin: 0;
  min-width: 0;
  font-size: 22px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.hero-secondary-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px 8px;
  color: var(--t3);
  font-size: 13px;
  line-height: 1.5;
}

.secondary-item {
  min-width: 0;
  max-width: 100%;
  overflow-wrap: anywhere;
}

.dot-separator {
  opacity: 0.5;
}
</style>
```

- [ ] **Step 2: Run targeted validation**

Run:

```bash
npm run build
```

Expected: TypeScript and Vite build pass. If this fails because the component is not yet imported, continue to Task 3 and rerun the same command after integration.

---

### Task 2: Create DeviceStatusBanner Component

**Files:**
- Create: `src/components/DeviceStatusBanner.vue`

- [ ] **Step 1: Add the status guidance component**

Create `src/components/DeviceStatusBanner.vue`. The banner may open the reconnect modal for a wireless offline device with an endpoint, but it must not launch mirroring. The bottom Launch Bar remains responsible for `重连投屏`.

```vue
<script setup lang="ts">
import { computed } from 'vue';
import type { ManagedDevice } from '../types/app';
import { useUiStore } from '../stores/ui';

type BannerTone = 'warn' | 'error';
type BannerAction = 'reconnect' | null;

const props = defineProps<{
  device: ManagedDevice;
}>();

const ui = useUiStore();

const banner = computed<{
  tone: BannerTone;
  title: string;
  message: string;
  action: BannerAction;
} | null>(() => {
  if (props.device.presence === 'offline') {
    if (props.device.connection === 'wireless') {
      return {
        tone: 'error',
        title: '无线设备已离线',
        message: props.device.endpoint
          ? '手机无线调试端口可能已变化。可先重连恢复连接，或使用底部“重连投屏”继续启动。'
          : '缺少保存的无线连接地址，请重新配对或通过 USB 转无线后再启动投屏。',
        action: props.device.endpoint ? 'reconnect' : null,
      };
    }

    return {
      tone: 'error',
      title: 'USB 设备已离线',
      message: '请重新插入 USB 数据线，确认手机已解锁并允许 USB 调试，设备列表会自动刷新。',
      action: null,
    };
  }

  if (props.device.state === 'unauthorized') {
    return {
      tone: 'warn',
      title: '设备待授权',
      message: '请解锁手机，在 USB 调试授权弹窗中勾选“一律允许使用这台电脑进行调试”，然后点击允许。',
      action: null,
    };
  }

  return null;
});

function handleReconnect() {
  if (!props.device.endpoint) return;
  ui.openReconnectModal(props.device.serial, props.device.endpoint, props.device.wireless_source, false);
}
</script>

<template>
  <div v-if="banner" class="status-banner" :class="banner.tone">
    <div class="banner-icon" aria-hidden="true">
      <svg v-if="banner.tone === 'warn'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0zM12 9v4M12 17h.01" />
      </svg>
      <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="8" x2="12" y2="12" />
        <line x1="12" y1="16" x2="12.01" y2="16" />
      </svg>
    </div>
    <div class="banner-content">
      <div class="banner-title">{{ banner.title }}</div>
      <div class="banner-message">{{ banner.message }}</div>
    </div>
    <button v-if="banner.action === 'reconnect'" class="btn btn-ghost compact-button banner-btn" @click="handleReconnect">
      重连
    </button>
  </div>
</template>

<style scoped>
.status-banner {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 16px;
  border-radius: 10px;
  margin-bottom: 18px;
  border: 1px solid transparent;
}

.status-banner.warn {
  background: var(--yellow-d);
  border-color: rgba(251, 191, 36, 0.2);
  color: var(--yellow);
}

.status-banner.error {
  background: var(--red-d);
  border-color: rgba(248, 113, 113, 0.2);
  color: var(--red);
}

.banner-icon {
  padding-top: 2px;
  flex: 0 0 auto;
}

.banner-content {
  flex: 1;
  min-width: 0;
}

.banner-title {
  font-weight: 600;
  margin-bottom: 2px;
}

.banner-message {
  font-size: 13px;
  line-height: 1.5;
  color: var(--t1);
}

.banner-btn {
  flex: 0 0 auto;
  align-self: center;
  color: inherit;
}
</style>
```

- [ ] **Step 2: Run targeted validation**

Run:

```bash
npm run build
```

Expected: TypeScript and Vite build pass. If this fails because the component is not yet imported, continue to Task 3 and rerun the same command after integration.

---

### Task 3: Refactor DeviceDetailPanel

**Files:**
- Modify: `src/components/DeviceDetailPanel.vue`

- [ ] **Step 1: Import the new components**

Update imports and remove the now-unused `StatusChip` import only after the old hero markup is removed.

```ts
import DeviceHero from './DeviceHero.vue';
import DeviceStatusBanner from './DeviceStatusBanner.vue';
```

- [ ] **Step 2: Replace the top hero block**

Replace the old `.device-hero`, `.device-warning`, and `.metadata-grid` markup with the two new components.

```vue
<div class="device-hero-block">
  <DeviceHero
    :device="device"
    :ip-address="ipAddress"
    :connection-label="connectionLabel"
    @edit-alias="handleEditAlias"
  />
  <DeviceStatusBanner :device="device" />
</div>
```

- [ ] **Step 3: Keep Launch Bar as the launch owner**

Keep the existing `canReconnectAndLaunch`, `launchButtonText`, `launchHint`, and `launch()` behavior. The expected behavior remains:

- USB online and authorized: bottom button says `启动投屏`.
- Wireless online and authorized: bottom button says `启动投屏`.
- Wireless offline with `endpoint` and tools ready: bottom button says `重连投屏` and opens reconnect modal with `launchAfterConnect = true`.
- Wireless offline with `endpoint`: banner button says `重连` and opens reconnect modal with `launchAfterConnect = false`.
- Wireless offline without `endpoint`: no banner button; bottom button disabled.
- Unauthorized: bottom button disabled with authorization title; banner shows inline phone-side authorization guidance.

- [ ] **Step 4: Run automated validation**

Run:

```bash
npm run test
npm run build
```

Expected: both commands pass.

- [ ] **Step 5: Manual UI verification**

Verify these states in the app or a local browser preview:

1. USB online device: Hero shows device name, USB badge, status `可用`, and no banner.
2. Wireless online device: Hero shows wireless badge plus IP or endpoint, status `可用`, and no banner.
3. Unauthorized USB device: Hero shows status `待授权`, banner tells the user to check the phone authorization popup, bottom button disabled.
4. Wireless offline with endpoint: Hero keeps endpoint visible, banner button is `重连`, bottom button is `重连投屏`.
5. Wireless offline without endpoint: banner has no action button, bottom button disabled with clear recovery hint.
6. Long alias / long serial / narrow detail panel: alias truncates cleanly, secondary row wraps, status chip and edit button remain visible.

---

### Task 4: Clean Up CSS

**Files:**
- Modify: `src/styles.css`

- [ ] **Step 1: Remove unused legacy styles**

After Task 3 passes, remove only the selectors no longer referenced by Vue templates:

```css
.device-hero
.hero-icon
.device-hero-info
.hero-title-row
.hero-title
.hero-chips
.metadata-grid
.metadata-grid.compact
.metadata-grid span:nth-child(odd)
.device-warning
```

Keep shared selectors still used elsewhere, especially `.btn-icon-sm`, `.launch-bar`, `.launch-button`, `.hint-text`, `.detail-section`, and CSS variables.

- [ ] **Step 2: Run final validation**

Run:

```bash
npm run test
npm run build
```

Expected: both commands pass.

- [ ] **Step 3: Optional final commit**

Only commit if the user explicitly asks for a commit or this task is being executed in a commit-based workflow.

```bash
git add src/components/DeviceHero.vue src/components/DeviceStatusBanner.vue src/components/DeviceDetailPanel.vue src/styles.css
git commit -m "refactor: optimize device detail panel guidance"
```

---

## Acceptance Criteria

- Device identity is the strongest visual element in the detail panel.
- Users can distinguish USB vs wireless within one second from the badge and text.
- Wireless IP / endpoint remains visible after removing the old metadata grid.
- Unauthorized and offline states explain what happened and what the user should do next.
- Banner `重连` restores wireless connection only; Launch Bar `重连投屏` reconnects and launches.
- Long names, long serials, and narrow panels do not cause overlapping UI.
- `npm run test` and `npm run build` pass after integration and CSS cleanup.

## Self-Review

- [x] No placeholder help modal, browser popup, or unresolved future behavior.
- [x] Banner and Launch Bar responsibilities are explicit.
- [x] Wireless IP / endpoint preservation is covered.
- [x] Automated and manual validation are both specified.
- [x] Commit steps are optional and do not override the user's workflow.
