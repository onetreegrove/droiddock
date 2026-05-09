import { defineStore } from 'pinia';
import type { ModalKey, PageKey, WirelessSource } from '../types/app';

export const useUiStore = defineStore('ui', {
  state: () => ({
    currentPage: 'devices' as PageKey,
    selectedSerial: null as string | null,
    modal: null as ModalKey,
    selectedLogSessionId: null as string | null,
    wirelessReconnectEndpoint: null as string | null,
    wirelessReconnectSource: 'manual' as WirelessSource,
  }),
  actions: {
    openPage(page: PageKey) {
      this.currentPage = page;
    },
    openModal(modal: Exclude<ModalKey, null>) {
      this.modal = modal;
    },
    openWirelessReconnect(endpoint: string, source: WirelessSource) {
      this.wirelessReconnectEndpoint = endpoint;
      this.wirelessReconnectSource = source;
      this.modal = 'wireless';
    },
    closeModal() {
      this.modal = null;
      this.wirelessReconnectEndpoint = null;
      this.wirelessReconnectSource = 'manual';
    },
    toggleLogSession(sessionId: string) {
      this.selectedLogSessionId = this.selectedLogSessionId === sessionId ? null : sessionId;
    },
  },
});
