import { describe, expect, it } from 'vitest';
import { normalizeIpcError } from './errors';

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
});
