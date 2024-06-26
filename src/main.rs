//! In Module 1, we discussed Block ciphers like AES. Block ciphers have a fixed length input.
//! Real wold data that we wish to encrypt _may_ be exactly the right length, but is probably not.
//! When your data is too short, you can simply pad it up to the correct length.
//! When your data is too long, you have some options.
//!
//! In this exercise, we will explore a few of the common ways that large pieces of data can be
//! broken up and combined in order to encrypt it with a fixed-length block cipher.
//!
//! WARNING: ECB MODE IS NOT SECURE.
//! Seriously, ECB is NOT secure. Don't use it irl. We are implementing it here to understand _why_
//! it is not secure and make the point that the most straight-forward approach isn't always the
//! best, and can sometimes be trivially broken.

use aes::{
    cipher::{generic_array::GenericArray, BlockCipher, BlockDecrypt, BlockEncrypt, KeyInit},
    Aes128,
};
mod utils;

///We're using AES 128 which has 16-byte (128 bit) blocks.
const BLOCK_SIZE: usize = 16;
const NONCE_SIZE: usize = 8;

fn main() {
    todo!("Maybe this should be a library crate. TBD");
}

/// Simple AES encryption
/// Helper function to make the core AES block cipher easier to understand.
fn aes_encrypt(data: [u8; BLOCK_SIZE], key: &[u8; BLOCK_SIZE]) -> [u8; BLOCK_SIZE] {
    // Convert the inputs to the necessary data type
    let mut block = GenericArray::from(data);
    let key = GenericArray::from(*key);

    let cipher = Aes128::new(&key);

    cipher.encrypt_block(&mut block);

    block.into()
}

/// Simple AES encryption
/// Helper function to make the core AES block cipher easier to understand.
fn aes_decrypt(data: [u8; BLOCK_SIZE], key: &[u8; BLOCK_SIZE]) -> [u8; BLOCK_SIZE] {
    // Convert the inputs to the necessary data type
    let mut block = GenericArray::from(data);
    let key = GenericArray::from(*key);

    let cipher = Aes128::new(&key);

    cipher.decrypt_block(&mut block);

    block.into()
}

/// Before we can begin encrypting our raw data, we need it to be a multiple of the
/// block length which is 16 bytes (128 bits) in AES128.
///
/// The padding algorithm here is actually not trivial. The trouble is that if we just
/// naively throw a bunch of zeros on the end, there is no way to know, later, whether
/// those zeros are padding, or part of the message, or some of each.
///
/// The scheme works like this. If the data is not a multiple of the block length,  we
/// compute how many pad bytes we need, and then write that number into the last several bytes.
/// Later we look at the last byte, and remove that number of bytes.
///
/// But if the data _is_ a multiple of the block length, then we have a problem. We don't want
/// to later look at the last byte and remove part of the data. Instead, in this case, we add
/// another entire block containing the block length in each byte. In our case,
/// [16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16]
fn pad(mut data: Vec<u8>) -> Vec<u8> {
    let number_bytes_to_pad = BLOCK_SIZE - (data.len() % BLOCK_SIZE);

    for _ in 0..number_bytes_to_pad {
        data.push(number_bytes_to_pad as u8);
    }

    data
}

/// Groups the data into BLOCK_SIZE blocks. Assumes the data is already
/// a multiple of the block size. If this is not the case, call `pad` first.
fn group(data: Vec<u8>) -> Vec<[u8; BLOCK_SIZE]> {
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let mut block: [u8; BLOCK_SIZE] = Default::default();
        block.copy_from_slice(&data[i..i + BLOCK_SIZE]);
        blocks.push(block);

        i += BLOCK_SIZE;
    }

    blocks
}

/// Does the opposite of the group function
fn un_group(blocks: Vec<[u8; BLOCK_SIZE]>) -> Vec<u8> {
    blocks.iter().flat_map(|&block| block.to_vec()).collect()
}

/// Does the opposite of the pad function.
fn un_pad(mut data: Vec<u8>) -> Vec<u8> {
    if let Some(&pad_len) = data.last() {
        let len = data.len();
        if (pad_len as usize) <= len && (pad_len as usize) <= BLOCK_SIZE {
            data.truncate(len - (pad_len as usize));
        }
    }
    data
}

