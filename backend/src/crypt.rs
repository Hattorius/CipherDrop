use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{decode, encode};
use rand::RngCore;

pub struct Encrypted {
    pub key: String,
    pub nonce: String,
    pub result: Vec<u8>,
}

pub fn encrypt(value: Vec<u8>) -> Option<Encrypted> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let key_base64 = encode(key.as_slice());

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let nonce_base64 = encode(nonce_bytes);

    let cipher = Aes256Gcm::new(&key);
    let ciphertext = match cipher.encrypt(nonce, value.as_ref()) {
        Ok(res) => res,
        _ => return None,
    };

    Some(Encrypted {
        key: key_base64,
        nonce: nonce_base64,
        result: ciphertext,
    })
}

pub fn decrypt(en: Encrypted) -> Option<Vec<u8>> {
    let decoded_key = decode(&en.key).expect("Failed decoding base64 key"); // This is ok since the key never leaves our eco
    let decoded_nonce = decode(&en.nonce).expect("Failed decoding base64 nonce"); // This is ok since the key never leaves our eco

    let key_for_decryption = aes_gcm::Key::<Aes256Gcm>::from_slice(&decoded_key);
    let nonce = Nonce::from_slice(&decoded_nonce);

    let cipher_for_decryption = Aes256Gcm::new(key_for_decryption);
    match cipher_for_decryption.decrypt(nonce, en.result.as_ref()) {
        Ok(res) => Some(res),
        _ => return None,
    }
}
