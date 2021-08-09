use std::fs::OpenOptions;
use std::io::{self, Read};
use std::path::Path;
use crcx::{Crc, CRC_32_ISO_HDLC};

const BUFFER_SIZE: usize = 1_048_576;

pub fn crc_hash<P: AsRef<Path>>(p: P) -> io::Result<String> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(p)?;
    let mut buf = [0u8; BUFFER_SIZE];

    let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    let mut hasher = crc.digest();
    loop {
        let bytes_read = file.read(&mut buf)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buf[..bytes_read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
