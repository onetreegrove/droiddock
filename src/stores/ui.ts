import { defineStore } from 'pinia';
import type { ModalKey, PageKey, WirelessSource } from '../types/app';

export const useUiStore = defineStore('ui', {
  state: () => ({
    currentPage: 'devices' as PageKey,
    selectedSerial: null as string | null,
    modal: null as ModalKey,
    selectedLogSessionId: null as string | null,
    reconnectDeviceSerial: null as string | null,
    reconnectEndpoint: null as string | null,
    reconnectSource: 'manual' as WirelessSource,
    reconnectLaunchAfterConnect: false,
  }),
  actions: {
    openPage(page: PageKey) {
      this.currentPage = page;
    },
    showDevice(serial: string) {
      this.selectedSerial = serial;
      this.currentPage = 'devices';
    },
    openModal(modal: Exclude<ModalKey, null>) {
      this.modal = modal;
    },
    openReconnectModal(serial: string, endpoint: string, source: WirelessSource | null = 'manual', launchAfterConnect = false) {
      this.reconnectDeviceSerial = serial;
      this.reconnectEndpoint = endpoint;
      this.reconnectSource = source || 'manual';
      this.reconnectLaunchAfterConnect = launchAfterConnect;
      this.modal = 'reconnect';
    },
    closeModal() {
      this.modal = null;
      this.reconnectDeviceSerial = null;
      this.reconnectEndpoint = null;
      this.reconnectSource = 'manual';
      this.reconnectLaunchAfterConnect = false;
    },
    toggleLogSession(sessionId: string) {
      this.selectedLogSessionId = this.selectedLogSessionId === sessionId ? null : sessionId;
    },
  },
});
