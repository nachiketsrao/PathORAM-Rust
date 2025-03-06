use aes::Aes256;
use ctr::Ctr128BE;
use cipher::{KeyIvInit, StreamCipher};
use rand::rngs::OsRng;
use rand::RngCore;
use std::error::Error;

/// Returns a random integer in the range [0, max).
pub fn get_random_int(max: i32) -> i32 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(0..max)
}

/// Generates a random 32-byte key for AES-256.
pub fn generate_random_key() -> Result<Vec<u8>, Box<dyn Error>> {
    let mut key = vec![0u8; 32]; // AES-256 key size
    OsRng.fill_bytes(&mut key);
    Ok(key)
}

/// Encrypts data using AES-256 in CTR mode.
/// The returned ciphertext has the IV (16 bytes) prepended.
pub fn encrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    // Generate a random IV of 16 bytes.
    let mut iv = [0u8; 16];
    OsRng.fill_bytes(&mut iv);

    // Create a CTR cipher instance using AES-256.
    type Aes256Ctr = Ctr128BE<Aes256>;
    let mut cipher = Aes256Ctr::new_from_slices(key, &iv)
        .map_err(|e| format!("Error initializing cipher: {:?}", e))?;
    
    // Encrypt the data in-place.
    let mut ciphertext = data.to_vec();
    cipher.apply_keystream(&mut ciphertext);

    // Prepend the IV to the ciphertext.
    let mut output = Vec::with_capacity(iv.len() + ciphertext.len());
    output.extend_from_slice(&iv);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

/// Decrypts data using AES-256 in CTR mode.
/// Expects the ciphertext to have the IV (first 16 bytes) prepended.
pub fn decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if data.len() < 16 {
        return Err("ciphertext too short".into());
    }
    let iv = &data[..16];
    let mut ciphertext = data[16..].to_vec();

    type Aes256Ctr = Ctr128BE<Aes256>;
    let mut cipher = Aes256Ctr::new_from_slices(key, iv)
        .map_err(|e| format!("Error initializing cipher: {:?}", e))?;
    cipher.apply_keystream(&mut ciphertext);
    Ok(ciphertext)
}