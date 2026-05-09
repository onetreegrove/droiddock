import { defineStore } from 'pinia';
import type { ManagedDevice } from '../lib/ipc/types';
import { invokeCommand } from '../lib/ipc/client';
import { useUiStore } from './ui';

function preferredSerial(devices: ManagedDevice[]): string | null {
  return (
    devices.find((device) => device.presence === 'online' && device.state === 'device')?.serial ??
    devices[0]?.serial ??
    null
  );
}

export const useDevicesStore = defineStore('devices', {
  state: () => ({
    devices: [] as ManagedDevice[],
  }),
  getters: {
    availableDeviceCount: (state) =>
      state.devices.filter((device) => device.presence === 'online' && device.state === 'device').length,
    selectedDevice: (state) => {
      const ui = useUiStore();
      return state.devices.find((device) => device.serial === ui.selectedSerial) ?? null;
    },
  },
  actions: {
    async refreshDevices() {
      this.devices = await invokeCommand<ManagedDevice[]>('list_devices');
      const ui = useUiStore();
      if (!ui.selectedSerial && this.devices.length > 0) {
        ui.selectedSerial = preferredSerial(this.devices);
      }
      if (ui.selectedSerial && !this.devices.some((device) => device.serial === ui.selectedSerial)) {
        ui.selectedSerial = preferredSerial(this.devices);
      }
    },
  },
});
