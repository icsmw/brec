import init, { decode_packet, encode_packet } from 'wasmjs';

const ui = {
  status: document.getElementById('status'),
  rx: document.getElementById('rx'),
  tx: document.getElementById('tx'),
  rxBytes: document.getElementById('rx-bytes'),
  txBytes: document.getElementById('tx-bytes'),
  error: document.getElementById('error'),
};

class Tracking {
  constructor(elements) {
    this.ui = elements;
    this.rx = 0;
    this.tx = 0;
    this.rxBytes = 0;
    this.txBytes = 0;
    this.status = 'init';
    this.error = null;
  }

  setStatus(text) {
    this.ui.status.textContent = text;
    this.status = text;
  }

  setError(text) {
    this.ui.error.textContent = text;
    this.setStatus('error');
    this.error = text;
    console.error('[client] error:', text);
  }

  asUint8Array(data) {
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

  addRxBytes(count) {
    this.rxBytes += count;
  }

  addTxBytes(count) {
    this.txBytes += count;
  }

  incPackets() {
    this.rx += 1;
    this.tx += 1;
  }

  render() {
    this.ui.rx.textContent = `received: ${this.rx}`;
    this.ui.tx.textContent = `sent: ${this.tx}`;
    this.ui.rxBytes.textContent = `received bytes: ${this.rxBytes}`;
    this.ui.txBytes.textContent = `sent bytes: ${this.txBytes}`;
  }

  printStatus() {
    console.log(
      `[client] status packets_rx=${this.rx} packets_tx=${this.tx} bytes_rx=${this.rxBytes} bytes_tx=${this.txBytes}`,
    );
  }

  summaryLine() {
    return `CLIENT_SUMMARY packets_rx=${this.rx} packets_tx=${this.tx} bytes_rx=${this.rxBytes} bytes_tx=${this.txBytes}`;
  }
}


async function run() {
  const tracking = new Tracking(ui);
  await init();

  const params = new URLSearchParams(window.location.search);
  const wsUrl = params.get('ws') ?? 'ws://127.0.0.1:19001';

  tracking.setStatus(`connecting ${wsUrl}`);

  const ws = new WebSocket(wsUrl);
  ws.binaryType = 'arraybuffer';

  ws.onopen = () => {
    tracking.setStatus('connected');
  };

  ws.onerror = () => {
    tracking.setError('websocket error');
  };

  ws.onclose = () => {
    if (tracking.status !== 'error') {
      tracking.setStatus('done');
    }
    console.log(tracking.summaryLine());
  };

  ws.onmessage = async (event) => {
    try {
      const inBytes = await tracking.asUint8Array(event.data);
      tracking.addRxBytes(inBytes.length);
      const packet = decode_packet(inBytes);
      tracking.printStatus();
      const outBytes = encode_packet(packet);
      tracking.addTxBytes(outBytes.length);

      ws.send(outBytes);

      tracking.incPackets();
      tracking.render();
    } catch (err) {
      const message = err instanceof Error ? err.stack ?? err.message : String(err);
      tracking.setError(message);
      ws.close();
    }
  };
}

run().catch((err) => {
  const message = err instanceof Error ? err.stack ?? err.message : String(err);
  console.error('[client] error:', message);
});
