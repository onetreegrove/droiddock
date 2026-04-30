import { describe, expect, it } from 'vitest';
import { buildPairRequest, deviceIpAddress } from './wireless';

describe('wireless device helpers', () => {
  it('extracts IP address from a wireless adb serial', () => {
    expect(deviceIpAddress('192.168.1.105:5555')).toBe('192.168.1.105');
    expect(deviceIpAddress('R9YT301WXXX')).toBe(null);
  });

  it('keeps pair and connect endpoints independent in pair requests', () => {
    expect(
      buildPairRequest({
        pairHost: '192.168.1.100',
        pairPort: '38521',
        pairingCode: '123456',
        connectHost: '192.168.1.101',
        connectPort: '39845',
      }),
    ).toEqual({
      host: '192.168.1.100',
      pair_port: 38521,
      pairing_code: '123456',
      connect_host: '192.168.1.101',
      connect_port: 39845,
    });
  });

  it('leaves connect host unset when the form submits a blank connect host', () => {
    expect(
      buildPairRequest({
        pairHost: '192.168.1.100',
        pairPort: '38521',
        pairingCode: '123456',
        connectHost: '',
        connectPort: '39845',
      }),
    ).toMatchObject({
      host: '192.168.1.100',
      connect_host: null,
    });
  });

  it('rejects a pair port outside the adb u16 port range', () => {
    expect(() =>
      buildPairRequest({
        pairHost: '192.168.1.100',
        pairPort: '728115',
        pairingCode: '123456',
        connectHost: '192.168.1.100',
        connectPort: '39845',
      }),
    ).toThrow('配对端口必须是 1-65535 之间的数字');
  });
});
