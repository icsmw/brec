use super::file::{JavaFile, JavaPackage, write_imports};
use crate::*;

pub(super) struct PacketFile<'a> {
    model: &'a Model,
}

impl<'a> PacketFile<'a> {
    pub(super) fn new(model: &'a Model) -> Self {
        Self { model }
    }

    pub(super) fn file(self) -> Result<JavaFile, Error> {
        JavaFile::new(self.model, JavaPackage::Root, "Packet.java", |writer| {
            write_imports(
                writer,
                &[
                    "com.icsmw.brec.block.Block",
                    "com.icsmw.brec.payload.Payload",
                    "java.util.ArrayList",
                    "java.util.HashMap",
                    "java.util.List",
                    "java.util.Map",
                    "java.util.Objects",
                ],
            )?;
            writer.block(
                r#"
public final class Packet {
	public List<Block> blocks = new ArrayList<>(0);
	public Payload payload;

	public Packet() {}

	public Packet(List<Block> blocks, Payload payload) {
		this.blocks = blocks == null ? new ArrayList<>(0) : blocks;
		this.payload = payload;
	}

	public static Packet withoutPayload(List<Block> blocks) {
		return new Packet(blocks, null);
	}

	static Packet fromBrecObject(Object value) {
		Map<?, ?> map = (Map<?, ?>) value;
		List<?> rawBlocks = (List<?>) map.get("blocks");
		List<Block> blocks = new ArrayList<>(rawBlocks.size());
		for (Object block : rawBlocks) {
			blocks.add(Block.fromBrecObject(block));
		}
		Object payload = map.get("payload");
		return new Packet(blocks, payload == null ? null : Payload.fromBrecObject(payload));
	}

	Map<String, Object> toBrecObject() {
		HashMap<String, Object> out = new HashMap<>(2);
		ArrayList<Object> encodedBlocks = new ArrayList<>(blocks.size());
		for (Block block : blocks) {
			encodedBlocks.add(block.toBrecObject());
		}
		out.put("blocks", encodedBlocks);
		out.put("payload", payload == null ? null : payload.toBrecObject());
		return out;
	}

	@Override
	public boolean equals(Object other) {
		if (!(other instanceof Packet)) {
			return false;
		}
		Packet that = (Packet) other;
		return Objects.equals(blocks, that.blocks) && Objects.equals(payload, that.payload);
	}

	@Override
	public int hashCode() {
		int result = Objects.hashCode(blocks);
		result = 31 * result + Objects.hashCode(payload);
		return result;
	}
}
"#,
            )
        })
    }
}
