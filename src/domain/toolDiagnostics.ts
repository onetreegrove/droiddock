import type { ToolDiagnostic, ToolHealth, ToolSource } from '../lib/ipc/types';

export type ToolHealthTone = 'green' | 'yellow' | 'red' | 'blue' | 'gray';

export function toolSourceLabel(source: ToolSource | null): string {
  const labels: Record<ToolSource, string> = {
    configured: '手动配置',
    bundled: 'DroidDock 管理',
    android_sdk: 'Android SDK',
    local_bin: '本地用户目录',
    homebrew: 'Homebrew',
    system_path: '系统 PATH',
  };

  return source ? labels[source] : '未找到来源';
}

export function toolHealthTone(diagnostic: Pick<ToolDiagnostic, 'health'>): ToolHealthTone {
  const redHealth: ToolHealth[] = ['missing', 'incompatible_arch'];
  const yellowHealth: ToolHealth[] = ['not_executable', 'wrong_tool', 'version_failed'];

  if (diagnostic.health === 'ready') return 'green';
  if (redHealth.includes(diagnostic.health)) return 'red';
  if (yellowHealth.includes(diagnostic.health)) return 'yellow';
  return 'gray';
}

export function toolActionLabel(diagnostic: Pick<ToolDiagnostic, 'health' | 'path'>): string {
  if (diagnostic.health === 'incompatible_arch' && diagnostic.path === null) {
    return '当前 Mac 不受支持';
  }

  const labels: Record<ToolHealth, string> = {
    ready: '重新检测',
    missing: '自动安装或手动选择',
    not_executable: '选择可执行文件',
    wrong_tool: '重新选择正确工具',
    version_failed: '重新选择或重新安装',
    incompatible_arch: '更换 Apple Silicon 版本',
  };

  return labels[diagnostic.health];
}

export function toolSummary(diagnostic: ToolDiagnostic): string {
  if (diagnostic.health === 'ready') {
    return `${diagnostic.kind} 可用，来源：${toolSourceLabel(diagnostic.source)}`;
  }

  return diagnostic.message;
}
