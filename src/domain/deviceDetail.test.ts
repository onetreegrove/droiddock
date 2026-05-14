import { describe, expect, it } from 'vitest';
import type { ManagedDevice } from '../types/app';
import {
  canRestoreConnection,
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

  it('keeps recoverable connection issues inside the session action area', () => {
    expect(statusBanner(device({ presence: 'offline' }))).toBeNull();
    expect(statusBanner(device({ state: 'offline', presence: 'online' }))).toBeNull();
  });

  it('still shows the top banner for device authorization guidance', () => {
    expect(statusBanner(device({ state: 'unauthorized', connection: 'usb', endpoint: null }))).toMatchObject({
      tone: 'warn',
      title: '设备待授权',
    });
  });

  it('keeps session relaunch eligibility aligned with the primary launch action', () => {
    expect(canRelaunchSession(device({ state: 'unauthorized', connection: 'usb', endpoint: null }))).toBe(false);
    expect(canRelaunchSession(device({ presence: 'offline', endpoint: null }))).toBe(false);
    expect(canRelaunchSession(device({ presence: 'offline', endpoint: '10.128.0.151:41317' }))).toBe(false);
    expect(canRelaunchSession(device(), false)).toBe(false);
  });

  it('returns a visible disabled reason for blocked primary launches', () => {
    expect(launchAvailability(device({ state: 'unauthorized', connection: 'usb', endpoint: null }), false, true)).toMatchObject({
      canLaunch: false,
      title: '暂不能启动',
      hint: '请先在手机上允许 USB 调试授权',
    });
  });

  it('does not treat offline wireless devices as launch-ready', () => {
    expect(launchAvailability(device({ presence: 'offline' }), false, true)).toMatchObject({
      canLaunch: false,
      title: '暂不能启动',
      buttonText: '启动投屏',
      hint: '请先恢复无线连接',
    });
  });

  it('allows restoring wireless connections without launching', () => {
    expect(canRestoreConnection(device({ presence: 'offline' }), true)).toBe(true);
    expect(canRestoreConnection(device({ state: 'offline', presence: 'online' }), true)).toBe(true);
    expect(canRestoreConnection(device({ presence: 'offline', endpoint: null }), true)).toBe(false);
    expect(canRestoreConnection(device({ connection: 'usb', presence: 'offline', endpoint: null }), true)).toBe(false);
    expect(canRestoreConnection(device({ presence: 'offline' }), false)).toBe(false);
  });
});
