'use strict';

const WebSocket = require('ws');
const addon = require('../native/bindings.node');

const decodePacket = addon.decodePacket ?? addon.decode_packet;
const encodePacketObjectRaw =
  addon.encodePacketObject ?? addon.encode_packet_object;
const encodePacket = addon.encodePacket ?? addon.encode_packet;

const encodePacketObject = typeof encodePacketObjectRaw === 'function'
  ? encodePacketObjectRaw
  : (packet) => {
      if (typeof encodePacket !== 'function') {
        throw new Error(
          'bindings.node does not export encodePacketObject/encode_packet_object or encodePacket/encode_packet',
        );
      }
      return encodePacket(packet.blocks, packet.payload ?? null);
    };

if (typeof decodePacket !== 'function') {
  throw new Error(
    'bindings.node does not export decodePacket/decode_packet',
  );
}

const wsAddr = process.env.TEST_WS_ADDR || 'host.docker.internal:19001';
const expectedCount = Number.parseInt(process.env.TEST_PACKAGE_COUNT || '1000', 10);
const socket = new WebSocket(`ws://${wsAddr}`);

let packetCount = 0;
let recvBytes = 0;
let sentBytes = 0;
let closed = false;

const findDiff = (a, b, path = '$') => {
  if (Object.is(a, b)) {
    return null;
  }

  if (typeof a !== typeof b) {
    return { path, left: a, right: b, reason: 'type mismatch' };
  }

  if (a === null || b === null) {
    return { path, left: a, right: b, reason: 'null mismatch' };
  }

  if (Array.isArray(a) || Array.isArray(b)) {
    if (!Array.isArray(a) || !Array.isArray(b)) {
      return { path, left: a, right: b, reason: 'array mismatch' };
    }
    if (a.length !== b.length) {
      return { path: `${path}.length`, left: a.length, right: b.length, reason: 'array length mismatch' };
    }
    for (let i = 0; i < a.length; i += 1) {
      const diff = findDiff(a[i], b[i], `${path}[${i}]`);
      if (diff) {
        return diff;
      }
    }
    return null;
  }

  if (typeof a === 'object') {
    const keys = Array.from(new Set([...Object.keys(a), ...Object.keys(b)])).sort();
    for (const key of keys) {
      if (!(key in a) || !(key in b)) {
        return { path: `${path}.${key}`, left: a[key], right: b[key], reason: 'missing key' };
      }
      const diff = findDiff(a[key], b[key], `${path}.${key}`);
      if (diff) {
        return diff;
      }
    }
    return null;
  }

  return { path, left: a, right: b, reason: 'value mismatch' };
};


const fail = (err) => {
  if (closed) {
    return;
  }
  closed = true;
  console.error(`[client] error: ${err instanceof Error ? err.stack || err.message : String(err)}`);
  process.exit(1);
};

const assertAddonThrows = (label, fn, expectedSubstr) => {
  let thrown = null;
  try {
    fn();
  } catch (err) {
    thrown = err;
  }
  if (!thrown) {
    throw new Error(`[negative] ${label}: expected addon call to throw`);
  }
  const message = String(thrown && thrown.message ? thrown.message : thrown);
  if (expectedSubstr && !message.includes(expectedSubstr)) {
    throw new Error(
      `[negative] ${label}: unexpected error message: ${message}`,
    );
  }
  console.log(`[negative] ${label}: ok (${message})`);
};

