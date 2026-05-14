import type { ManagedDevice } from '../types/app';

export type DetailBanner = {
  tone: 'warn' | 'error';
  title: string;
  message: string;
  action: 'reconnect' | null;
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
  if (device.presence === 'offline') {
    if (device.connection === 'wireless') {
      return {
        tone: 'error',
        title: '无线设备已离线',
        message: device.endpoint
          ? '手机无线调试端口可能已变化。可先只恢复连接，或使用底部“重连投屏”继续启动。'
          : '缺少保存的无线连接地址，请重新配对或通过 USB 转无线后再启动投屏。',
        action: device.endpoint ? 'reconnect' : null,
      };
    }

    return {
      tone: 'error',
      title: 'USB 设备已离线',
      message: '请重新插入 USB 数据线，确认手机已解锁并允许 USB 调试，设备列表会自动刷新。',
      action: null,
    };
  }

  if (device.state === 'unauthorized') {
    return {
      tone: 'warn',
      title: '设备待授权',
      message: '请解锁手机，在 USB 调试授权弹窗中勾选“一律允许使用这台电脑进行调试”，然后点击允许。',
      action: null,
    };
  }

  if (device.state === 'offline') {
    return {
      tone: 'error',
      title: '设备连接异常',
      message:
        device.connection === 'wireless' && device.endpoint
          ? '设备仍在列表中，但 adb 显示连接异常。可先只恢复连接，或使用底部“重连投屏”继续启动。'
          : '设备仍在列表中，但 adb 显示连接异常。请重新连接设备并确认手机已解锁授权。',
      action: device.connection === 'wireless' && device.endpoint ? 'reconnect' : null,
    };
  }

  return null;
}

export function canReconnectAndLaunch(device: ManagedDevice | null, toolsReady: boolean): boolean {
  return Boolean(device?.connection === 'wireless' && device.presence === 'offline' && device.endpoint && toolsReady);
}

export function launchAvailability(device: ManagedDevice | null, isMirroring: boolean, toolsReady: boolean) {
  const reconnectAndLaunch = canReconnectAndLaunch(device, toolsReady);
  const canLaunch = Boolean(
    !isMirroring &&
      ((device?.presence === 'online' && device.state === 'device' && toolsReady) || reconnectAndLaunch),
  );

  let hint = '';
  if (!device) hint = '请选择设备';
  else if (!toolsReady) hint = '请先完成工具配置';
  else if (device.presence === 'offline') {
    if (reconnectAndLaunch) hint = '确认无线调试地址后重连并启动投屏';
    else hint = device.connection === 'wireless' ? '缺少无线连接地址，请重新配对或通过 USB 转无线' : '设备当前不在线，插入 USB 后会自动刷新';
  } else if (device.state === 'unauthorized') {
    hint = '请先在手机上允许 USB 调试授权';
  } else if (device.state === 'offline') {
    hint = '设备连接异常，请重新连接';
  }

  return {
    canLaunch,
    reconnectAndLaunch,
    buttonText: reconnectAndLaunch ? '重连投屏' : '启动投屏',
    title: reconnectAndLaunch ? '需要先重连' : canLaunch ? '准备就绪' : '暂不能启动',
    hint,
  };
}

export function canRelaunchSession(device: ManagedDevice, toolsReady = true): boolean {
  if (!toolsReady) return false;
  if (device.connection === 'wireless' && device.presence === 'offline') {
    return Boolean(device.endpoint);
  }
  return device.presence === 'online' && device.state === 'device';
}
