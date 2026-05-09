import { defineStore } from 'pinia';
import type { Device } from '../lib/ipc/types';
import { invokeCommand } from '../lib/ipc/client';
import { useUiStore } from './ui';

export const useDevicesStore = defineStore('devices', {
  state: () => ({
    devices: [] as Device[],
  }),
  getters: {
    availableDeviceCount: (state) => state.devices.filter((device) => device.state === 'device').length,
    selectedDevice: (state) => {
      const ui = useUiStore();
      return state.devices.find((device) => device.serial === ui.selectedSerial) ?? null;
    },
  },
  actions: {
    async refreshDevices() {
      this.devices = await invokeCommand<Device[]>('list_devices');
      const ui = useUiStore();
      if (!ui.selectedSerial && this.devices.length > 0) {
        ui.selectedSerial = this.devices[0].serial;
      }
      if (ui.selectedSerial && !this.devices.some((device) => device.serial === ui.selectedSerial)) {
        ui.selectedSerial = this.devices[0]?.serial ?? null;
      }
    },
  },
});
