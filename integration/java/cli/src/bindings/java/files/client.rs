use super::file::{JavaFile, JavaPackage};
use crate::*;

pub(super) struct ClientFile<'a> {
    model: &'a Model,
}

impl<'a> ClientFile<'a> {
    pub(super) fn new(model: &'a Model) -> Self {
        Self { model }
    }

    pub(super) fn file(self) -> Result<JavaFile, Error> {
        JavaFile::new(self.model, JavaPackage::Root, "Client.java", |writer| {
            writer.block(
                r#"
import com.icsmw.brec.block.Block;
import com.icsmw.brec.payload.Payload;

public final class Client {
	static {
		System.loadLibrary("bindings");
	}

	private Client() {}

	private static native Object decodeBlockNative(byte[] bytes);
	private static native byte[] encodeBlockNative(Object block);
	private static native Object decodePayloadNative(byte[] bytes);
	private static native byte[] encodePayloadNative(Object payload);
	private static native Object decodePacketNative(byte[] bytes);
	private static native byte[] encodePacketNative(Object packet);

	public static Block decodeBlock(byte[] bytes) {
		return Block.fromBrecObject(decodeBlockNative(bytes));
	}

	public static byte[] encodeBlock(Block block) {
		return encodeBlockNative(block.toBrecObject());
	}

	public static Payload decodePayload(byte[] bytes) {
		return Payload.fromBrecObject(decodePayloadNative(bytes));
	}

	public static byte[] encodePayload(Payload payload) {
		return encodePayloadNative(payload.toBrecObject());
	}

	public static Packet decodePacket(byte[] bytes) {
		return Packet.fromBrecObject(decodePacketNative(bytes));
	}

	public static byte[] encodePacket(Packet packet) {
		return encodePacketNative(packet.toBrecObject());
	}
}
"#,
            )
        })
    }
}
