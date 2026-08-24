use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::server::transfer::emit_log;

pub static INDEX_SYSTEM: Lazy<IndexSystem> = Lazy::new(|| IndexSystem::new());

/// Система индексации ботов
pub struct IndexSystem {
  pub map: RwLock<Vec<String>>,
}

impl IndexSystem {
  /// Метод создания менеджера состояний
  pub fn new() -> Self {
    Self {
      map: RwLock::new(Vec::with_capacity(255)),
    }
  }

  /// Метод регистрации бота
  pub async fn register(&self, username: &str) -> u8 {
    let mut guard = self.map.write().await;
    if guard.len() >= 255 {
      return 0;
    }

    guard.push(username.to_string());
    guard.len() as u8 - 1
  }

  /// Метод получения юзернейма по индексу
  pub async fn username_by_index(&self, index: &u8) -> Option<String> {
    self.map.read().await.get(*index as usize).map(|s| s.clone())
  }

  /// Метод попытки получения юзернейма по индексу
  pub fn get_username_by_index(&self, index: &u8) -> Option<String> {
    if let Ok(guard) = self.map.try_read() {
      return guard.get(*index as usize).map(|s| s.clone());
    }

    None
  }

  /// Метод получения индекса по юзернейму
  pub async fn index_by_username(&self, username: &str) -> Option<u8> {
    let guard = self.map.read().await;

    for (i, u) in guard.iter().enumerate() {
      if u == username {
        return Some(i as u8);
      }
    }

    None
  }

  /// Метод получения всех зарегистрированных юзернеймов
  pub async fn get_all_usernames(&self) -> Vec<String> {
    self.map.read().await.clone()
  }

  /// Метод очистки хранилища состояний
  pub async fn clear(&self) {
    self.map.write().await.clear();
    emit_log("Реестр индексов ботов очищен", "system");
  }
}
