import { describe, expect, it } from 'vitest';
import { installProgressLines, installSummary, shouldShowInstallLogDialog } from './installProgress';

describe('installProgressLines', () => {
  it('shows target-specific guidance while installing adb', () => {
    expect(installProgressLines(true, [], 'adb')).toEqual([
      '正在准备工具安装，请保持网络连接。',
      '接下来会下载 adb，并自动校验、解压到 DroidDock 工具目录。',
    ]);
  });

  it('shows target-specific guidance while installing all tools', () => {
    expect(installProgressLines(true, [], 'all')).toEqual([
      '正在准备工具安装，请保持网络连接。',
      '接下来会下载 adb 和 scrcpy，并自动校验、解压到 DroidDock 工具目录。',
    ]);
  });

  it('keeps backend logs after installation returns', () => {
    expect(installProgressLines(false, ['工具安装完成'], 'all')).toEqual(['工具安装完成']);
  });

  it('shows the install log dialog while installing or when logs exist', () => {
    expect(shouldShowInstallLogDialog(true, true, [])).toBe(true);
    expect(shouldShowInstallLogDialog(true, false, ['工具安装完成'])).toBe(true);
    expect(shouldShowInstallLogDialog(false, true, ['下载 adb'])).toBe(false);
    expect(shouldShowInstallLogDialog(true, false, [])).toBe(false);
  });

  it('summarizes install state with success and failure details', () => {
    expect(installSummary('running', null, 2)).toEqual({
      tone: 'running',
      title: '安装进行中',
      detail: '2 条记录',
    });
    expect(installSummary('success', null, 5)).toEqual({
      tone: 'success',
      title: '安装完成',
      detail: '5 条记录',
    });
    expect(installSummary('failed', 'curl: (56) The requested URL returned error: 403', 3)).toEqual({
      tone: 'failed',
      title: '安装失败',
      detail: 'curl: (56) The requested URL returned error: 403',
    });
  });
});
