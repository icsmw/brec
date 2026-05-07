use crate::*;
use brec_scheme::SchemeFile;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

pub struct NpmPackage<'a> {
    dir: PathBuf,
    scheme: &'a SchemeFile,
    generated: &'a GeneratedFiles<'a>,
    binding: PathBuf,
}

impl<'a> NpmPackage<'a> {
    pub fn new(
        dir: impl Into<PathBuf>,
        scheme: &'a SchemeFile,
        generated: &'a GeneratedFiles<'a>,
        binding: impl Into<PathBuf>,
    ) -> Self {
        Self {
            dir: dir.into(),
            scheme,
            generated,
            binding: binding.into(),
        }
    }

    pub fn write(&self) -> Result<(), Error> {
        fs::create_dir_all(&self.dir)?;
        fs::create_dir_all(self.dir.join("native"))?;

        self.generated.write_to(&self.dir)?;
        fs::write(self.dir.join("index.ts"), self.index_ts())?;
        fs::write(self.dir.join("index.js"), self.index_js())?;
        fs::write(self.dir.join("index.d.ts"), self.index_d_ts())?;
        fs::write(self.dir.join("package.json"), self.package_json()?)?;
        fs::copy(&self.binding, self.dir.join("native").join("bindings.node"))?;
        Ok(())
    }

    fn package_json(&self) -> Result<String, Error> {
        let package = json!({
            "name": self.scheme.package,
            "version": "0.1.0",
            "private": true,
            "main": "index.js",
            "types": "index.d.ts",
            "files": [
                "index.js",
                "index.ts",
                "index.d.ts",
                "blocks.ts",
                "payloads.ts",
                "packet.ts",
                "native/bindings.node"
            ],
            "devDependencies": {
                "typescript": "^5.0.0"
            }
        });
        Ok(format!("{}\n", serde_json::to_string_pretty(&package)?))
    }

    fn index_js(&self) -> &'static str {
        r#"'use strict';

const native = require('./native/bindings.node');

const pick = (camel, snake) => {
  const value = native[camel] || native[snake];
  if (typeof value !== 'function') {
    throw new Error(`bindings.node does not export ${camel}/${snake}`);
  }
  return value;
};

const decodeBlock = pick('decodeBlock', 'decode_block');
const encodeBlock = pick('encodeBlock', 'encode_block');
const decodePayload = pick('decodePayload', 'decode_payload');
const encodePayload = pick('encodePayload', 'encode_payload');
const decodePacket = pick('decodePacket', 'decode_packet');
const encodePacketObject = pick('encodePacketObject', 'encode_packet_object');
const encodePacketParts = pick('encodePacket', 'encode_packet');
const encodePacketFromJson = pick('encodePacketFromJson', 'encode_packet_from_json');

module.exports = {
  decodeBlock,
  encodeBlock,
  decodePayload,
  encodePayload,
  decodePacket,
  encodePacket: encodePacketObject,
  encodePacketObject,
  encodePacketParts,
  encodePacketFromJson,
};
"#
    }

    fn index_ts(&self) -> &'static str {
        r#"declare const require: any;

import type { Block } from './blocks';
import type { Payload } from './payloads';
import type { Packet } from './packet';

export * from './blocks';
export * from './payloads';
export * from './packet';

const native = require('./native/bindings.node');

const pick = (camel: string, snake: string): any => {
  const value = native[camel] || native[snake];
  if (typeof value !== 'function') {
    throw new Error(`bindings.node does not export ${camel}/${snake}`);
  }
  return value;
};

export const decodeBlock = pick('decodeBlock', 'decode_block') as (bytes: Uint8Array) => Block;
export const encodeBlock = pick('encodeBlock', 'encode_block') as (block: Block) => Uint8Array;
export const decodePayload = pick('decodePayload', 'decode_payload') as (bytes: Uint8Array) => Payload;
export const encodePayload = pick('encodePayload', 'encode_payload') as (payload: Payload) => Uint8Array;
export const decodePacket = pick('decodePacket', 'decode_packet') as (bytes: Uint8Array) => Packet;
export const encodePacketObject = pick('encodePacketObject', 'encode_packet_object') as (packet: Packet) => Uint8Array;
export const encodePacket = encodePacketObject;
export const encodePacketParts = pick('encodePacket', 'encode_packet') as (blocks: Block[], payload?: Payload) => Uint8Array;
export const encodePacketFromJson = pick('encodePacketFromJson', 'encode_packet_from_json') as (packetJson: string) => Uint8Array;
"#
    }

    fn index_d_ts(&self) -> &'static str {
        r#"import type { Block } from './blocks';
import type { Payload } from './payloads';
import type { Packet } from './packet';

export * from './blocks';
export * from './payloads';
export * from './packet';

export declare const decodeBlock: (bytes: Uint8Array) => Block;
export declare const encodeBlock: (block: Block) => Uint8Array;
export declare const decodePayload: (bytes: Uint8Array) => Payload;
export declare const encodePayload: (payload: Payload) => Uint8Array;
export declare const decodePacket: (bytes: Uint8Array) => Packet;
export declare const encodePacketObject: (packet: Packet) => Uint8Array;
export declare const encodePacket: (packet: Packet) => Uint8Array;
export declare const encodePacketParts: (blocks: Block[], payload?: Payload) => Uint8Array;
export declare const encodePacketFromJson: (packetJson: string) => Uint8Array;
"#
    }
}
