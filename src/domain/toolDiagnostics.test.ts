import { describe, expect, it } from 'vitest';
import type { ToolDiagnostic } from '../lib/ipc/types';
import { toolActionLabel, toolHealthTone, toolSourceLabel, toolSummary } from './toolDiagnostics';

function diagnostic(overrides: Partial<ToolDiagnostic>): ToolDiagnostic {
  return {
    kind: 'adb',
    path: null,
    source: null,
    version: null,
    arch: null,
    health: 'missing',
    message: '未找到 adb',
    ...overrides,
  };
}

describe('toolDiagnostics', () => {
  it('maps ready bundled diagnostics to green and DroidDock managed labels', () => {
    const item = diagnostic({
      path: '/app/tools/adb',
      source: 'bundled',
      version: 'Android Debug Bridge version 1.0.41',
      arch: 'arm64',
      health: 'ready',
      message: 'adb 可用',
    });

    expect(toolHealthTone(item)).toBe('green');
    expect(toolSourceLabel(item.source)).toBe('DroidDock 管理');
    expect(toolSummary(item)).toContain('可用');
  });

  it('maps missing diagnostics to red and install or manual selection action', () => {
    const item = diagnostic({ health: 'missing', message: '未找到 adb' });

    expect(toolHealthTone(item)).toBe('red');
    expect(toolActionLabel(item)).toBe('自动安装或手动选择');
  });

  it('maps unsupported host diagnostics to an unsupported Mac action', () => {
    const item = diagnostic({
      health: 'incompatible_arch',
      path: null,
      arch: 'x86_64',
      message: '当前版本仅支持 Apple Silicon Mac，暂不支持 Intel Mac',
    });

    expect(toolActionLabel(item)).toBe('当前 Mac 不受支持');
    expect(toolSummary(item)).toContain('不支持 Intel Mac');
  });

  it('maps incompatible tool binaries to Apple Silicon replacement action', () => {
    const item = diagnostic({
      health: 'incompatible_arch',
      path: '/usr/local/bin/adb',
      arch: 'x86_64',
      message: '当前工具不是 Apple Silicon 版本',
    });

    expect(toolActionLabel(item)).toBe('更换 Apple Silicon 版本');
  });
});
