import init, { decode_packet, encode_packet } from 'wasmjs';

const statusEl = document.getElementById('status');
const rxEl = document.getElementById('rx');
const txEl = document.getElementById('tx');
const rxBytesEl = document.getElementById('rx-bytes');
const txBytesEl = document.getElementById('tx-bytes');
const errorEl = document.getElementById('error');

let rx = 0;
let tx = 0;
let rxBytes = 0;
let txBytes = 0;

function setStatus(text) {
  statusEl.textContent = text;
}

function setError(text) {
  errorEl.textContent = text;
  setStatus('error');
  console.error('[client] error:', text);
}

function asUint8Array(data) {
  if (data instanceof ArrayBuffer) {
    return new Uint8Array(data);
  }
  if (ArrayBuffer.isView(data)) {
    return new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
  }
  if (data instanceof Blob) {
    return data.arrayBuffer().then((buf) => new Uint8Array(buf));
  }
  throw new Error(`Unsupported WS binary type: ${Object.prototype.toString.call(data)}`);
}

function variantName(value) {
  if (value == null) {
    return 'none';
  }
  if (typeof value !== 'object') {
    return String(value);
  }

  if (typeof value.kind === 'string') {
    return value.kind;
  }
  if (typeof value.type === 'string') {
    return value.type;
  }

  const keys = Object.keys(value);
  if (keys.length === 1) {
    return keys[0];
  }
  return 'unknown';
}

function describePacket(packet) {
  const blocks = Array.isArray(packet.blocks)
    ? packet.blocks.map((b) => variantName(b)).join(', ')
    : '';
  const payload = variantName(packet.payload ?? null);
  return `got packet: blocks: [${blocks}], payload: ${payload}`;
}

async function run() {
  await init();

  const params = new URLSearchParams(window.location.search);
  const wsUrl = params.get('ws') ?? 'ws://127.0.0.1:19001';

  setStatus(`connecting ${wsUrl}`);

  const ws = new WebSocket(wsUrl);
  ws.binaryType = 'arraybuffer';

  ws.onopen = () => {
    setStatus('connected');
  };

  ws.onerror = () => {
    setError('websocket error');
  };

  ws.onclose = () => {
    if (statusEl.textContent !== 'error') {
      setStatus('done');
    }
    window.__wsEcho = { rx, tx, rxBytes, txBytes, status: statusEl.textContent };
    console.log(
      `CLIENT_SUMMARY packets_rx=${rx} packets_tx=${tx} bytes_rx=${rxBytes} bytes_tx=${txBytes}`,
    );
  };

  ws.onmessage = async (event) => {
    try {
      const inBytes = await asUint8Array(event.data);
      rxBytes += inBytes.length;
      const packet = decode_packet(inBytes);
      console.log(`[client] ${describePacket(packet)}`);
      const outBytes = encode_packet(packet.blocks, packet.payload ?? null);
      txBytes += outBytes.length;

      ws.send(outBytes);

      rx += 1;
      tx += 1;
      rxEl.textContent = `received: ${rx}`;
      txEl.textContent = `sent: ${tx}`;
      rxBytesEl.textContent = `received bytes: ${rxBytes}`;
      txBytesEl.textContent = `sent bytes: ${txBytes}`;
      window.__wsEcho = { rx, tx, rxBytes, txBytes, status: statusEl.textContent };
    } catch (err) {
      const message = err instanceof Error ? err.stack ?? err.message : String(err);
      setError(message);
      window.__wsEcho = { rx, tx, rxBytes, txBytes, status: 'error', error: message };
      ws.close();
    }
  };
}

run().catch((err) => {
  const message = err instanceof Error ? err.stack ?? err.message : String(err);
  setError(message);
  window.__wsEcho = { rx, tx, rxBytes, txBytes, status: 'error', error: message };
});