/// The first mode we will implement is the Electronic Code Book, or ECB mode.
/// Warning: THIS MODE IS NOT SECURE!!!!
///
/// This is probably the first thing you think of when considering how to encrypt
/// large data. In this mode we simply encrypt each block of data under the same key.
/// One good thing about this mode is that it is parallelizable. But to see why it is
/// insecure look at: https://www.ubiqsecurity.com/wp-content/uploads/2022/02/ECB2.png
fn ecb_encrypt(plain_text: Vec<u8>, key: [u8; 16]) -> Vec<u8> {
    let padded_text = pad(plain_text);

    // Group the padded text into 16-byte blocks
    let blocks = group(padded_text);

    // Encrypt each block and collect the results
    let encrypted_blocks: Vec<[u8; BLOCK_SIZE]> = blocks
        .iter()
        .map(|&block| aes_encrypt(block, &key))
        .collect();

    un_group(encrypted_blocks)
}

/// Opposite of ecb_encrypt.
fn ecb_decrypt(cipher_text: Vec<u8>, key: [u8; BLOCK_SIZE]) -> Vec<u8> {
    // Group the ciphertext into 16-byte blocks
    let blocks = group(cipher_text);

    // Decrypt each block and collect the results
    let decrypted_blocks: Vec<[u8; BLOCK_SIZE]> = blocks
        .iter()
        .map(|&block| aes_decrypt(block, &key))
        .collect();

    // Ungroup the decrypted blocks into a single byte vector
    let decrypted_data = un_group(decrypted_blocks);

    // Remove padding
    un_pad(decrypted_data)
}

/// The next mode, which you can implement on your own is cipherblock chaining.
/// This mode actually is secure, and it often used in real world applications.
///
/// In this mode, the ciphertext from the first block is XORed with the
/// plaintext of the next block before it is encrypted.
///
/// For more information, and a very clear diagram,
/// see https://de.wikipedia.org/wiki/Cipher_Block_Chaining_Mode
///
/// You will need to generate a random initialization vector (IV) to encrypt the
/// very first block because it doesn't have a previous block. Typically this IV
/// is inserted as the first block of ciphertext.
fn cbc_encrypt(plain_text: Vec<u8>, key: [u8; BLOCK_SIZE]) -> Vec<u8> {
    // Inputs
    let padded_text = pad(plain_text);
    let rand_init_vector = utils::create_rand_init_vector();

    let blocks = group(padded_text);

    // Initial values, assuming the initialization vector is the first vector in the group
    let mut previous_block = rand_init_vector;
    let mut encrypted_blocks = vec![rand_init_vector];

    for block in blocks {
        // XOR input
        let xored_block = utils::xor_block_bytes(&block, &previous_block);
        // Encrypt with key
        let encrypted_block = aes_encrypt(xored_block, &key);
        encrypted_blocks.push(encrypted_block);
        previous_block = encrypted_block;
    }

    un_group(encrypted_blocks)
}

fn cbc_decrypt(cipher_text: Vec<u8>, key: [u8; BLOCK_SIZE]) -> Vec<u8> {
    let blocks = group(cipher_text);

    // The first block is the initialization vector (IV)
    let rand_init_vector = blocks[0];
    let encrypted_blocks = &blocks[1..];

    let mut previous_block = rand_init_vector;
    let mut decrypted_blocks = Vec::new();

    for block in encrypted_blocks {
        // Decrypt
        let decrypted_block = aes_decrypt(*block, &key);
        // Unxor
        let xored_block = utils::xor_block_bytes(&decrypted_block, &previous_block);
        decrypted_blocks.push(xored_block);
        previous_block = *block;
    }

    let decrypted_data = un_group(decrypted_blocks);
    un_pad(decrypted_data)
}