const runNegativeNapiCases = () => {
  if (typeof encodePacketObject !== 'function') {
    console.log('[negative] skip: encodePacketObject is not available');
    return;
  }

  const makePayloadABase = () => ({
    field_u8: 1,
    field_u16: 2,
    field_u32: 3,
    field_u64: 4n,
    field_u128: 5n,
    field_i8: -1,
    field_i16: -2,
    field_i32: -3,
    field_i64: -4n,
    field_i128: -5n,
    field_f32: 0,
    field_f64: 0n,
    field_bool: true,
    field_str: 'ok',
    vec_u8: [],
    vec_u16: [],
    vec_u32: [],
    vec_u64: [],
    vec_u128: [],
    vec_i8: [],
    vec_i16: [],
    vec_i32: [],
    vec_i64: [],
    vec_i128: [],
    vec_str: [],
  });

  const makePacketWithPayloadA = (payloadOverrides) => ({
    blocks: [],
    payload: {
      PayloadA: {
        ...makePayloadABase(),
        ...payloadOverrides,
      },
    },
  });

  assertAddonThrows(
    'packet object is not object',
    () => encodePacketObject(42),
    'Encode packet object',
  );

  assertAddonThrows(
    'packet object missing blocks',
    () => encodePacketObject({ payload: null }),
    'Encode packet object',
  );

  assertAddonThrows(
    'packet blocks wrong type',
    () => encodePacketObject({ blocks: 123, payload: null }),
    'Encode packet object',
  );

  assertAddonThrows(
    'packet block unknown key',
    () => encodePacketObject({ blocks: [{ UnknownBlock: { field: 1 } }], payload: null }),
    'Encode packet object',
  );

  assertAddonThrows(
    'packet block field wrong type',
    () => encodePacketObject({ blocks: [{ BlockU8: { field: 'bad' } }], payload: null }),
    'Encode packet object',
  );

  // Explicit macro-level conversion errors for i64/u64/i128/u128 in NapiConvert.
  assertAddonThrows(
    'payloadA i64 wrong js type',
    () => encodePacketObject(makePacketWithPayloadA({ field_i64: 10 })),
    'Encode packet object',
  );

  assertAddonThrows(
    'payloadA i64 out of range',
    () => encodePacketObject(makePacketWithPayloadA({ field_i64: 1n << 80n })),
    'Encode packet object',
  );

  assertAddonThrows(
    'payloadA u64 negative bigint',
    () => encodePacketObject(makePacketWithPayloadA({ field_u64: -1n })),
    'Encode packet object',
  );

  assertAddonThrows(
    'payloadA i128 out of range',
    () => encodePacketObject(makePacketWithPayloadA({ field_i128: 1n << 200n })),
    'Encode packet object',
  );

  assertAddonThrows(
    'payloadA u128 negative bigint',
    () => encodePacketObject(makePacketWithPayloadA({ field_u128: -1n })),
    'Encode packet object',
  );

  assertAddonThrows(
    'payloadA vec_u64 contains negative bigint',
    () => encodePacketObject(makePacketWithPayloadA({ vec_u64: [1n, -2n] })),
    'Encode packet object',
  );

  assertAddonThrows(
    'payloadA vec_i64 contains number',
    () => encodePacketObject(makePacketWithPayloadA({ vec_i64: [1n, 2] })),
    'Encode packet object',
  );
};

try {
  runNegativeNapiCases();
} catch (err) {
  fail(err);
}

socket.on('open', () => {
  console.log(`[client] connected to ws://${wsAddr}`);
});

socket.on('message', (data, isBinary) => {
  if (!isBinary) {
    return;
  }

  try {
    const input = Buffer.isBuffer(data) ? data : Buffer.from(data);
    recvBytes += input.length;

    const packet = decodePacket(input);
    
    const output = Buffer.from(encodePacketObject(packet));
    const packetAfter = decodePacket(output);
    const diff = findDiff(packet, packetAfter);
    if (diff) {
      throw new Error(`client reencode mismatch at ${diff.path}: ${diff.reason}; before=${String(diff.left)} after=${String(diff.right)}`);
    }

    sentBytes += output.length;
    packetCount += 1;

    socket.send(output, { binary: true });

    if (packetCount % 10 === 0 || packetCount === expectedCount) {
      console.log(`[client] progress: ${packetCount}/${expectedCount}`);
    }
  } catch (err) {
    fail(err);
  }
});

socket.on('close', () => {
  if (closed) {
    return;
  }
  closed = true;

  if (packetCount !== expectedCount) {
    console.error(`CLIENT_SUMMARY packets=${packetCount} sent_bytes=${sentBytes} recv_bytes=${recvBytes} verified=false expected=${expectedCount}`);
    process.exit(1);
    return;
  }

  console.log(`CLIENT_SUMMARY packets=${packetCount} sent_bytes=${sentBytes} recv_bytes=${recvBytes} verified=true`);
  process.exit(0);
});

socket.on('error', fail);
