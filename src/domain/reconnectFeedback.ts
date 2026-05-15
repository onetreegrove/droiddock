export const reconnectSuccessCloseDelayMs = 1000;

export function reconnectSuccessMessage(endpoint: string): string {
  return `已连接到 ${endpoint}`;
}

export function canAutoCloseReconnectModal(modal: string | null): boolean {
  return modal === 'reconnect';
}
