use std::io::{Error, ErrorKind};

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Nonce};

pub fn decrypt_data(cipher: &Aes256Gcm, input: &[u8]) -> std::io::Result<Vec<u8>> {
  let nonce = Nonce::from_slice(&input[..12]);
  let decrypted = match cipher.decrypt(nonce, &input[12..]) {
    Ok(data) => data,
    Err(e) => {
      return Err(Error::new(ErrorKind::InvalidData, e.to_string()));
    }
  };

  Ok(decrypted)
}
