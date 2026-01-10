pub mod utils {
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use aes_gcm::aead::Aead;

    pub fn decrypt_data(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
        let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;

        let nonce_bytes = &encrypted_data[0..12];
        let nonce = Nonce::from_slice(nonce_bytes);

        let ciphertext = &encrypted_data[12..];

        match cipher.decrypt(nonce, ciphertext) {
            Ok(plaintext) => Ok(plaintext),
            Err(_) => Err("failed to decrypt.".to_string()),
        }
    }
}