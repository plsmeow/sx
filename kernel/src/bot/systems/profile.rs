use bytes::{BufMut, BytesMut};
use hashbrown::HashMap;
use md5::{digest::Update, Digest, Md5};
use once_cell::sync::Lazy;
use salarixi_extensions::buffer::BufferExt;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::bot::systems::index::INDEX_SYSTEM;
use crate::server::transfer::{emit_log, TransferEvent, UpdateBotProfilePayload, TRANSFER};
use crate::sleep;

pub static PROFILE_SYSTEM: Lazy<ProfileSystem> = Lazy::new(|| ProfileSystem::new());

#[derive(Clone)]
pub struct Profile {
  pub status: BotStatus,
  pub password: Option<String>,
  pub email: Option<String>,
  pub proxy: ProfileProxy,
  pub ping: u32,
  pub health: u32,
  pub registered: bool,
  pub logined: bool,
  pub captcha_caught: bool,
  pub group: String,
}

impl Profile {
  pub fn write(&self, buf: &mut bytes::BytesMut) {
    self.status.write(buf);
    self.password.write(buf);
    self.email.write(buf);
    self.proxy.write(buf);
    self.ping.write(buf);
    self.health.write(buf);
    self.registered.write(buf);
    self.logined.write(buf);
    self.captcha_caught.write(buf);
    self.group.write(buf);
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BotStatus {
  Waiting,
  Connected,
  Disconnected,
}

impl BotStatus {
  pub fn write(&self, buf: &mut bytes::BytesMut) {
    buf.put_u8(match self {
      Self::Waiting => 0x00,
      Self::Connected => 0x01,
      Self::Disconnected => 0x02,
    });
  }
}

#[derive(Clone)]
pub struct ProfileProxy {
  pub ip_address: Option<String>,
  pub proxy: Option<String>,
  pub username: Option<String>,
  pub password: Option<String>,
}

impl ProfileProxy {
  pub fn write(&self, buf: &mut bytes::BytesMut) {
    self.ip_address.write(buf);
    self.proxy.write(buf);
    self.username.write(buf);
    self.password.write(buf);
  }
}

impl Profile {
  pub fn new(password: Option<String>, email: Option<String>) -> Self {
    Self {
      status: BotStatus::Waiting,
      password: password,
      email: email,
      proxy: ProfileProxy {
        ip_address: None,
        proxy: None,
        username: None,
        password: None,
      },
      ping: 0,
      health: 0,
      registered: false,
      logined: false,
      captcha_caught: false,
      group: "global".to_string(),
    }
  }
}

pub struct ProfileSystem {
  pub map: RwLock<HashMap<u8, Profile>>,
  updater_task: RwLock<Option<JoinHandle<()>>>,
}

impl ProfileSystem {
  pub fn new() -> Self {
    Self {
      map: RwLock::new(HashMap::new()),
      updater_task: RwLock::new(None),
    }
  }

  /// Метод активации системы
  pub async fn activate(&self, update_rate: u64, optimized: bool) {
    let updater_task = if optimized {
      tokio::spawn(async move {
        let mut last_hashes = HashMap::new();

        loop {
          for (index, profile) in PROFILE_SYSTEM.map.read().await.iter() {
            let mut buf = BytesMut::new();
            profile.write(&mut buf);

            let finalized = Md5::new().chain(&buf).finalize();
            let hash: [u8; 16] = match finalized.as_slice()[..16].try_into() {
              Ok(h) => h,
              Err(_) => continue,
            };

            drop(buf);

            if let Some((last_status, last_hash)) = last_hashes.get(index) {
              if profile.status == *last_status && hash == *last_hash {
                // println!("[debug :: monitoring] {} skipped (status: {:?}, hash: {:?})", username, profile.status, hash);
                continue;
              }
            };

            // println!("[debug :: monitoring] {} updated (status: {:?}, hash: {:?})", username, profile.status, hash);

            let Some(username) = INDEX_SYSTEM.username_by_index(index).await else {
              continue;
            };

            last_hashes.insert(*index, (profile.status.clone(), hash));

            TRANSFER.emit(TransferEvent::UpdateBotProfile(UpdateBotProfilePayload {
              username,
              profile: profile.clone(),
            }));
          }

          sleep!(update_rate);
        }
      })
    } else {
      tokio::spawn(async move {
        loop {
          for (index, profile) in PROFILE_SYSTEM.map.read().await.iter() {
            let Some(username) = INDEX_SYSTEM.username_by_index(index).await else {
              continue;
            };

            TRANSFER.emit(TransferEvent::UpdateBotProfile(UpdateBotProfilePayload {
              username,
              profile: profile.clone(),
            }));
          }

          sleep!(update_rate);
        }
      })
    };

    *self.updater_task.write().await = Some(updater_task);
  }

  /// Метод выключения системы
  pub async fn shutdown(&self) {
    if let Some(task) = self.updater_task.write().await.take() {
      task.abort();
    }

    self.map.write().await.clear();
    emit_log("Профили ботов очищены", "system");
  }

  /// Метод регистрации бота
  pub async fn register(&self, index: u8, password: Option<String>, email: Option<String>) {
    self.map.write().await.insert(index, Profile::new(password, email));
  }

  /// Метод получения количества зарегистрированных профилей
  pub async fn len(&self) -> usize {
    self.map.read().await.len()
  }

  /// Метод получения количества подключенных ботов
  pub async fn get_connected_count(&self) -> usize {
    let mut count = 0;
    let guard = self.map.read().await;

    guard.iter().for_each(|(_, profile)| {
      if profile.status == BotStatus::Connected {
        count += 1;
      }
    });

    count
  }

  /// Метод попытки получения количества подключенных ботов
  pub fn try_get_connected_count(&self) -> i32 {
    if let Ok(guard) = self.map.try_read() {
      let mut count = 0;

      guard.iter().for_each(|(_, profile)| {
        if profile.status == BotStatus::Connected {
          count += 1;
        }
      });

      return count;
    }

    -1
  }

  /// Метод получения клонированного профиля указанного бота
  pub async fn get(&self, index: &u8) -> Option<Profile> {
    let guard = self.map.read().await;
    guard.get(index).cloned()
  }

  /// Метод захвата мутабельного профиля указанного бота
  pub async fn lock_mut<F>(&self, index: &u8, func: F)
  where
    F: AsyncFnOnce(&mut Profile),
  {
    let mut guard = self.map.write().await;
    if let Some(profile) = guard.get_mut(index) {
      func(profile).await;
    }
  }

  /// Метод получения всех профилей
  pub async fn get_all(&self) -> HashMap<u8, Profile> {
    let mut result = HashMap::new();
    let guard = self.map.read().await;

    guard.iter().for_each(|(index, profile)| {
      result.insert(*index, profile.clone());
    });

    result
  }

  /// Метод получения всех профилей подключенных ботов
  pub async fn get_all_connected(&self) -> HashMap<u8, Profile> {
    let mut result = HashMap::new();
    let guard = self.map.read().await;

    guard.iter().for_each(|(index, profile)| {
      if profile.status == BotStatus::Connected {
        result.insert(*index, profile.clone());
      }
    });

    result
  }

  /// Метод попытки получения профиля указанного бота
  pub fn try_get(&self, index: &u8) -> Option<Profile> {
    if let Ok(guard) = self.map.try_read() {
      if let Some(p) = guard.get(index) {
        return Some(p.clone());
      }
    }

    None
  }

  /// Метод попытки получения всех профилей подключенных ботов
  pub fn try_get_all_connected(&self) -> HashMap<u8, Profile> {
    let mut result = HashMap::new();

    if let Ok(guard) = self.map.try_read() {
      guard.iter().for_each(|(index, profile)| {
        if profile.status == BotStatus::Connected {
          result.insert(*index, profile.clone());
        }
      });
    }

    result
  }
}

/// Вспомогательный макрос захвата профиля указанного бота
#[macro_export]
macro_rules! take_profile {
  ($index:expr, $func:expr) => {
    crate::bot::systems::profile::PROFILE_SYSTEM
      .lock_mut($index, $func)
      .await;
  };
}
