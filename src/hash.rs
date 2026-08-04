use sha2::{Digest, Sha256};

fn hex_from_u8(x: u8) -> char {
    if x < 10 {
        (b'0' + x) as char
    }
    else {
        (b'A' + x - 10) as char
    }
}

fn hex(buf: &mut String, blob: &[u8]) {
    for x in blob {
        buf.push(hex_from_u8(x / 16));
        buf.push(hex_from_u8(x % 16));
    }
}

pub fn sha256_digest_alloc(content: &[u8]) -> String {

    // important(perf): Update this alongside the algorithm should it ever change.
    const DIGEST_SIZE: usize = 256;
    let digest = Sha256::digest(content);

    let mut hash_str = String::with_capacity(2 * DIGEST_SIZE / 8);
    hex(&mut hash_str, digest.as_slice());

    hash_str
}
