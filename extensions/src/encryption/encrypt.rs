use std::io::{Error, ErrorKind};

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Nonce};
use rand::Rng;

pub fn encrypt_data(cipher: &Aes256Gcm, input: &[u8]) -> std::io::Result<Vec<u8>> {
  let rng = rand::thread_rng().gen::<[u8; 12]>();
  let nonce = Nonce::from_slice(&rng);
  let encrypted = match cipher.encrypt(nonce, input) {
    Ok(data) => data,
    Err(e) => {
      return Err(Error::new(ErrorKind::InvalidData, e.to_string()));
    }
  };

  Ok([nonce.to_vec(), encrypted].concat())
}
