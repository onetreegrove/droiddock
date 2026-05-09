import { invoke } from '@tauri-apps/api/core';
import { normalizeIpcError } from './errors';

export async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw normalizeIpcError(error);
  }
}
