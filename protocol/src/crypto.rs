//! Cryptographic functions for Bufferbane protocol
//! Uses ChaCha20-Poly1305 AEAD for encryption and authentication

use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305,
};
use thiserror::Error;

/// Shared secret size (32 bytes for ChaCha20)
pub const SECRET_SIZE: usize = 32;

/// Authentication tag size (16 bytes for Poly1305)
pub const TAG_SIZE: usize = 16;

/// Nonce size (12 bytes)
pub const NONCE_SIZE: usize = 12;

/// Crypto errors
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Encryption failed")]
    EncryptionFailed,
    
    #[error("Decryption failed (invalid auth tag or corrupted data)")]
    DecryptionFailed,
    
    #[error("Invalid shared secret length (expected {expected}, got {got})")]
    InvalidSecretLength { expected: usize, got: usize },
}

/// Encrypt payload using ChaCha20-Poly1305 AEAD
///
/// # Arguments
/// * `plaintext` - Data to encrypt
/// * `shared_secret` - 32-byte shared secret
/// * `nonce` - 12-byte nonce (derived from client_id + timestamp)
/// * `associated_data` - Additional authenticated data (packet header)
///
/// # Returns
/// * `Ok(Vec<u8>)` - Ciphertext + 16-byte auth tag
/// * `Err(CryptoError)` - Encryption failed
pub fn encrypt(
    plaintext: &[u8],
    shared_secret: &[u8],
    nonce: &[u8; NONCE_SIZE],
    associated_data: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    if shared_secret.len() != SECRET_SIZE {
        return Err(CryptoError::InvalidSecretLength {
            expected: SECRET_SIZE,
            got: shared_secret.len(),
        });
    }
    
    let cipher = ChaCha20Poly1305::new(shared_secret.into());
    let nonce = nonce.into();
    
    let payload = Payload {
        msg: plaintext,
        aad: associated_data,
    };
    
    cipher
        .encrypt(nonce, payload)
        .map_err(|_| CryptoError::EncryptionFailed)
}

/// Decrypt payload using ChaCha20-Poly1305 AEAD
///
/// # Arguments
/// * `ciphertext` - Encrypted data + 16-byte auth tag
/// * `shared_secret` - 32-byte shared secret
/// * `nonce` - 12-byte nonce (derived from client_id + timestamp)
/// * `associated_data` - Additional authenticated data (packet header)
///
/// # Returns
/// * `Ok(Vec<u8>)` - Decrypted plaintext
/// * `Err(CryptoError)` - Decryption or authentication failed
pub fn decrypt(
    ciphertext: &[u8],
    shared_secret: &[u8],
    nonce: &[u8; NONCE_SIZE],
    associated_data: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    if shared_secret.len() != SECRET_SIZE {
        return Err(CryptoError::InvalidSecretLength {
            expected: SECRET_SIZE,
            got: shared_secret.len(),
        });
    }
    
    let cipher = ChaCha20Poly1305::new(shared_secret.into());
    let nonce = nonce.into();
    
    let payload = Payload {
        msg: ciphertext,
        aad: associated_data,
    };
    
    cipher
        .decrypt(nonce, payload)
        .map_err(|_| CryptoError::DecryptionFailed)
}

/// Parse hex-encoded shared secret from configuration
///
/// # Arguments
/// * `hex_str` - Hex string (64 characters = 32 bytes)
///
/// # Returns
/// * `Ok([u8; 32])` - Parsed secret
/// * `Err(String)` - Parse error
pub fn parse_shared_secret(hex_str: &str) -> Result<[u8; SECRET_SIZE], String> {
    let hex_str = hex_str.trim();
    
    if hex_str.len() != SECRET_SIZE * 2 {
        return Err(format!(
            "Invalid shared secret length: expected {} hex characters, got {}",
            SECRET_SIZE * 2,
            hex_str.len()
        ));
    }
    
    let mut secret = [0u8; SECRET_SIZE];
    for i in 0..SECRET_SIZE {
        let byte_str = &hex_str[i * 2..i * 2 + 2];
        secret[i] = u8::from_str_radix(byte_str, 16)
            .map_err(|e| format!("Invalid hex at position {}: {}", i * 2, e))?;
    }
    
    Ok(secret)
}

/// Generate a new random shared secret
pub fn generate_shared_secret() -> [u8; SECRET_SIZE] {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut secret = [0u8; SECRET_SIZE];
    rng.fill(&mut secret);
    secret
}

/// Format shared secret as hex string
pub fn format_shared_secret(secret: &[u8; SECRET_SIZE]) -> String {
    secret.iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encrypt_decrypt() {
        let secret = generate_shared_secret();
        let nonce = [0u8; NONCE_SIZE];
        let plaintext = b"Hello, Bufferbane!";
        let aad = b"header data";
        
        // Encrypt
        let ciphertext = encrypt(plaintext, &secret, &nonce, aad).unwrap();
        assert!(ciphertext.len() > plaintext.len()); // Includes auth tag
        
        // Decrypt
        let decrypted = decrypt(&ciphertext, &secret, &nonce, aad).unwrap();
        assert_eq!(decrypted, plaintext);
    }
    
    #[test]
    fn test_decrypt_wrong_key_fails() {
        let secret1 = generate_shared_secret();
        let secret2 = generate_shared_secret();
        let nonce = [0u8; NONCE_SIZE];
        let plaintext = b"Secret message";
        let aad = b"header";
        
        let ciphertext = encrypt(plaintext, &secret1, &nonce, aad).unwrap();
        let result = decrypt(&ciphertext, &secret2, &nonce, aad);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_decrypt_tampered_ciphertext_fails() {
        let secret = generate_shared_secret();
        let nonce = [0u8; NONCE_SIZE];
        let plaintext = b"Important data";
        let aad = b"header";
        
        let mut ciphertext = encrypt(plaintext, &secret, &nonce, aad).unwrap();
        
        // Tamper with ciphertext
        if !ciphertext.is_empty() {
            ciphertext[0] ^= 0xFF;
        }
        
        let result = decrypt(&ciphertext, &secret, &nonce, aad);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parse_shared_secret() {
        let hex = "a7b3c9d8e1f4a2b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9";
        let secret = parse_shared_secret(hex).unwrap();
        assert_eq!(secret.len(), SECRET_SIZE);
        
        // Round trip
        let formatted = format_shared_secret(&secret);
        assert_eq!(formatted, hex);
    }
    
    #[test]
    fn test_parse_invalid_secret() {
        // Too short
        assert!(parse_shared_secret("abcd").is_err());
        
        // Invalid hex
        assert!(parse_shared_secret("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz").is_err());
    }
}

