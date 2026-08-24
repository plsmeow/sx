use hashbrown::HashMap;
use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::bot::plugins::*;
use crate::bot::systems::profile::PROFILE_SYSTEM;
use crate::bot::traits::SalarixiPlugin;
use crate::launch::options::PluginOptions;
use crate::server::transfer::emit_log;

pub static PLUGINS: Lazy<PluginManager> = Lazy::new(|| PluginManager::new());

/// Имя плагина
#[derive(Eq, Hash, PartialEq)]
pub enum PluginName {
  InstantArmorEquip,
  AutoTotem,
  AutoEat,
  PotionConsumer,
  AutoLook,
  AutoShield,
  AutoMending,
  PearlLeave,
}

/// Менеджер плагинов.
/// Изначально он хранит в себе объекты всех существующих плагинов и список их задач.
/// Данный менеджер выполняет роль загрузчика плагинов для бота.
pub struct PluginManager {
  tasks: RwLock<HashMap<u8, HashMap<PluginName, Option<JoinHandle<()>>>>>,
  plugins: PluginList,
}

struct PluginList {
  instant_armor_equip: InstantArmorEquipPlugin,
  auto_totem: AutoTotemPlugin,
  auto_eat: AutoEatPlugin,
  potion_consumer: PotionConsumerPlugin,
  auto_look: AutoLookPlugin,
  auto_shield: AutoShieldPlugin,
  auto_mending: AutoMendingPlugin,
  pearl_leave: PearlLeavePlugin,
}

impl PluginManager {
  /// Метод создания менеджера плагинов
  pub fn new() -> Self {
    Self {
      tasks: RwLock::new(HashMap::new()),
      plugins: PluginList {
        instant_armor_equip: InstantArmorEquipPlugin::new(),
        auto_totem: AutoTotemPlugin::new(),
        auto_eat: AutoEatPlugin::new(),
        potion_consumer: PotionConsumerPlugin::new(),
        auto_look: AutoLookPlugin::new(),
        auto_shield: AutoShieldPlugin::new(),
        auto_mending: AutoMendingPlugin::new(),
        pearl_leave: PearlLeavePlugin::new(),
      },
    }
  }

  /// Метод активации плагинов для указанного бота
  pub async fn activate_for(&'static self, index: u8, plugins: PluginOptions) {
    self.tasks.write().await.insert(index, HashMap::new());

    if plugins.instant_armor_equip {
      self.plugins.instant_armor_equip.activate(index);
    }

    if plugins.auto_totem {
      self.plugins.auto_totem.activate(index);
    }

    if plugins.auto_eat {
      self.plugins.auto_eat.activate(index);
    }

    if plugins.potion_consumer {
      self.plugins.potion_consumer.activate(index);
    }

    if plugins.auto_look {
      self.plugins.auto_look.activate(index);
    }

    if plugins.auto_shield {
      self.plugins.auto_shield.activate(index);
    }

    if plugins.auto_mending {
      self.plugins.auto_mending.activate(index);
    }

    if plugins.pearl_leave {
      self.plugins.pearl_leave.activate(index);
    }
  }

  /// Метод добавления задачи плагина указанному боту
  pub fn push_task(&self, index: &u8, plugin_name: PluginName, task: JoinHandle<()>) {
    if let Ok(mut guard) = self.tasks.try_write() {
      if let Some(tasks) = guard.get_mut(index) {
        if let Some(old_task) = tasks.get(&plugin_name) {
          if let Some(active_old_task) = old_task {
            active_old_task.abort();
          }
        }

        tasks.insert(plugin_name, Some(task));
      }
    }
  }

  /// Метод уничтожения всех задач плагинов указанного бота
  pub async fn kill_all_tasks_for(&self, index: &u8) {
    let mut map = self.tasks.write().await;

    if let Some(tasks) = map.get_mut(index) {
      for (_, t) in tasks {
        if let Some(h) = t {
          h.abort();
        }
      }
    }
  }

  /// Метод очистки всех задач
  pub async fn clear(&self) {
    for index in PROFILE_SYSTEM.get_all().await.keys() {
      self.kill_all_tasks_for(index).await;
    }

    emit_log("Активные задачи плагинов остановлены", "system");
  }
}
