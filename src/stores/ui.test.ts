import { beforeEach, describe, expect, it } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useUiStore } from './ui';

describe('ui store reconnect modal', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('opens reconnect modal with the current wireless device endpoint', () => {
    const ui = useUiStore();

    ui.openReconnectModal('192.168.1.105:5555', '192.168.1.105:39845', 'adb_pair');

    expect(ui.modal).toBe('reconnect');
    expect(ui.reconnectDeviceSerial).toBe('192.168.1.105:5555');
    expect(ui.reconnectEndpoint).toBe('192.168.1.105:39845');
    expect(ui.reconnectSource).toBe('adb_pair');
  });

  it('falls back to manual source when saved source is missing', () => {
    const ui = useUiStore();

    ui.openReconnectModal('192.168.1.105:5555', '192.168.1.105:39845', null);

    expect(ui.modal).toBe('reconnect');
    expect(ui.reconnectSource).toBe('manual');
  });

  it('marks reconnect modal as launch-after-connect when opened from mirror start', () => {
    const ui = useUiStore();

    ui.openReconnectModal('192.168.1.105:5555', '192.168.1.105:39845', 'adb_pair', true);

    expect(ui.modal).toBe('reconnect');
    expect(ui.reconnectLaunchAfterConnect).toBe(true);
  });
});
