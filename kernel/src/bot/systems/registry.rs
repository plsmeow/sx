use std::sync::Arc;

use azalea::prelude::*;
use azalea::swarm::Swarm;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::server::transfer::emit_log;

pub static REGISTRY_SYSTEM: Lazy<RegistrySystem> = Lazy::new(|| RegistrySystem::new());

/// Система реестра роя и ботов
pub struct RegistrySystem {
  pub swarm: Arc<RwLock<Option<Swarm>>>,
  pub bots: DashMap<u8, Arc<Client>>,
}

impl RegistrySystem {
  pub fn new() -> Self {
    Self {
      swarm: Arc::new(RwLock::new(None)),
      bots: DashMap::new(),
    }
  }

  /// Метод установки роя
  pub async fn set_swarm(&self, swarm: Swarm) {
    let mut guard = self.swarm.write().await;
    *guard = Some(swarm);
  }

  /// Метод уничтожения роя
  pub async fn destroy_swarm(&self) {
    if let Some(swarm) = self.swarm.write().await.take() {
      swarm.ecs.write().write_message(AppExit::Success);
    }
  }

  /// Метод регистрации бота
  pub fn register_bot(&self, index: u8, bot: Client) {
    self.bots.insert(index, Arc::new(bot));
  }

  /// Метод получения бота по юзернейму
  pub fn get_bot(&self, index: &u8) -> Option<Arc<Client>> {
    self.bots.get(index).map(|cell| cell.clone())
  }

  /// Асинхронный метод получения бота по юзернейму
  pub async fn async_get_bot<F, T>(&self, index: &u8, f: F) -> Option<T>
  where
    F: AsyncFnOnce(&Client) -> T,
  {
    if let Some(reference) = self.bots.get(index) {
      return Some(f(reference.as_ref()).await);
    }

    None
  }

  /// Метод полной очистки реестра
  pub async fn clear(&self) {
    self.bots.clear();
    *self.swarm.write().await = None;

    emit_log("Реестр ботов очищен", "system");
  }
}

/// Вспомогательный макрос захвата указанного бота
#[macro_export]
macro_rules! take_bot {
  ($username:expr, $func:expr) => {
    crate::bot::systems::registry::REGISTRY_SYSTEM
      .async_get_bot($username, $func)
      .await;
  };
}
