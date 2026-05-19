import type { ToolInstallStatus, ToolInstallTarget } from '../lib/ipc/types';

function targetLabel(target: ToolInstallTarget): string {
  if (target === 'adb') return 'adb';
  if (target === 'scrcpy') return 'scrcpy';
  return 'adb 和 scrcpy';
}

export function installProgressLines(isInstalling: boolean, logs: string[], target: ToolInstallTarget): string[] {
  if (logs.length > 0) return logs;

  if (!isInstalling) return [];

  return [
    '正在准备工具安装，请保持网络连接。',
    `接下来会下载 ${targetLabel(target)}，并自动校验、解压到 DroidDock 工具目录。`,
  ];
}

export function shouldShowInstallLogDialog(isDialogOpen: boolean, isInstalling: boolean, logs: string[]): boolean {
  return isDialogOpen && (isInstalling || logs.length > 0);
}

export function installSummary(status: ToolInstallStatus, error: string | null, logCount: number) {
  if (status === 'failed') {
    return {
      tone: 'failed' as const,
      title: '安装失败',
      detail: error || '请查看日志中的失败原因',
    };
  }

  if (status === 'success') {
    return {
      tone: 'success' as const,
      title: '安装完成',
      detail: `${logCount} 条记录`,
    };
  }

  return {
    tone: 'running' as const,
    title: '安装进行中',
    detail: `${logCount} 条记录`,
  };
}
