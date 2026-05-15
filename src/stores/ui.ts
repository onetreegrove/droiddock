import { defineStore } from 'pinia';
import type { ModalKey, PageKey, WirelessSource } from '../types/app';

let nextToastId = 1;

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
    toasts: [] as { id: number; message: string; type: 'success' | 'error' | 'info' }[],
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
    pushToast(message: string, type: 'success' | 'error' | 'info' = 'success', duration = 3000) {
      const id = nextToastId;
      nextToastId += 1;
      this.toasts.push({ id, message, type });
      setTimeout(() => {
        this.removeToast(id);
      }, duration);
    },
    removeToast(id: number) {
      this.toasts = this.toasts.filter((t) => t.id !== id);
    },
  },
});
