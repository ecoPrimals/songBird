//! Minimal pure Rust SHA3-256 (Keccak-f\[1600\])
//!
//! Zero external dependencies. Used for onion address checksum verification
//! and descriptor ID computation where BearDog is not required.
//!
//! Implements FIPS 202 (SHA-3) using the Keccak sponge construction.
//! Only SHA3-256 is provided as that's all Tor v3 requires.

/// SHA3-256 hash function (pure Rust, zero dependencies)
///
/// Returns the 32-byte SHA3-256 digest of the input.
///
/// # Example
///
/// ```
/// use songbird_tor_protocol::crypto::sha3::sha3_256;
///
/// let hash = sha3_256(b"hello");
/// assert_eq!(hash.len(), 32);
/// ```
pub fn sha3_256(data: &[u8]) -> [u8; 32] {
    let mut state = KeccakState::new();
    state.absorb(data);
    state.squeeze()
}

/// Keccak-f[1600] state (5x5 matrix of u64)
struct KeccakState {
    state: [u64; 25],
    /// Buffer for absorbing (rate = 136 bytes for SHA3-256)
    buffer: Vec<u8>,
}

/// SHA3-256 rate in bytes (1600 - 2*256) / 8 = 136
const RATE: usize = 136;

/// Keccak-f[1600] round constants
const RC: [u64; 24] = [
    0x0000000000000001,
    0x0000000000008082,
    0x800000000000808A,
    0x8000000080008000,
    0x000000000000808B,
    0x0000000080000001,
    0x8000000080008081,
    0x8000000000008009,
    0x000000000000008A,
    0x0000000000000088,
    0x0000000080008009,
    0x000000008000000A,
    0x000000008000808B,
    0x800000000000008B,
    0x8000000000008089,
    0x8000000000008003,
    0x8000000000008002,
    0x8000000000000080,
    0x000000000000800A,
    0x800000008000000A,
    0x8000000080008081,
    0x8000000000008080,
    0x0000000080000001,
    0x8000000080008008,
];

/// Rotation offsets for Keccak rho step
const ROTATIONS: [u32; 25] =
    [0, 1, 62, 28, 27, 36, 44, 6, 55, 20, 3, 10, 43, 25, 39, 41, 45, 15, 21, 8, 18, 2, 61, 56, 14];

/// Pi step permutation indices
const PI: [usize; 25] =
    [0, 10, 20, 5, 15, 16, 1, 11, 21, 6, 7, 17, 2, 12, 22, 23, 8, 18, 3, 13, 14, 24, 9, 19, 4];

impl KeccakState {
    fn new() -> Self {
        Self {
            state: [0u64; 25],
            buffer: Vec::with_capacity(RATE),
        }
    }

    /// Absorb data into the sponge
    fn absorb(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);

        while self.buffer.len() >= RATE {
            let block: Vec<u8> = self.buffer.drain(..RATE).collect();
            self.xor_block(&block);
            self.keccak_f();
        }
    }

    /// Squeeze 32 bytes from the sponge (SHA3-256 output)
    fn squeeze(&mut self) -> [u8; 32] {
        // SHA3 padding: append 0x06, pad with zeros, set last bit
        let mut padded = self.buffer.clone();
        padded.push(0x06); // SHA3 domain separation
        while padded.len() < RATE {
            padded.push(0x00);
        }
        padded[RATE - 1] |= 0x80; // Set last bit

        self.xor_block(&padded);
        self.keccak_f();

        // Extract 32 bytes from state
        let mut output = [0u8; 32];
        for i in 0..4 {
            output[i * 8..(i + 1) * 8].copy_from_slice(&self.state[i].to_le_bytes());
        }
        output
    }

    /// XOR a rate-sized block into the state
    #[allow(clippy::unwrap_used)] // slice length is bounds-checked by the `if` guard
    fn xor_block(&mut self, block: &[u8]) {
        for i in 0..(RATE / 8) {
            if i * 8 + 8 <= block.len() {
                let word = u64::from_le_bytes(block[i * 8..i * 8 + 8].try_into().unwrap());
                self.state[i] ^= word;
            }
        }
    }

    /// Keccak-f\[1600\] permutation (24 rounds)
    fn keccak_f(&mut self) {
        for &rc in &RC {
            // θ (theta)
            let mut c = [0u64; 5];
            for (x, c_val) in c.iter_mut().enumerate() {
                *c_val = self.state[x]
                    ^ self.state[x + 5]
                    ^ self.state[x + 10]
                    ^ self.state[x + 15]
                    ^ self.state[x + 20];
            }
            let mut d = [0u64; 5];
            for (x, d_val) in d.iter_mut().enumerate() {
                *d_val = c[(x + 4) % 5] ^ c[(x + 1) % 5].rotate_left(1);
            }
            for i in 0..25 {
                self.state[i] ^= d[i % 5];
            }

            // ρ (rho) and π (pi)
            let mut temp = [0u64; 25];
            for i in 0..25 {
                temp[PI[i]] = self.state[i].rotate_left(ROTATIONS[i]);
            }

            // χ (chi)
            for y in 0..5 {
                let base = y * 5;
                let t0 = temp[base];
                let t1 = temp[base + 1];
                let t2 = temp[base + 2];
                let t3 = temp[base + 3];
                let t4 = temp[base + 4];
                self.state[base] = t0 ^ (!t1 & t2);
                self.state[base + 1] = t1 ^ (!t2 & t3);
                self.state[base + 2] = t2 ^ (!t3 & t4);
                self.state[base + 3] = t3 ^ (!t4 & t0);
                self.state[base + 4] = t4 ^ (!t0 & t1);
            }

            // ι (iota)
            self.state[0] ^= rc;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha3_256_empty() {
        // SHA3-256("") = a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a
        let hash = sha3_256(b"");
        assert_eq!(hash[0], 0xa7);
        assert_eq!(hash[1], 0xff);
        assert_eq!(hash[2], 0xc6);
        assert_eq!(hash[31], 0x4a);
    }

    #[test]
    fn test_sha3_256_abc() {
        // SHA3-256("abc") = 3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532
        let hash = sha3_256(b"abc");
        assert_eq!(hash[0], 0x3a);
        assert_eq!(hash[1], 0x98);
        assert_eq!(hash[2], 0x5d);
        assert_eq!(hash[31], 0x32);
    }

    #[test]
    fn test_sha3_256_deterministic() {
        let h1 = sha3_256(b"test data");
        let h2 = sha3_256(b"test data");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_sha3_256_different_inputs() {
        let h1 = sha3_256(b"hello");
        let h2 = sha3_256(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_sha3_256_onion_checksum() {
        // Test the exact pattern used for .onion address checksums
        let pubkey = [0u8; 32];
        let mut input = Vec::new();
        input.extend_from_slice(b".onion checksum");
        input.extend_from_slice(&pubkey);
        input.push(0x03);

        let hash = sha3_256(&input);
        let checksum = &hash[..2];
        // Just verify it produces a consistent 2-byte checksum
        assert_eq!(checksum.len(), 2);

        // Same input = same checksum
        let hash2 = sha3_256(&input);
        assert_eq!(&hash2[..2], checksum);
    }

    #[test]
    fn test_sha3_256_long_input() {
        // Input larger than one block (136 bytes)
        let data = vec![0xABu8; 200];
        let hash = sha3_256(&data);
        assert_eq!(hash.len(), 32);
        // Non-trivial output
        assert!(hash.iter().any(|&b| b != 0));
    }
}
