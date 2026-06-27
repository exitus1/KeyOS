// SPDX-License-Identifier: Apache-2.0
//! BLAKE-256 (the SHA-3 finalist, 14 rounds) — NOT BLAKE2/BLAKE3.
//!
//! Decred hashes *everything* with this: transaction IDs, sighashes, address
//! Hash160 (`ripemd160(blake256(x))`) and the base58check checksum
//! (`blake256(blake256(x))[..4]`). It is the single most load-bearing
//! primitive in this crate, so it is vendored here and pinned by a direct
//! known-answer test (`blake256("") == 716f6e86…`) plus the transitive dcrd
//! address/HD reference vectors in `tests/vectors.rs`.
//!
//! Reference: dcrd `crypto/blake256` (Go), original BLAKE spec.

pub const OUT_LEN: usize = 32;
const BLOCK_LEN: usize = 64;

const IV: [u32; 8] = [
    0x6a09_e667, 0xbb67_ae85, 0x3c6e_f372, 0xa54f_f53a,
    0x510e_527f, 0x9b05_688c, 0x1f83_d9ab, 0x5be0_cd19,
];

// Fractional bits of pi — the BLAKE round constants.
const C: [u32; 16] = [
    0x243f_6a88, 0x85a3_08d3, 0x1319_8a2e, 0x0370_7344,
    0xa409_3822, 0x299f_31d0, 0x082e_fa98, 0xec4e_6c89,
    0x4528_21e6, 0x38d0_1377, 0xbe54_66cf, 0x34e9_0c6c,
    0xc0ac_29b7, 0xc97c_50dd, 0x3f84_d5b5, 0xb547_0917,
];

const SIGMA: [[usize; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

struct State {
    h: [u32; 8],
    t: u64, // bits hashed so far
}

impl State {
    fn new() -> Self {
        State { h: IV, t: 0 }
    }

    /// Compress one 64-byte block. `counter` is the total bit length that this
    /// block commits to (0 for a padding-only final block — the BLAKE edge case).
    fn compress(&mut self, block: &[u8; BLOCK_LEN], counter: u64) {
        let mut m = [0u32; 16];
        for i in 0..16 {
            m[i] = u32::from_be_bytes([
                block[i * 4], block[i * 4 + 1], block[i * 4 + 2], block[i * 4 + 3],
            ]);
        }

        let t0 = counter as u32;
        let t1 = (counter >> 32) as u32;

        // salt is always zero in Decred's usage.
        let mut v = [0u32; 16];
        v[..8].copy_from_slice(&self.h);
        v[8] = C[0];
        v[9] = C[1];
        v[10] = C[2];
        v[11] = C[3];
        v[12] = C[4] ^ t0;
        v[13] = C[5] ^ t0;
        v[14] = C[6] ^ t1;
        v[15] = C[7] ^ t1;

        #[inline(always)]
        fn g(v: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize, x: u32, y: u32) {
            v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
            v[d] = (v[d] ^ v[a]).rotate_right(16);
            v[c] = v[c].wrapping_add(v[d]);
            v[b] = (v[b] ^ v[c]).rotate_right(12);
            v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
            v[d] = (v[d] ^ v[a]).rotate_right(8);
            v[c] = v[c].wrapping_add(v[d]);
            v[b] = (v[b] ^ v[c]).rotate_right(7);
        }

        for r in 0..14 {
            let s = &SIGMA[r % 10];
            // Each G mixes in m[s[2i]] ^ C[s[2i+1]] then m[s[2i+1]] ^ C[s[2i]].
            g(&mut v, 0, 4, 8, 12, m[s[0]] ^ C[s[1]], m[s[1]] ^ C[s[0]]);
            g(&mut v, 1, 5, 9, 13, m[s[2]] ^ C[s[3]], m[s[3]] ^ C[s[2]]);
            g(&mut v, 2, 6, 10, 14, m[s[4]] ^ C[s[5]], m[s[5]] ^ C[s[4]]);
            g(&mut v, 3, 7, 11, 15, m[s[6]] ^ C[s[7]], m[s[7]] ^ C[s[6]]);
            g(&mut v, 0, 5, 10, 15, m[s[8]] ^ C[s[9]], m[s[9]] ^ C[s[8]]);
            g(&mut v, 1, 6, 11, 12, m[s[10]] ^ C[s[11]], m[s[11]] ^ C[s[10]]);
            g(&mut v, 2, 7, 8, 13, m[s[12]] ^ C[s[13]], m[s[13]] ^ C[s[12]]);
            g(&mut v, 3, 4, 9, 14, m[s[14]] ^ C[s[15]], m[s[15]] ^ C[s[14]]);
        }

        for i in 0..8 {
            // salt == 0, so the salt XOR terms vanish.
            self.h[i] ^= v[i] ^ v[i + 8];
        }
    }
}

/// One-shot BLAKE-256.
pub fn sum256(data: &[u8]) -> [u8; OUT_LEN] {
    let mut st = State::new();
    let mut chunks = data.chunks_exact(BLOCK_LEN);
    let mut consumed_bits: u64 = 0;
    for blk in &mut chunks {
        consumed_bits += (BLOCK_LEN as u64) * 8;
        let mut b = [0u8; BLOCK_LEN];
        b.copy_from_slice(blk);
        st.compress(&b, consumed_bits);
    }

    let rem = chunks.remainder();
    let total_bits = (data.len() as u64) * 8;

    // Padding: 0x80, zeros, a 0x01 terminator bit before the 64-bit BE length.
    // If the remainder leaves no room for the 9 trailing bytes, emit two blocks.
    let mut last = [0u8; BLOCK_LEN];
    last[..rem.len()].copy_from_slice(rem);
    last[rem.len()] = 0x80;

    if rem.len() <= 55 {
        // Single final block holds data bits + length → counter = total_bits.
        last[55] |= 0x01;
        last[56..].copy_from_slice(&total_bits.to_be_bytes());
        // counter must reflect the actual message bits in THIS block.
        let counter = if rem.is_empty() && data.len() % BLOCK_LEN == 0 && !data.is_empty() {
            0
        } else {
            total_bits
        };
        st.compress(&last, counter);
    } else {
        // First padding block carries the remaining data bits (counter = total),
        // second block is length-only (counter = 0).
        st.compress(&last, total_bits);
        let mut tail = [0u8; BLOCK_LEN];
        tail[55] |= 0x01;
        tail[56..].copy_from_slice(&total_bits.to_be_bytes());
        st.compress(&tail, 0);
    }

    let mut out = [0u8; OUT_LEN];
    for i in 0..8 {
        out[i * 4..i * 4 + 4].copy_from_slice(&st.h[i].to_be_bytes());
    }
    out
}

/// `blake256(blake256(x))` — used by the base58check checksum and TxHash.
pub fn sum256d(data: &[u8]) -> [u8; OUT_LEN] {
    sum256(&sum256(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    #[test]
    fn empty_string_kat() {
        // Canonical BLAKE-256 known-answer test.
        assert_eq!(
            hex(&sum256(b"")),
            "716f6e863f744b9ac22c97ec7b76ea5f5908bc5b2f67c61510bfc4751384ea7a"
        );
    }

    #[test]
    fn single_zero_byte_kat() {
        assert_eq!(
            hex(&sum256(&[0u8])),
            "0ce8d4ef4dd7cd8d62dfded9d4edb0a774ae6a41929a74da23109e8f11139c87"
        );
    }
}
