use std::io::{Error, ErrorKind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use aes_gcm::Aes256Gcm;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use once_cell::sync::Lazy;
use salarixi_extensions::encryption::{decrypt_data, encrypt_data};
use salarixi_kernel::launch::runner::stop_bots_and_destroy_data;
use salarixi_kernel::server::Server;
use salarixi_kernel::sleep;
use salarixi_kernel::tools::{randnum, randstr, CharClass};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::emit::EMIT_MANAGER;
use crate::result::CommandResult;
use crate::session::client_process;
use crate::session::transform::transform_id;
use crate::{failed, success};

pub static SESSION: Lazy<Session> = Lazy::new(|| Session::new());

pub struct Session {
  local_server: RwLock<Option<Server>>,
  reader_task: RwLock<Option<JoinHandle<()>>>,
  reader: Arc<RwLock<Option<OwnedReadHalf>>>,
  writer: Arc<RwLock<Option<OwnedWriteHalf>>>,
  cipher: Arc<RwLock<Option<Aes256Gcm>>>,
  stopping: AtomicBool,
  establishing: AtomicBool,
}

impl Session {
  pub fn new() -> Self {
    Self {
      local_server: RwLock::new(None),
      reader_task: RwLock::new(None),
      reader: Arc::new(RwLock::new(None)),
      writer: Arc::new(RwLock::new(None)),
      cipher: Arc::new(RwLock::new(None)),
      stopping: AtomicBool::new(false),
      establishing: AtomicBool::new(false),
    }
  }

  /// Метод запуска локального сервера
  pub async fn run_local_server(&self) -> CommandResult<(String, String)> {
    if self.establishing.load(Ordering::SeqCst) {
      failed!("session is already being established");
    }

    if !self.shutdown().await {
      failed!("it is not possible to establish a new session until the current one is terminated");
    }

    self.establishing.store(true, Ordering::SeqCst);

    let mut data = None;
    let mut error = None;
    let random_password = randstr(CharClass::Multi, randnum(16, 24));

    for port in 30000..=50000 {
      let addr = format!("127.0.0.1:{}", port);
      match Server::run(&addr, Some(random_password.clone())).await {
        Ok(s) => {
          data = Some((s, addr));
          break;
        }
        Err(e) => {
          error = Some(e);
        }
      }
    }

    if let Some((server, addr)) = data {
      server.start_processing().await;
      *self.local_server.write().await = Some(server);
      sleep!(1000);

      let result = self
        .connect_to_remote_server(addr.clone(), Some(random_password.clone()), false)
        .await;

      self.establishing.store(false, Ordering::SeqCst);

      if let Some(e) = result.error {
        failed!("{}", e);
      } else {
        success!((addr, random_password));
      }
    } else {
      self.establishing.store(false, Ordering::SeqCst);

      if let Some(e) = error {
        failed!("{}", e);
      } else {
        failed!("unknown error occurred while starting server");
      }
    }
  }

  /// Метод подключения к удалённому серверу
  pub async fn connect_to_remote_server(
    &self,
    addr: String,
    password: Option<String>,
    is_changing: bool,
  ) -> CommandResult<()> {
    if is_changing {
      if self.establishing.load(Ordering::SeqCst) {
        failed!("session is already being established");
      }

      if !self.shutdown().await {
        failed!("it is not possible to establish a new session until the current one is terminated");
      }
    }

    let mut socket = match TcpStream::connect(addr).await {
      Ok(s) => s,
      Err(_) => {
        failed!("target server is not responding");
      }
    };

    let cipher = match client_process(&mut socket, password).await {
      Ok(c) => c,
      Err(e) => failed!("{}", e),
    };

    *self.cipher.write().await = Some(cipher);

    let (r, w) = socket.into_split();

    *self.reader.write().await = Some(r);
    *self.writer.write().await = Some(w);

    self.run_reader_task().await;

    success!(());
  }

  /// Метод выключения сессии
  pub async fn shutdown(&self) -> bool {
    if self.stopping.load(Ordering::SeqCst) {
      return false;
    }

    self.stopping.store(true, Ordering::SeqCst);

    let mut task_guard = self.reader_task.write().await;
    if let Some(t) = task_guard.as_ref() {
      t.abort();
    }

    *task_guard = None;
    drop(task_guard);

    let mut writer_guard = self.writer.write().await;
    if let Some(w) = writer_guard.as_mut() {
      let _ = w.shutdown().await;
    }

    *writer_guard = None;
    drop(writer_guard);

    *self.reader.write().await = None;

    let mut server_guard = self.local_server.write().await;
    if let Some(s) = server_guard.as_ref() {
      stop_bots_and_destroy_data(false).await;
      s.stop_processing().await;
    }

    *server_guard = None;
    drop(server_guard);

    *self.cipher.write().await = None;

    self.stopping.store(false, Ordering::SeqCst);

    true
  }

  /// Метод запуска цикла чтения входящих пакетов
  async fn run_reader_task(&self) {
    let reader = self.reader.clone();
    let cipher = self.cipher.clone();
    let task = tokio::spawn(async move {
      let read_fn = async |half: &mut OwnedReadHalf| {
        let len = half.read_u32().await?;
        let mut input = Vec::with_capacity(len as usize);

        for _ in 0..len {
          input.push(half.read_u8().await?);
        }

        let cipher_guard = cipher.read().await;
        let Some(cipher) = cipher_guard.as_ref() else {
          return Err(Error::new(ErrorKind::Other, "cipher is not set"));
        };

        let decrypted = decrypt_data(cipher, &input)?;
        let mut data = Bytes::from_iter(decrypted);
        let id = data.try_get_u8()?;

        Ok((id, data.to_vec()))
      };

      loop {
        let mut reader_guard = reader.write().await;
        let Some(half) = reader_guard.as_mut() else {
          continue;
        };

        let Ok((id, data)) = read_fn(half).await else {
          continue;
        };

        let transformed_id = transform_id(id);

        EMIT_MANAGER.emit(transformed_id, data);
      }
    });

    *self.reader_task.write().await = Some(task);
  }

  /// Метод отправки команды
  pub async fn send_command(&self, data: Vec<u8>) -> CommandResult<()> {
    let mut writer_guard = self.writer.write().await;
    let Some(writer) = writer_guard.as_mut() else {
      failed!("session not established");
    };

    let cipher_guard = self.cipher.read().await;
    let Some(cipher) = cipher_guard.as_ref() else {
      failed!("cipher is not set");
    };

    let encrypted = match encrypt_data(cipher, &data) {
      Ok(e) => e,
      Err(e) => failed!("encryption error: {}", e),
    };

    let mut buf = BytesMut::new();
    buf.put_u32(encrypted.len() as u32);
    buf.put_slice(&encrypted);

    match writer.write_all(&buf).await {
      Ok(_) => {}
      Err(e) => failed!("write error: {}", e),
    }

    success!(());
  }
}
