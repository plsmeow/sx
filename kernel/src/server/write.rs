use std::sync::Arc;

use aes_gcm::Aes256Gcm;
use bytes::{BufMut, BytesMut};
use salarixi_extensions::encryption::encrypt_data;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::RwLock;

use crate::server::Sockets;

/// Вспомогательная функция записи данных в указанный сокет
pub async fn write_to_socket(addr: &str, writer: &Arc<RwLock<OwnedWriteHalf>>, cipher: &Arc<Aes256Gcm>, data: &[u8]) {
  let mut buf = BytesMut::new();

  let encrypted = match encrypt_data(cipher, data) {
    Ok(e) => e,
    Err(e) => {
      println!("[error :: {}] encryption error: {}", addr, e);
      return;
    }
  };

  buf.put_u32(encrypted.len() as u32);
  buf.put_slice(&encrypted);

  match writer.write().await.write_all(&buf).await {
    Ok(_) => {}
    Err(e) => {
      println!("[error :: {}] write error: {}", addr, e);
    }
  }
}

/// Вспомогательная функция записи данных во все сокеты
pub async fn write_to_sockets(sockets: &Sockets, data: &[u8]) {
  for (addr, (cipher, _, writer)) in sockets.read().await.iter() {
    write_to_socket(addr, writer, cipher, data).await;
  }
}
