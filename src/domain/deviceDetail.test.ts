import { describe, expect, it } from 'vitest';
import type { ManagedDevice } from '../types/app';
import {
  canRelaunchSession,
  detailSecondaryItems,
  launchAvailability,
  statusBanner,
} from './deviceDetail';

function device(overrides: Partial<ManagedDevice> = {}): ManagedDevice {
  return {
    serial: '10.128.0.151:41317',
    state: 'device',
    presence: 'online',
    model: 'PFTM20',
    product: null,
    connection: 'wireless',
    wireless_source: 'adb_pair',
    endpoint: '10.128.0.151:41317',
    display_name: 'PFTM20',
    alias: null,
    raw: null,
    last_seen_at: 0,
    last_connected_at: null,
    ...overrides,
  };
}

describe('device detail presentation helpers', () => {
  it('does not repeat the same wireless endpoint as both endpoint and serial', () => {
    expect(detailSecondaryItems(device(), 'ADB Pair 无线')).toEqual(['PFTM20', 'ADB Pair 无线', '10.128.0.151:41317']);
  });

  it('shows guidance for adb offline devices that are still present in the live list', () => {
    expect(statusBanner(device({ state: 'offline', presence: 'online' }))).toMatchObject({
      tone: 'error',
      title: '设备连接异常',
      action: 'reconnect',
    });
  });

  it('keeps session relaunch eligibility aligned with the primary launch action', () => {
    expect(canRelaunchSession(device({ state: 'unauthorized', connection: 'usb', endpoint: null }))).toBe(false);
    expect(canRelaunchSession(device({ presence: 'offline', endpoint: null }))).toBe(false);
    expect(canRelaunchSession(device({ presence: 'offline', endpoint: '10.128.0.151:41317' }))).toBe(true);
    expect(canRelaunchSession(device(), false)).toBe(false);
  });

  it('returns a visible disabled reason for blocked primary launches', () => {
    expect(launchAvailability(device({ state: 'unauthorized', connection: 'usb', endpoint: null }), false, true)).toMatchObject({
      canLaunch: false,
      title: '暂不能启动',
      hint: '请先在手机上允许 USB 调试授权',
    });
  });

  it('labels reconnect-and-launch as a reconnect step instead of ready', () => {
    expect(launchAvailability(device({ presence: 'offline' }), false, true)).toMatchObject({
      canLaunch: true,
      reconnectAndLaunch: true,
      title: '需要先重连',
      buttonText: '重连投屏',
    });
  });
});
