import { type RawData } from 'ws';

export function rawDataToBuffer(data: RawData): Buffer {
  if (Buffer.isBuffer(data)) {
    return data;
  }
  if (Array.isArray(data)) {
    return Buffer.concat(data);
  }
  if (data instanceof ArrayBuffer) {
    return Buffer.from(data);
  }
  return assertNever(data);
}

function assertNever(value: never): never {
  throw new Error(`unsupported websocket binary frame type: ${String(value)}`);
}
