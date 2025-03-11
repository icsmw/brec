use brec::prelude::*;
use proptest::prelude::*;

use crate::*;

brec::include_generated!("crate::*");

impl Arbitrary for Block {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        prop_oneof![
            BlockU8::arbitrary().prop_map(Block::BlockU8),
            BlockU16::arbitrary().prop_map(Block::BlockU16),
            BlockU32::arbitrary().prop_map(Block::BlockU32),
            BlockU64::arbitrary().prop_map(Block::BlockU64),
            BlockU128::arbitrary().prop_map(Block::BlockU128),
            BlockI8::arbitrary().prop_map(Block::BlockI8),
            BlockI16::arbitrary().prop_map(Block::BlockI16),
            BlockI32::arbitrary().prop_map(Block::BlockI32),
            BlockI64::arbitrary().prop_map(Block::BlockI64),
            BlockI128::arbitrary().prop_map(Block::BlockI128),
            BlockF32::arbitrary().prop_map(Block::BlockF32),
            BlockF64::arbitrary().prop_map(Block::BlockF64),
            BlockBool::arbitrary().prop_map(Block::BlockBool),
            BlockBlob::arbitrary().prop_map(Block::BlockBlob),
            BlockBlobs::arbitrary().prop_map(Block::BlockBlobs),
            BlockEnums::arbitrary().prop_map(Block::BlockEnums),
            BlockCombination::arbitrary().prop_map(Block::BlockCombination),
        ]
        .boxed()
    }
}

fn write_to_buf<W: std::io::Write>(buf: &mut W, blks: &[Block]) -> std::io::Result<()> {
    for blk in blks.iter() {
        WriteTo::write(blk, buf)?;
    }
    Ok(())
}

fn read_blocks(buffer: &[u8]) -> std::io::Result<(Vec<Block>, u64)> {
    use std::io::{BufReader, Cursor, Seek};

    let mut blocks = Vec::new();
    let mut reader = BufReader::new(Cursor::new(buffer));
    let mut consumed = 0;
    loop {
        match <Block as TryReadFrom>::try_read(&mut reader) {
            Ok(ReadStatus::Success(blk)) => {
                consumed = reader.stream_position()?;
                blocks.push(blk);
            }
            Ok(ReadStatus::NotEnoughData(_needed)) => {
                break;
            }
            Err(err) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    err.to_string(),
                ));
            }
        }
    }
    Ok((blocks, consumed))
}

fn read_blocks_from_buffered(buffer: &[u8]) -> std::io::Result<(Vec<Block>, usize)> {
    use brec::BufferedReader;
    use std::io::Cursor;

    let mut inner = Cursor::new(buffer);
    let mut reader = BufferedReader::new(&mut inner);
    let mut blocks = Vec::new();
    loop {
        if reader.buffer_len().unwrap() < 4 {
            reader.refill().unwrap();
        }
        match <Block as TryReadFromBuffered>::try_read(&mut reader) {
            Ok(ReadStatus::Success(blk)) => {
                blocks.push(blk);
            }
            Ok(ReadStatus::NotEnoughData(_needed)) => {
                reader.refill()?;
                break;
            }
            Err(err) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    err.to_string(),
                ));
            }
        }
    }
    Ok((blocks, reader.consumed()))
}

use std::sync::atomic::{AtomicUsize, Ordering};

static BYTES: AtomicUsize = AtomicUsize::new(0);
static INSTANCES: AtomicUsize = AtomicUsize::new(0);

fn report(bytes: usize, instance: usize) {
    use num_format::{Locale, ToFormattedString};

    BYTES.fetch_add(bytes, Ordering::Relaxed);
    INSTANCES.fetch_add(instance, Ordering::Relaxed);
    let bytes = BYTES.load(Ordering::Relaxed);
    println!(
        "Generated {} blocks ({}, {} B)",
        INSTANCES
            .load(Ordering::Relaxed)
            .to_formatted_string(&Locale::en),
        if bytes > 1024 * 1024 {
            format!(
                "{} Mb",
                (bytes / (1024 * 1024)).to_formatted_string(&Locale::en)
            )
        } else if bytes > 1024 {
            format!(
                "{} Kb",
                (bytes / (1024 * 1024)).to_formatted_string(&Locale::en)
            )
        } else {
            format!("{} B", bytes.to_formatted_string(&Locale::en))
        },
        bytes.to_formatted_string(&Locale::en)
    );
}

proptest! {
    #![proptest_config(ProptestConfig {
        max_shrink_iters: 50,
        ..ProptestConfig::with_cases(500)
    })]


    #[test]
    fn try_read_from(blks in proptest::collection::vec(any::<Block>(), 1..2000)) {
        let mut buf = Vec::new();
        write_to_buf(&mut buf, &blks)?;
        let size = buf.len() as u64;
        let (restored, consumed) = read_blocks(&buf)?;
        assert_eq!(size, consumed);
        assert_eq!(blks.len(), restored.len());
        for (left, right) in restored.iter().zip(blks.iter()) {
            assert_eq!(left, right);
        }
        report(buf.len(), restored.len());
    }

    #[test]
    fn try_read_from_buffered(blks in proptest::collection::vec(any::<Block>(), 1..2000)) {
        let mut buf = Vec::new();
        write_to_buf(&mut buf, &blks)?;
        let write = buf.len() as u64;
        let (restored, read) = read_blocks_from_buffered(&buf)?;
        assert_eq!(write, read as u64);
        assert_eq!(blks.len(), restored.len());
        for (left, right) in restored.iter().zip(blks.iter()) {
            assert_eq!(left, right);
        }
        report(buf.len(), restored.len());
    }

    #[test]
    fn try_read_from_slice(blks in proptest::collection::vec(any::<Block>(), 1..2000)) {
        let mut bytes = 0;
        for blk in blks.iter() {
            let mut buf = Vec::new();
            blk.write_all(&mut buf)?;
            bytes += buf.len();
            let refered = <BlockReferred as ReadBlockFromSlice>::read_from_slice(&buf, false)?;
            let restored: Block = refered.into();
            assert_eq!(blk, &restored);
        }
        report(bytes, blks.len());
    }

}
