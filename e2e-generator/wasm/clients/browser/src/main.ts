import { decodePacket, encodePacket, initWasm } from 'protocol';

type Ui = {
  status: HTMLElement;
  rx: HTMLElement;
  tx: HTMLElement;
  rxBytes: HTMLElement;
  txBytes: HTMLElement;
  error: HTMLElement;
};

function requiredElement(id: string): HTMLElement {
  const element = document.getElementById(id);
  if (!element) {
    throw new Error(`Missing element #${id}`);
  }
  return element;
}

const ui: Ui = {
  status: requiredElement('status'),
  rx: requiredElement('rx'),
  tx: requiredElement('tx'),
  rxBytes: requiredElement('rx-bytes'),
  txBytes: requiredElement('tx-bytes'),
  error: requiredElement('error'),
};

class Tracking {
  private rx = 0;
  private tx = 0;
  private rxBytes = 0;
  private txBytes = 0;
  private status = 'init';
  private error: string | null = null;

  public constructor(private readonly elements: Ui) {}

  public isFailed(): boolean {
    return this.status === 'error';
  }

  public setStatus(text: string): void {
    this.elements.status.textContent = text;
    this.status = text;
  }

  public setError(text: string): void {
    this.elements.error.textContent = text;
    this.setStatus('error');
    this.error = text;
    console.error('[client] error:', text);
  }

  public async asUint8Array(data: MessageEvent['data']): Promise<Uint8Array> {
    if (data instanceof ArrayBuffer) {
      return new Uint8Array(data);
    }
    if (ArrayBuffer.isView(data)) {
      return new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
    }
    if (data instanceof Blob) {
      const buf = await data.arrayBuffer();
      return new Uint8Array(buf);
    }
    throw new Error(`Unsupported WS binary type: ${Object.prototype.toString.call(data)}`);
  }

  public addRxBytes(count: number): void {
    this.rxBytes += count;
  }

  public addTxBytes(count: number): void {
    this.txBytes += count;
  }

  public incPackets(): void {
    this.rx += 1;
    this.tx += 1;
  }

  public render(): void {
    this.elements.rx.textContent = `received: ${this.rx}`;
    this.elements.tx.textContent = `sent: ${this.tx}`;
    this.elements.rxBytes.textContent = `received bytes: ${this.rxBytes}`;
    this.elements.txBytes.textContent = `sent bytes: ${this.txBytes}`;
  }

  public printStatus(): void {
    console.log(
      `[client] status packets_rx=${this.rx} packets_tx=${this.tx} bytes_rx=${this.rxBytes} bytes_tx=${this.txBytes}`,
    );
  }

  public summaryLine(): string {
    return `CLIENT_SUMMARY target=browser packets_rx=${this.rx} packets_tx=${this.tx} bytes_rx=${this.rxBytes} bytes_tx=${this.txBytes}`;
  }
}

async function run(): Promise<void> {
  const tracking = new Tracking(ui);
  await initWasm();

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
    if (!tracking.isFailed()) {
      tracking.setStatus('done');
    }
    console.log(tracking.summaryLine());
  };

  ws.onmessage = async (event: MessageEvent) => {
    try {
      const inBytes = await tracking.asUint8Array(event.data);
      tracking.addRxBytes(inBytes.length);
      const packet = decodePacket(inBytes);
      tracking.printStatus();
      const outBytes = encodePacket(packet);
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

run().catch((err: unknown) => {
  const message = err instanceof Error ? err.stack ?? err.message : String(err);
  console.error('[client] error:', message);
});
