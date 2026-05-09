import type { PairRequest } from '../types/app';
import type { WirelessSource } from '../lib/ipc/types';

type PairFormValues = {
  pairHost: string;
  pairPort: string;
  pairingCode: string;
  connectHost: string;
  connectPort: string;
};

const wirelessSerialPattern = /^(\d{1,3}(?:\.\d{1,3}){3}):\d+$/;
const MAX_PORT = 65535;

export function deviceIpAddress(serial: string): string | null {
  return wirelessSerialPattern.exec(serial)?.[1] ?? null;
}

function parsePort(value: string, label: string, required: boolean): number | null {
  const trimmed = value.trim();
  if (!trimmed) {
    if (required) throw new Error(`${label}必须是 1-65535 之间的数字`);
    return null;
  }

  const port = Number(trimmed);
  if (!Number.isInteger(port) || port < 1 || port > MAX_PORT) {
    throw new Error(`${label}必须是 1-65535 之间的数字`);
  }

  return port;
}

export function splitConnectEndpoint(endpoint: string): { host: string; port: string } {
  const trimmed = endpoint.trim();
  const separator = trimmed.lastIndexOf(':');
  if (separator <= 0 || separator === trimmed.length - 1) {
    return { host: trimmed, port: '' };
  }

  return {
    host: trimmed.slice(0, separator),
    port: trimmed.slice(separator + 1),
  };
}

export function buildConnectEndpoint(host: string, port: string): string {
  const trimmedHost = host.trim();
  if (!trimmedHost) {
    throw new Error('连接 IP 不能为空');
  }

  const connectPort = parsePort(port, '连接端口', true);
  return `${trimmedHost}:${connectPort}`;
}

export function wirelessSourceLabel(source: WirelessSource | null): string {
  if (source === 'adb_pair') return 'ADB Pair 无线';
  if (source === 'usb_tcpip') return 'USB 转无线';
  return '无线';
}

export function buildPairRequest(values: PairFormValues): PairRequest {
  const pairHost = values.pairHost.trim();
  const connectHost = values.connectHost.trim();

  return {
    host: pairHost,
    pair_port: parsePort(values.pairPort, '配对端口', true) ?? 0,
    pairing_code: values.pairingCode.trim(),
    connect_host: connectHost || null,
    connect_port: parsePort(values.connectPort, '连接端口', false),
  };
}
