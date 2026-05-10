import WebSocket, { type RawData } from 'ws';
import { decodePacket, encodePacket } from 'protocol';
import { findDiff } from './diff';
import { rawDataToBuffer, valueToBuffer } from './frames';

type Counters = {
  packets: number;
  recvBytes: number;
  sentBytes: number;
};

class Connector {
  private readonly socket: WebSocket;

  public constructor(
    private readonly wsAddr: string,
    private readonly onMessage: (data: Buffer) => Buffer,
    private readonly onClose: () => void,
    private readonly onError: (err: unknown) => void,
  ) {
    this.socket = new WebSocket(`ws://${wsAddr}`);
  }

  public connect(): void {
    this.socket.on('open', this.handleOpen.bind(this));
    this.socket.on('message', this.handleMessage.bind(this));
    this.socket.on('close', this.onClose);
    this.socket.on('error', this.onError);
  }

  private handleOpen(): void {
    console.log(`[client] connected to ws://${this.wsAddr}`);
  }

  private handleMessage(data: RawData, isBinary: boolean): void {
    if (!isBinary) {
      return;
    }

    try {
      const input = rawDataToBuffer(data);
      const output = this.onMessage(input);
      this.socket.send(output, { binary: true });
    } catch (err) {
      this.onError(err);
    }
  }
}

class Test {
  private readonly counters: Counters = {
    packets: 0,
    recvBytes: 0,
    sentBytes: 0,
  };

  private closed = false;

  public constructor(
    private readonly wsAddr: string,
    private readonly expectedCount: number,
  ) {}

  public run(): void {
    const connector = new Connector(
      this.wsAddr,
      this.handlePacket.bind(this),
      this.handleClose.bind(this),
      this.fail.bind(this),
    );

    connector.connect();
  }

  private handlePacket(input: Buffer): Buffer {
    const output = this.verifyRoundtrip(input);

    this.counters.recvBytes += input.length;
    this.counters.sentBytes += output.length;
    this.counters.packets += 1;
    this.reportProgress();

    return output;
  }

  private verifyRoundtrip(input: Buffer): Buffer {
    const packet = decodePacket(input);
    const output = valueToBuffer(encodePacket(packet));
    const packetAfter = decodePacket(output);
    const diff = findDiff(packet, packetAfter);

    if (diff) {
      throw new Error(
        `client reencode mismatch at ${diff.path}: ${diff.reason}; before=${String(diff.left)} after=${String(diff.right)}`,
      );
    }

    return output;
  }

  private reportProgress(): void {
    if (this.counters.packets % 10 === 0 || this.counters.packets === this.expectedCount) {
      console.log(`[client] progress: ${this.counters.packets}/${this.expectedCount}`);
    }
  }

  private handleClose(): void {
    if (this.closed) {
      return;
    }

    this.closed = true;
    const verified = this.counters.packets === this.expectedCount;
    this.printSummary(verified);
    process.exit(verified ? 0 : 1);
  }

  private fail(err: unknown): void {
    if (this.closed) {
      return;
    }

    this.closed = true;
    console.error(`[client] error: ${err instanceof Error ? err.stack || err.message : String(err)}`);
    process.exit(1);
  }

  private printSummary(verified: boolean): void {
    const expectedSuffix = verified ? '' : ` expected=${this.expectedCount}`;
    console.log(
      `CLIENT_SUMMARY target=node packets=${this.counters.packets} sent_bytes=${this.counters.sentBytes} recv_bytes=${this.counters.recvBytes} verified=${verified}${expectedSuffix}`,
    );
  }
}

function main(): void {
  const wsAddr = process.env.TEST_WS_ADDR || 'host.docker.internal:19001';
  const expectedCount = Number.parseInt(process.env.TEST_PACKAGE_COUNT || '1000', 10);

  new Test(wsAddr, expectedCount).run();
}

main();
