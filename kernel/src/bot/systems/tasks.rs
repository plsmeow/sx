use hashbrown::HashMap;
use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::server::transfer::emit_log;

pub static TASK_SYSTEM: Lazy<TaskSystem> = Lazy::new(|| TaskSystem::new());

/// Название задачи бота
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum TaskName {
  Spamming,
  Movement,
  Jumping,
  Shifting,
  Waving,
  AntiAfk,
  Stalker,
  Flight,
  Killaura,
  Scaffold,
  AntiFall,
  BowAim,
  Stealer,
  Miner,
  Farmer,
}

/// Структура задач бота
pub struct Tasks {
  pub root: HashMap<TaskName, JoinHandle<()>>,
  pub extra: HashMap<TaskName, Vec<JoinHandle<()>>>,
}

impl Tasks {
  pub fn new() -> Self {
    Self {
      root: HashMap::new(),
      extra: HashMap::new(),
    }
  }

  /// Метод получения активности корневой задачи
  pub fn get_root_task_activity(&self, name: TaskName) -> bool {
    self.root.contains_key(&name)
  }

  /// Метод добавления корневой задачи
  pub fn pushrtsk(&mut self, name: TaskName, handle: JoinHandle<()>) {
    self.root.insert(name.clone(), handle);
  }

  /// Метод добавления дополнительной задачи
  pub fn pushetsk(&mut self, name: TaskName, handle: JoinHandle<()>) {
    if let Some(tasks) = self.extra.get_mut(&name) {
      tasks.push(handle);
    } else {
      self.extra.insert(name, vec![handle]);
    }
  }

  /// Метод остановки корневой задачи и её дополнительных задач
  pub fn killtsk(&mut self, name: TaskName) {
    if let Some(mut tasks) = self.extra.remove(&name) {
      tasks.iter().for_each(|t| t.abort());
      tasks.clear();
    }

    if let Some(handle) = self.root.remove(&name) {
      handle.abort();
    }
  }

  /// Метод остановки всех задач
  pub fn kill_all_tasks(&mut self) {
    self
      .extra
      .iter()
      .for_each(|(_, tasks)| tasks.iter().for_each(|t| t.abort()));

    self.root.iter().for_each(|(_, t)| t.abort());

    self.extra.clear();
    self.root.clear();
  }
}

/// Система управления задачами
pub struct TaskSystem {
  pub map: RwLock<HashMap<u8, Tasks>>,
}

impl TaskSystem {
  pub fn new() -> Self {
    Self {
      map: RwLock::new(HashMap::new()),
    }
  }

  /// Метод регистрации бота
  pub async fn register(&self, index: u8) {
    let mut guard = self.map.write().await;
    if let Some(t) = guard.get_mut(&index) {
      t.kill_all_tasks();
    }

    guard.insert(index, Tasks::new());
  }

  /// Метод остановки всех задач указанного бота
  pub async fn kill_all_tasks_for(&self, index: &u8) {
    let mut guard = self.map.write().await;
    if let Some(t) = guard.get_mut(index) {
      t.kill_all_tasks();
    }
  }

  /// Метод удаления бота
  pub async fn remove(&self, index: &u8) {
    self.map.write().await.remove(index);
  }

  /// Метод остановки всех активных задач и очистки хранилища
  pub async fn clear(&self) {
    let mut guard = self.map.write().await;
    guard.iter_mut().for_each(|(_, t)| t.kill_all_tasks());
    guard.clear();

    emit_log("Активные задачи ботов остановлены", "system");
  }
}

/// Вспомогательная функция получения активности определённой задачи указанного бота
pub async fn gettskact(index: &u8, task_name: TaskName) -> bool {
  let guard = TASK_SYSTEM.map.read().await;

  if let Some(t) = guard.get(index) {
    return t.get_root_task_activity(task_name);
  }

  false
}

/// Вспомогательная функция добавления корневой задачи указанному бота
pub async fn pushrtsk(index: &u8, task_name: TaskName, root_task: JoinHandle<()>) {
  let mut guard = TASK_SYSTEM.map.write().await;
  if let Some(t) = guard.get_mut(index) {
    t.pushrtsk(task_name, root_task);
  }
}

/// Вспомогательная функция добавления дополнительной задачи указанному бота
pub async fn pushetsk(index: &u8, task_name: TaskName, extra_task: JoinHandle<()>) {
  let mut guard = TASK_SYSTEM.map.write().await;
  if let Some(t) = guard.get_mut(index) {
    t.pushetsk(task_name, extra_task);
  }
}

/// Вспомогательная функция остановки определённой задачи указанного бота
pub async fn killtsk(index: &u8, task_name: TaskName) {
  let mut guard = TASK_SYSTEM.map.write().await;
  if let Some(t) = guard.get_mut(index) {
    t.killtsk(task_name);
  }
}
