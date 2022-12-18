use core::panic;
use std::cmp;

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

fn keccak_f1600(a: &mut [u64; 25]) {
    // TODO: RHO (rotation offset table): Rho / Pi step table
    for round in 0..24 {
        // θ
        let mut c = [0u64; 5];
        for x in 0..5 {
            c[x] = a[5 * x] ^ a[5 * x + 1] ^ a[5 * x + 2] ^ a[5 * x + 3] ^ a[5 * x + 4];
            // Each lane xor-ed together
        }

        let mut d = [0u64; 5];
        for x in 0..5 {
            d[x] = c[(x + 4) % 5] ^ c[(x + 1) % 5].rotate_left(1); // cyclic indexing
        }

        // ρ and π
        let mut b = [[0u64; 4]; 4];
        for x in 0..5 {
            for y in 0..5 {
                b[y][(2 * x + 3 * y) % 4] = a[5 * x + y].rotate_left(RHO[5 * x + y]);
            }
        }

        // χ
        // ι
    }
}

const RHO: [u32; 24] = [
    1, 3, 6, 10, 15, 21, 28, 36, 45, 55, 2, 14, 27, 41, 56, 8, 25, 43, 62, 18, 39, 61, 20, 44,
];

const PI: [usize; 24] = [
    10, 7, 11, 17, 18, 3, 5, 16, 8, 21, 24, 4, 15, 23, 19, 13, 12, 2, 20, 14, 22, 9, 6, 1,
];

const WORDS: usize = 25;

fn keccak_f1600_copied(a: &mut [u64; 25]) {
    for i in 0..24 {
        let mut array: [u64; 5] = [0; 5];

        // Theta
        for x in 0..5 {
            for y_count in 0..5 {
                let y = y_count * 5;
                array[x] ^= a[x + y];
            }
        }

        for x in 0..5 {
            for y_count in 0..5 {
                let y = y_count * 5;
                a[y + x] ^= array[(x + 4) % 5] ^ array[(x + 1) % 5].rotate_left(1);
            }
        }

        // Rho and pi
        let mut last = a[1];
        for x in 0..24 {
            array[0] = a[PI[x]];
            a[PI[x]] = last.rotate_left(RHO[x]);
            last = array[0];
        }

        // Chi
        for y_step in 0..5 {
            let y = y_step * 5;

            for x in 0..5 {
                array[x] = a[y + x];
            }

            for x in 0..5 {
                a[y + x] = array[x] ^ ((!array[(x + 1) % 5]) & (array[(x + 2) % 5]));
            }
        }

        // Iota
        a[0] ^= RC[i];
    }
}

fn keccak_f1600_state(state: &mut [u8; 200]) {
    // Transmute to u64s
    let chunks = state.chunks(8);
    let mut lanes = [0u64; 25];
    let mut lane_index = 0;
    chunks.for_each(|chunk| {
        lanes[lane_index] = u64::from_le_bytes(chunk.try_into().unwrap());
        lane_index += 1;
    });

    keccak_f1600_copied(&mut lanes);

    // Transmute back to u8s
    lane_index = 0;
    lanes.into_iter().for_each(|lane| {
        lane.to_le_bytes().into_iter().for_each(|byte| {
            state[lane_index] = byte;
            lane_index += 1;
        });
    });
}

fn keccak(rate: usize, capacity: usize, input: &[u8], delimited_suffix: u8) -> [u8; 32] {
    // Notes
    // - 1600 means that it does 1600 bits at a time
    // - 1600 bits is 200 bytes at a time
    // - 1600 bits is divided into 64 lanes, or a 5x5 chunk of lanes
    // - rate + capacity == 1600
    // Plan
    // Take input bytes
    // Iterate through
    // State passed in to keccak is a total of 1600 bits [ <--rate bits--> | <--capacity bits--> ]
    // Pass this state into our 24 round permuter
    // Xor the state output with the next block
    // Re-run until all absorbed
    // Squeezing seems to typically be about extending the number of random bits beyond the "rate" amount

    let mut state = [0u8; 200];
    let byte_rate = rate / 8;

    if ((rate + capacity) != 1600) || ((rate % 8) != 0) {
        panic!("terrible params");
    }

    let mut input_offset = 0;
    let mut block_size = 0usize;
    while input_offset < input.len() {
        block_size = cmp::min(byte_rate, input.len() - input_offset);
        for i in 0..block_size {
            state[i] ^= input[i + input_offset];
        }
        input_offset += block_size;
        if block_size == byte_rate {
            keccak_f1600_state(&mut state);
            block_size = 0;
        }
    }

    state[block_size] = state[block_size] ^ delimited_suffix;
    state[byte_rate - 1] = state[byte_rate - 1] ^ 0x80;
    keccak_f1600_state(&mut state); // Fill block

    // Skip squeezing, only take first bytes
    let output: [u8; 32] = state[0..32]
        .try_into()
        .expect("state was not convertable to 32 byte array");
    output
}

pub fn keccak_256(input: &[u8]) -> [u8; 32] {
    keccak(1088, 512, input, 0x01) // Ethereum doesn't match the sha3 standard of 0x06 delimiter
}

fn bytes_to_lanes(bytes: [u8; 200]) -> [u64; 25] {
    let chunks = bytes.chunks(8);
    let mut lanes = [0u64; 25];
    let mut lane_index = 0;
    chunks.for_each(|chunk| {
        lanes[lane_index] = u64::from_le_bytes(chunk.try_into().unwrap());
        lane_index += 1;
    });
    lanes
}

fn lanes_to_bytes(lanes: [u64; 25]) -> [u8; 200] {
    let mut bytes = [0u8; 200];
    let mut byte_index = 0;

    lanes.into_iter().for_each(|lane| {
        lane.to_le_bytes().into_iter().for_each(|byte| {
            bytes[byte_index] = byte;
            byte_index += 1;
        });
    });
    bytes
}

#[cfg(test)]
mod tests {
    use crate::*;
    use rand::Rng;
    use tiny_keccak::Hasher;
    use tiny_keccak::Keccak;

    #[test]
    fn test_mini() {
        let hex = b"ABCD";
        let mut expected_hasher = Keccak::v256();
        expected_hasher.update(hex);
        let mut expected: [u8; 32] = [0; 32];
        expected_hasher.finalize(&mut expected);

        let actual = keccak_256(hex);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_short() {
        let hex = b"ABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCDABCD";
        let mut expected_hasher = Keccak::v256();
        expected_hasher.update(hex);
        let mut expected: [u8; 32] = [0; 32];
        expected_hasher.finalize(&mut expected);

        let actual = keccak_256(hex);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_long() {
        for i in 0..100 {
            let hex: Vec<u8> = (0..10_000).map(|_| rand::random::<u8>()).collect();
            let mut expected_hasher = Keccak::v256();
            expected_hasher.update(hex.as_slice());
            let mut expected: [u8; 32] = [0; 32];
            expected_hasher.finalize(&mut expected);

            let actual = keccak_256(hex.as_slice());

            assert_eq!(expected, actual);
        }
    }
}

// TODO: Benches
