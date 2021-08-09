use std::fs::OpenOptions;
use std::io::{self, Read};
use std::path::Path;
use md4::{Md4, Digest};

const CHUNK_SIZE: usize = 9_728_000;
const BUFFER_SIZE: usize = 1_048_576;

pub fn ed2k_hash<P: AsRef<Path>>(p: P) -> io::Result<(String, usize)> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(p)?;

    let mut buf = [0u8; BUFFER_SIZE];
    let mut hashlist: Vec<u8> = vec![];
    let mut hasher = Md4::new();
    let mut chunk_bytes_read = 0;
    let mut total_bytes_read = 0;
    loop {
        let bytes_read = file.read(&mut buf)?;
        if bytes_read == 0 {
            break;
        }
        if chunk_bytes_read + bytes_read > CHUNK_SIZE {
            let take = CHUNK_SIZE - chunk_bytes_read;
            hasher.update(&buf[..take]);
            hashlist.extend(hasher.finalize_reset());
            hasher.update(&buf[take..bytes_read]);
            chunk_bytes_read = bytes_read - take;
        } else {
            hasher.update(&buf[..bytes_read]);
            chunk_bytes_read += bytes_read;
        }
        total_bytes_read += bytes_read;
    }
    if hashlist.len() > 0 {
        hashlist.extend(hasher.finalize_reset());
        hasher.update(&hashlist);
    }
    Ok((hex::encode(hasher.finalize()), total_bytes_read))
}
