use sha2::{Digest, Sha256};

// important(perf): Update this alongside the algorithm should it ever change.
const DIGEST_SIZE: usize = 256;

// * 2 -> One byte results in 2 hex chars.
// / 8 -> The `DIGEST_SIZE` is measured in bits, not bytes.
pub const EFFECTIVE_BUF_SIZE: usize = DIGEST_SIZE * 2 / 8;

fn hex_from_u8(x: u8) -> char {
    if x < 10 {
        (b'0' + x) as char
    } else {
        (b'A' + x - 10) as char
    }
}

fn hex(buf: &mut String, blob: &[u8]) -> usize {
    let mut written = 0usize;
    for x in blob {
        buf.push(hex_from_u8(x / 16));
        buf.push(hex_from_u8(x % 16));
        written += 2;
    }

    written
}

pub fn sha256_digest_alloc(content: &[u8]) -> String {
    let digest = Sha256::digest(content);
    let mut hash_str = String::with_capacity(EFFECTIVE_BUF_SIZE);

    #[allow(unused_variables)] // due to this only being used in debug mode
    let written = hex(&mut hash_str, digest.as_slice());

    #[cfg(debug_assertions)]
    if written != EFFECTIVE_BUF_SIZE {
        panic!(
            r#"perf:hash/hex-string: The String buffer holding a hex-encoded digest hasn't received the pre-allocated amount of bytes.
It is thereby wasting space by either over-allocating, or wasting both space and performance by causing re-allocations."#
        );
    }

    hash_str
}
