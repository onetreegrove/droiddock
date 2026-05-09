import { defineStore } from 'pinia';
import type { ModalKey, PageKey } from '../types/app';

export const useUiStore = defineStore('ui', {
  state: () => ({
    currentPage: 'dashboard' as PageKey,
    selectedSerial: null as string | null,
    modal: null as ModalKey,
    selectedLogSessionId: null as string | null,
  }),
  actions: {
    openPage(page: PageKey) {
      this.currentPage = page;
    },
    openModal(modal: Exclude<ModalKey, null>) {
      this.modal = modal;
    },
    closeModal() {
      this.modal = null;
    },
    toggleLogSession(sessionId: string) {
      this.selectedLogSessionId = this.selectedLogSessionId === sessionId ? null : sessionId;
    },
  },
});
