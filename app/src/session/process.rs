use std::io::{Error, ErrorKind};

use aes_gcm::{Aes256Gcm, KeyInit};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::version::CLIENT_VERSION_STR;

pub async fn client_process(socket: &mut TcpStream, password: Option<String>) -> std::io::Result<Aes256Gcm> {
  process_handshake(socket).await?;
  process_authentication(socket, password).await?;
  process_encryption(socket).await
}

async fn process_handshake(socket: &mut TcpStream) -> std::io::Result<()> {
  socket.write_u8(CLIENT_VERSION_STR.len() as u8).await?;
  socket.write_all(CLIENT_VERSION_STR.as_bytes()).await?;

  let resp = socket.read_u8().await?;

  if resp > 0x01 {
    socket.shutdown().await?;
    return Err(Error::new(
      ErrorKind::NotConnected,
      "client version is incompatible with kernel version",
    ));
  }

  Ok(())
}

async fn process_authentication(socket: &mut TcpStream, password: Option<String>) -> std::io::Result<()> {
  let password_required = socket.read_u8().await? == 0x01;

  if !password_required {
    return Ok(());
  }

  let Some(pass) = password else {
    socket.shutdown().await?;
    return Err(Error::new(
      ErrorKind::InvalidInput,
      "kernel requires a password to establish a connection",
    ));
  };

  socket.write_u8(pass.len() as u8).await?;
  socket.write_all(pass.as_bytes()).await?;

  let resp = socket.read_u8().await?;

  if resp == 0x00 {
    return Ok(());
  } else {
    socket.shutdown().await?;
    return Err(Error::new(ErrorKind::InvalidInput, "incorrect password"));
  }
}

async fn process_encryption(socket: &mut TcpStream) -> std::io::Result<Aes256Gcm> {
  let mut key = [0u8; 32];
  socket.read_exact(&mut key).await?;

  let cipher = Aes256Gcm::new((&key).into());

  Ok(cipher)
}
