pub mod utils {
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use aes_gcm::aead::Aead;
    use rand::Rng;

    pub fn encrypt_data(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
        let cipher = Aes256Gcm::new_from_slice(key).expect("uncorrect key");

        let mut rng = rand::rng();
        let nonce_bytes: [u8; 12] = rng.r#gen();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext)
            .expect("encoding failed!");

        [nonce.as_slice(), &ciphertext].concat()
    }
}