use crate::test::*;
use std::time::Duration;

pub fn create_file(
    packets: Vec<WrappedPacket>,
    mut count: usize,
    filename: &str,
) -> std::io::Result<()> {
    let tmp = std::env::temp_dir().join(filename);
    if tmp.exists() {
        return Ok(());
    }
    let mut storage = FileStorage::new(tmp, Some(Duration::from_millis(400)), None)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    while count > 0 {
        for packet in packets.iter() {
            storage.insert(packet.into()).map_err(|err| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
            })?;
        }
        count -= 1;
    }
    Ok(())
}

pub fn read_file(filename: &str) -> std::io::Result<()> {
    let tmp = std::env::temp_dir().join(filename);
    let file = std::fs::OpenOptions::new().read(true).open(&tmp)?;
    let mut reader = Reader::new(&file)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    let mut count = 0;
    for packet in reader.iter() {
        match packet {
            Ok(_packet) => {
                count += 1;
            }
            Err(err) => {
                panic!("Fail to read storage: {err}");
            }
        }
    }
    if count != reader.count() {
        return Err(std::io::Error::other(format!(
            "Dismatch lengths: {} vs {count}",
            reader.count()
        )));
    }
    Ok(())
}
