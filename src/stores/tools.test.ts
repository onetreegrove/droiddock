import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useToolsStore } from './tools';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

vi.mock('../lib/ipc/client', () => ({
  invokeCommand: vi.fn(),
}));

describe('tools store install progress', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('deduplicates repeated adjacent install log lines', () => {
    const tools = useToolsStore();

    tools.appendInstallLog('准备工具安装目录');
    tools.appendInstallLog('准备工具安装目录');
    tools.appendInstallLog('查询 scrcpy 最新 macOS Apple Silicon 版本');

    expect(tools.installLogs).toEqual([
      '准备工具安装目录',
      '查询 scrcpy 最新 macOS Apple Silicon 版本',
    ]);
  });

  it('marks install as failed and keeps the failure reason', () => {
    const tools = useToolsStore();

    tools.resetInstallLogs('scrcpy');
    tools.markInstallFailed('curl: (56) The requested URL returned error: 403');
    tools.markInstallFailed('curl: (56) The requested URL returned error: 403');

    expect(tools.installStatus).toBe('failed');
    expect(tools.installError).toBe('curl: (56) The requested URL returned error: 403');
    expect(tools.installLogs).toEqual(['curl: (56) The requested URL returned error: 403']);
  });
});
