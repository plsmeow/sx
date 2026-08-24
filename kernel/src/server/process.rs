use std::io::{Error, ErrorKind};
use std::sync::Arc;

use aes_gcm::{Aes256Gcm, KeyInit};
use rand::Rng;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::version::{KERNEL_VERSION_ARR, KERNEL_VERSION_STR};

pub async fn process_socket(
  socket: &mut TcpStream,
  addr: &str,
  expected_password: &Arc<Option<String>>,
) -> std::io::Result<Aes256Gcm> {
  process_handshake(socket, addr).await?;
  process_authentication(socket, expected_password).await?;
  process_encryption(socket).await
}

async fn process_handshake(socket: &mut TcpStream, addr: &str) -> std::io::Result<()> {
  let version_len = socket.read_u8().await?;
  let mut version_buf = Vec::with_capacity(version_len as usize);

  for _ in 0..version_len {
    version_buf.push(socket.read_u8().await?);
  }

  let version = String::from_utf8_lossy(&version_buf);
  let version_without_postfix = version.split('-').collect::<Vec<&str>>();
  let splitted_version = version_without_postfix[0].split('.').collect::<Vec<&str>>();

  if splitted_version.len() != 3 {
    socket.write_u8(0x03).await?;
    return Err(Error::new(ErrorKind::InvalidData, "client sent an incorrect version"));
  }

  let major = splitted_version[0].parse::<u8>().unwrap_or(0);
  let minor = splitted_version[1].parse::<u8>().unwrap_or(0);
  let patch = splitted_version[2].parse::<u8>().unwrap_or(0);

  if major != KERNEL_VERSION_ARR[0] || minor != KERNEL_VERSION_ARR[1] {
    socket.write_u8(0x02).await?;
    return Err(Error::new(
      ErrorKind::InvalidData,
      format!("incompatible version (required {})", KERNEL_VERSION_STR),
    ));
  } else if patch != KERNEL_VERSION_ARR[2] {
    socket.write_u8(0x01).await?;
    println!("[warning :: {}] current kernel version does not match the client version, there may be bugs (kernel version: {}, client version: {})", addr, KERNEL_VERSION_STR, version);
  } else {
    socket.write_u8(0x00).await?;
  }

  Ok(())
}

async fn process_authentication(
  socket: &mut TcpStream,
  expected_password: &Arc<Option<String>>,
) -> std::io::Result<()> {
  let Some(expected) = expected_password.as_ref() else {
    socket.write_u8(0x00).await?;
    return Ok(());
  };

  socket.write_u8(0x01).await?;

  let password_len = socket.read_u8().await?;

  if password_len != expected.len() as u8 {
    socket.write_u8(0x01).await?;
    return Err(Error::new(ErrorKind::InvalidData, "incorrect password"));
  }

  let mut password_buf = Vec::with_capacity(password_len as usize);

  for _ in 0..password_len {
    password_buf.push(socket.read_u8().await?);
  }

  let password = String::from_utf8_lossy(&password_buf);

  if password != *expected {
    socket.write_u8(0x01).await?;
    return Err(Error::new(ErrorKind::InvalidData, "incorrect password"));
  }

  socket.write_u8(0x00).await?;

  Ok(())
}

async fn process_encryption(socket: &mut TcpStream) -> std::io::Result<Aes256Gcm> {
  let key = rand::thread_rng().gen::<[u8; 32]>();
  let cipher = Aes256Gcm::new((&key).into());

  socket.write_all(&key).await?;

  Ok(cipher)
}
