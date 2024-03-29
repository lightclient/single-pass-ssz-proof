use crate::CHUNK_SIZE;
use sha2::{Digest, Sha256};

pub fn hash(buf: &mut [u8; 2 * CHUNK_SIZE]) {
    let mut tmp = [0u8; CHUNK_SIZE];
    tmp.copy_from_slice(Sha256::digest(buf).as_ref());
    buf[0..CHUNK_SIZE].copy_from_slice(&tmp);
}

pub fn zh(mut depth: usize, buf: &mut [u8; 64]) {
    let mut tmp = [0u8; CHUNK_SIZE];
    buf[0..CHUNK_SIZE].copy_from_slice(&tmp);

    while depth > 0 {
        tmp.copy_from_slice(&buf[0..CHUNK_SIZE]);
        buf[CHUNK_SIZE..2 * CHUNK_SIZE].copy_from_slice(&tmp);
        hash(buf);
        depth -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_zero_hashes() {
        let zh0 = "0000000000000000000000000000000000000000000000000000000000000000";
        let zh1 = "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b";
        let zh2 = "db56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71";
        let zh3 = "c78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c";

        let mut buf = [0u8; 64];
        zh(0, &mut buf);
        assert_eq!(zh0, hex::encode(&buf[0..CHUNK_SIZE]));

        zh(1, &mut buf);
        assert_eq!(zh1, hex::encode(&buf[0..CHUNK_SIZE]));

        zh(2, &mut buf);
        assert_eq!(zh2, hex::encode(&buf[0..CHUNK_SIZE]));

        zh(3, &mut buf);
        assert_eq!(zh3, hex::encode(&buf[0..CHUNK_SIZE]));
    }
}

pub fn generate_branch(height: usize, chunks: &mut [u8]) {
    if height > chunks.len() / CHUNK_SIZE - 1 {
        panic!("chunks buffer too small: {}", chunks.len());
    }

    let mut buf = [0u8; 64];
    for i in 2..(height + 1) {
        zh(i - 1, &mut buf);

        let begin = i * CHUNK_SIZE;
        let end = begin + CHUNK_SIZE;
        chunks[begin..end].copy_from_slice(&buf[0..CHUNK_SIZE]);
    }
}
