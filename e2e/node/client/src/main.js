'use strict';

const WebSocket = require('ws');
const addon = require('../native/bindings.node');

const decodePacket = addon.decodePacket ?? addon.decode_packet;
const encodePacketObject =
  addon.encodePacketObject ?? addon.encode_packet_object;

if (typeof decodePacket !== 'function' || typeof encodePacketObject !== 'function') {
  throw new Error(
    'bindings.node does not export decodePacket and encodePacketObject',
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

const prettyPacket = (value) => JSON.stringify(
  value,
  (_key, v) => (typeof v === 'bigint' ? `__BINT__${v.toString()}` : v),
  2,
).replace(/"__BINT__(-?\d+)"/g, 'BigInt($1)');

const fail = (err) => {
  if (closed) {
    return;
  }
  closed = true;
  console.error(`[client] error: ${err instanceof Error ? err.stack || err.message : String(err)}`);
  process.exit(1);
};

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

    console.log(`[client] decoded packet #${packetCount + 1}:\n${prettyPacket(packet)}`);
    
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
