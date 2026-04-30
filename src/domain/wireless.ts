import type { PairRequest } from '../types/app';

type PairFormValues = {
  pairHost: string;
  pairPort: string;
  pairingCode: string;
  connectHost: string;
  connectPort: string;
};

const wirelessSerialPattern = /^(\d{1,3}(?:\.\d{1,3}){3}):\d+$/;

export function deviceIpAddress(serial: string): string | null {
  return wirelessSerialPattern.exec(serial)?.[1] ?? null;
}

export function buildPairRequest(values: PairFormValues): PairRequest {
  const pairHost = values.pairHost.trim();
  const connectHost = values.connectHost.trim();

  return {
    host: pairHost,
    pair_port: Number(values.pairPort),
    pairing_code: values.pairingCode.trim(),
    connect_host: connectHost || null,
    connect_port: values.connectPort.trim() ? Number(values.connectPort) : null,
  };
}
