export type AppErrorPayload = {
  code: string;
  userMessage: string;
  technicalDetail: string | null;
  retryable: boolean;
};

type BackendErrorPayload = {
  code?: string;
  user_message?: string;
  technical_detail?: string | null;
  retryable?: boolean;
};

export function normalizeIpcError(error: unknown): AppErrorPayload {
  if (typeof error === 'object' && error !== null) {
    const payload = error as BackendErrorPayload;
    if (typeof payload.user_message === 'string') {
      return {
        code: payload.code ?? 'unknown',
        userMessage: payload.user_message,
        technicalDetail: payload.technical_detail ?? null,
        retryable: Boolean(payload.retryable),
      };
    }
  }

  const message = String(error);
  return {
    code: 'unknown',
    userMessage: message,
    technicalDetail: message,
    retryable: false,
  };
}

export function errorUserMessage(error: unknown): string {
  if (typeof error === 'object' && error !== null && 'userMessage' in error) {
    return String((error as AppErrorPayload).userMessage);
  }

  return String(error instanceof Error ? error.message : error);
}
