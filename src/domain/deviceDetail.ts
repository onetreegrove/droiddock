import type { ManagedDevice } from '../types/app';

export type DetailBanner = {
  tone: 'warn' | 'error';
  title: string;
  message: string;
};

export type LaunchAvailability = {
  canLaunch: boolean;
  buttonText: string;
  title: string;
  hint: string;
};

export function detailSecondaryItems(device: ManagedDevice, connectionLabel: string, ipAddress: string | null = null): string[] {
  const items = [device.model || '-', connectionLabel];
  const networkLabel = device.connection === 'wireless' ? device.endpoint || ipAddress : null;

  if (networkLabel) {
    items.push(networkLabel);
  }
  if (device.serial !== networkLabel) {
    items.push(device.serial);
  }

  return items;
}

export function statusBanner(device: ManagedDevice): DetailBanner | null {
  if (device.state === 'unauthorized') {
    return {
      tone: 'warn',
      title: '设备待授权',
      message: '请解锁手机，在 USB 调试授权弹窗中勾选“一律允许使用这台电脑进行调试”，然后点击允许。',
    };
  }

  return null;
}

export function canRestoreConnection(device: ManagedDevice | null, toolsReady: boolean): boolean {
  return Boolean(
    device?.connection === 'wireless' &&
      device.endpoint &&
      toolsReady &&
      (device.presence === 'offline' || device.state === 'offline'),
  );
}

export function launchAvailability(device: ManagedDevice | null, isMirroring: boolean, toolsReady: boolean): LaunchAvailability {
  const canLaunch = Boolean(
    !isMirroring &&
      device?.presence === 'online' &&
      device.state === 'device' &&
      toolsReady,
  );

  let hint = '';
  if (!device) hint = '请选择设备';
  else if (!toolsReady) hint = '请先完成工具配置';
  else if (device.presence === 'offline') {
    hint = device.connection === 'wireless' ? '请先恢复无线连接' : '设备当前不在线，插入 USB 后会自动刷新';
  } else if (device.state === 'unauthorized') {
    hint = '请先在手机上允许 USB 调试授权';
  } else if (device.state === 'offline') {
    hint = '设备连接异常，请重新连接';
  }

  return {
    canLaunch,
    buttonText: '启动投屏',
    title: canLaunch ? '已连接' : '暂不能启动',
    hint,
  };
}

export function canRelaunchSession(device: ManagedDevice, toolsReady = true): boolean {
  if (!toolsReady) return false;
  if (device.connection === 'wireless' && device.presence === 'offline') {
    return false;
  }
  return device.presence === 'online' && device.state === 'device';
}
