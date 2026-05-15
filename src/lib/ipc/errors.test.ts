import { describe, expect, it } from 'vitest';
import { errorUserMessage, normalizeIpcError } from './errors';

describe('normalizeIpcError', () => {
  it('normalizes a backend structured error', () => {
    expect(
      normalizeIpcError({
        code: 'device_unauthorized',
        user_message: '请解锁手机',
        technical_detail: 'unauthorized',
        retryable: true,
      }),
    ).toEqual({
      code: 'device_unauthorized',
      userMessage: '请解锁手机',
      technicalDetail: 'unauthorized',
      retryable: true,
    });
  });

  it('normalizes string errors from legacy commands', () => {
    expect(normalizeIpcError('adb not found')).toEqual({
      code: 'unknown',
      userMessage: 'adb not found',
      technicalDetail: 'adb not found',
      retryable: false,
    });
  });

  it('extracts the user message from normalized errors for UI display', () => {
    expect(
      errorUserMessage({
        code: 'wireless_port_unavailable',
        userMessage: '无线调试端口不可用，请检查 IP、端口和手机无线调试是否开启',
        technicalDetail: 'failed to connect',
        retryable: true,
      }),
    ).toBe('无线调试端口不可用，请检查 IP、端口和手机无线调试是否开启');
  });
});