/// Another mode which you can implement on your own is counter mode.
/// This mode is secure as well, and is used in real world applications.
/// It allows parallelized encryption and decryption, as well as random read access when decrypting.
///
/// In this mode, there is an index for each block being encrypted (the "counter"), as well as a random nonce.
/// For a 128-bit cipher, the nonce is 64 bits long.
///
/// For the ith block, the 128-bit value V of `nonce | counter` is constructed, where | denotes
/// concatenation. Then, V is encrypted with the key using ECB mode. Finally, the encrypted V is
/// XOR'd with the plaintext to produce the ciphertext.
///
/// A very clear diagram is present here:
/// https://en.wikipedia.org/wiki/Block_cipher_mode_of_operation#Counter_(CTR)
///
/// Once again, you will need to generate a random nonce which is 64 bits long. This should be
/// inserted as the first block of the ciphertext.
fn ctr_encrypt(plain_text: Vec<u8>, key: [u8; BLOCK_SIZE]) -> Vec<u8> {
    let nonce = utils::create_rand_nonce();

    let mut cipher_text = nonce.to_vec();

    for (i, block) in plain_text.chunks(BLOCK_SIZE).enumerate() {
        let counter = i as u64;

        // Construct the counter block (nonce | counter)
        let mut counter_block = [0u8; BLOCK_SIZE];
        counter_block[..NONCE_SIZE].copy_from_slice(&nonce);
        counter_block[NONCE_SIZE..].copy_from_slice(&counter.to_le_bytes());

        // Encrypt with key
        let encrypted_counter = aes_encrypt(counter_block, &key);

        // XOR
        let mut encrypted_block = vec![0u8; block.len()];
        for j in 0..block.len() {
            encrypted_block[j] = block[j] ^ encrypted_counter[j];
        }

        cipher_text.extend_from_slice(&encrypted_block);
    }

    cipher_text
}

fn ctr_decrypt(cipher_text: Vec<u8>, key: [u8; BLOCK_SIZE]) -> Vec<u8> {
    //
    let nonce = &cipher_text[..NONCE_SIZE];

    let mut plain_text = Vec::new();

    for (i, block) in cipher_text[NONCE_SIZE..].chunks(BLOCK_SIZE).enumerate() {
        let counter = i as u64;

        // Construct the counter block (nonce | counter)
        let mut counter_block = [0u8; BLOCK_SIZE];
        counter_block[..NONCE_SIZE].copy_from_slice(nonce);
        counter_block[NONCE_SIZE..].copy_from_slice(&counter.to_le_bytes());

        // Encrypt with key
        let encrypted_counter = aes_decrypt(counter_block, &key);

        // XOR the encrypted counter block with the ciphertext block
        let mut decrypted_block = vec![0u8; block.len()];
        for j in 0..block.len() {
            decrypted_block[j] = block[j] ^ encrypted_counter[j];
        }

        plain_text.extend_from_slice(&decrypted_block);
    }

    plain_text
}

#[cfg(test)]
mod tests {
    use super::*;
    const KEY: [u8; BLOCK_SIZE] = [0u8; BLOCK_SIZE];

    #[test]
    fn test_ecb() {
        let simple_text = b"Hello, AES Encryption!".to_vec();
        let text_with_padding = b"Short".to_vec();

        let encrypted_text = ecb_encrypt(simple_text.clone(), KEY);
        let decrypted_text = ecb_decrypt(encrypted_text, KEY);
        assert_eq!(decrypted_text, simple_text);

        let encrypted_text = ecb_encrypt(text_with_padding.clone(), KEY);
        let decrypted_text = ecb_decrypt(encrypted_text, KEY);
        assert_eq!(decrypted_text, text_with_padding);
    }

    #[test]
    fn test_cbc() {
        let simple_text = b"Hello, AES Encryption!".to_vec();
        let text_with_padding = b"Short".to_vec();

        let encrypted_text = cbc_encrypt(simple_text.clone(), KEY);
        let decrypted_text = cbc_decrypt(encrypted_text, KEY);
        assert_eq!(decrypted_text, simple_text);

        let encrypted_text = cbc_encrypt(text_with_padding.clone(), KEY);
        let decrypted_text = cbc_decrypt(encrypted_text, KEY);
        assert_eq!(decrypted_text, text_with_padding);
    }

    #[test]
    fn test_ctr() {
        let simple_text = b"Hello, AES Encryption!".to_vec();
        let text_with_padding = b"Short".to_vec();
        let text_spans_multiple_blocks = b"Longer text that spans multiple blocks!".to_vec();

        let encrypted_text = ctr_encrypt(simple_text.clone(), KEY);
        let decrypted_text = ctr_decrypt(encrypted_text, KEY);
        assert_eq!(decrypted_text, simple_text);

        let encrypted_text = ctr_encrypt(text_with_padding.clone(), KEY);
        let decrypted_text = ctr_decrypt(encrypted_text, KEY);
        assert_eq!(decrypted_text, text_with_padding);

        let encrypted_text = ctr_encrypt(text_spans_multiple_blocks.clone(), KEY);
        let decrypted_text = ctr_decrypt(encrypted_text, KEY);
        assert_eq!(decrypted_text, text_spans_multiple_blocks);
    }
}
