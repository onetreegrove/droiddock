import { describe, expect, it } from 'vitest';
import { canAutoCloseReconnectModal, reconnectSuccessCloseDelayMs, reconnectSuccessMessage } from './reconnectFeedback';

describe('reconnect modal feedback', () => {
  it('shows the endpoint before closing after a successful reconnect', () => {
    expect(reconnectSuccessMessage('10.128.0.151:39407')).toBe('已连接到 10.128.0.151:39407');
  });

  it('keeps successful feedback visible briefly before auto close', () => {
    expect(reconnectSuccessCloseDelayMs).toBe(1000);
  });

  it('only allows the delayed success timer to close the reconnect modal', () => {
    expect(canAutoCloseReconnectModal('reconnect')).toBe(true);
    expect(canAutoCloseReconnectModal('pair')).toBe(false);
    expect(canAutoCloseReconnectModal(null)).toBe(false);
  });
});
