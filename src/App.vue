<script setup lang="ts">
import { computed, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

type ToolStatus = {
  adb_path: string | null;
  scrcpy_path: string | null;
  adb_version: string | null;
  scrcpy_version: string | null;
};

type Device = {
  serial: string;
  state: string;
  model: string | null;
  product: string | null;
  connection: 'usb' | 'wireless';
  raw: string;
};

const toolStatus = ref<ToolStatus | null>(null);
const devices = ref<Device[]>([]);
const selectedSerial = ref('');
const logLines = ref<string[]>([]);
const loading = ref(false);

const selectedDevice = computed(() => devices.value.find((device) => device.serial === selectedSerial.value));

function log(message: string) {
  const time = new Date().toLocaleTimeString();
  logLines.value.unshift(`[${time}] ${message}`);
}

async function loadTools() {
  loading.value = true;
  try {
    toolStatus.value = await invoke<ToolStatus>('get_tool_status');
    log('工具状态检查完成');
  } catch (error) {
    log(`工具状态检查失败：${String(error)}`);
  } finally {
    loading.value = false;
  }
}

async function refreshDevices() {
  loading.value = true;
  try {
    devices.value = await invoke<Device[]>('list_devices');
    if (!selectedSerial.value && devices.value.length > 0) {
      selectedSerial.value = devices.value[0].serial;
    }
    log(`发现 ${devices.value.length} 台设备`);
  } catch (error) {
    log(`设备刷新失败：${String(error)}`);
  } finally {
    loading.value = false;
  }
}

async function startMirror() {
  if (!selectedSerial.value) {
    log('请先选择一台设备');
    return;
  }

  try {
    await invoke('start_scrcpy', {
      serial: selectedSerial.value,
      options: {
        maxSize: 1920,
        maxFps: 60,
        noAudio: true,
        stayAwake: true
      }
    });
    log(`已启动投屏：${selectedSerial.value}`);
  } catch (error) {
    log(`启动投屏失败：${String(error)}`);
  }
}

loadTools();
</script>

<template>
  <main class="app-shell">
    <aside class="sidebar">
      <div class="brand">
        <span class="brand-mark">D</span>
        <div>
          <h1>DroidDock</h1>
          <p>Android 投屏控制台</p>
        </div>
      </div>

      <nav class="nav">
        <button class="active">首页</button>
        <button>设备</button>
        <button>连接向导</button>
        <button>设置</button>
      </nav>

      <section class="tool-card">
        <div class="tool-row">
          <span>adb</span>
          <strong :class="toolStatus?.adb_path ? 'ok' : 'warn'">
            {{ toolStatus?.adb_version || '未检测' }}
          </strong>
        </div>
        <p>{{ toolStatus?.adb_path || '需要安装或配置 adb 路径' }}</p>
        <div class="tool-row">
          <span>scrcpy</span>
          <strong :class="toolStatus?.scrcpy_path ? 'ok' : 'warn'">
            {{ toolStatus?.scrcpy_version || '未检测' }}
          </strong>
        </div>
        <p>{{ toolStatus?.scrcpy_path || '需要安装或配置 scrcpy 路径' }}</p>
      </section>
    </aside>

    <section class="workspace">
      <header class="topbar">
        <div>
          <h2>设备控制台</h2>
          <p>第一版使用 scrcpy 独立窗口投屏，DroidDock 负责连接和会话控制。</p>
        </div>
        <div class="actions">
          <button @click="loadTools" :disabled="loading">检查工具</button>
          <button class="primary" @click="refreshDevices" :disabled="loading">刷新设备</button>
        </div>
      </header>

      <section class="quick-grid">
        <article>
          <strong>USB 连接</strong>
          <p>插入数据线，允许 USB 调试后即可投屏。</p>
        </article>
        <article>
          <strong>无线连接</strong>
          <p>先通过 USB 切换 TCP/IP，再拔线继续使用。</p>
        </article>
        <article>
          <strong>ADB Pair</strong>
          <p>Android 11+ 使用无线调试配对码连接。</p>
        </article>
      </section>

      <section class="content-grid">
        <section class="panel">
          <div class="panel-head">
            <h3>设备列表</h3>
            <span>{{ devices.length }} 台</span>
          </div>
          <div class="device-list">
            <button
              v-for="device in devices"
              :key="device.serial"
              :class="['device-item', { selected: selectedSerial === device.serial }]"
              @click="selectedSerial = device.serial"
            >
              <strong>{{ device.model || device.serial }}</strong>
              <span>{{ device.serial }} · {{ device.connection }} · {{ device.state }}</span>
            </button>
            <p v-if="devices.length === 0" class="empty">暂未发现设备</p>
          </div>
        </section>

        <section class="panel">
          <div class="panel-head">
            <h3>投屏会话</h3>
            <span>{{ selectedDevice?.state || '未选择' }}</span>
          </div>
          <div class="session-card">
            <strong>{{ selectedDevice?.model || selectedDevice?.serial || '请选择设备' }}</strong>
            <p>默认参数：1920 / 60fps / 禁用音频 / 保持亮屏。</p>
            <button class="primary" @click="startMirror">启动投屏</button>
          </div>
          <pre class="log">{{ logLines.join('\n') }}</pre>
        </section>
      </section>
    </section>
  </main>
</template>

