const BLOCK_SIZE: usize = 16;
use rand::Rng;
pub fn xor_bytes(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| x ^ y)
        .collect()
}

pub fn xor_block_bytes(block1: &[u8; BLOCK_SIZE], block2: &[u8; BLOCK_SIZE]) -> [u8; BLOCK_SIZE] {
    let mut xored = [0u8; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        xored[i] = block1[i] ^ block2[i];
    }
    xored
}

pub fn create_rand_init_vector() -> [u8; BLOCK_SIZE] {
    let mut rand_init_vector = [0u8; BLOCK_SIZE];
    rand::thread_rng().fill(&mut rand_init_vector);
    rand_init_vector
}
