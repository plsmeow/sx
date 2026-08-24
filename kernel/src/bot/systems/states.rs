use hashbrown::HashMap;
use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::server::transfer::emit_log;

pub static STATE_SYSTEM: Lazy<StateSystem> = Lazy::new(|| StateSystem::new());

const DEFAULT_STATES: [u8; 14] = [1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0];

/// Состояние бота
pub enum State {
  CanWalking,
  CanSprinting,
  CanEating,
  CanDrinking,
  CanAttacking,
  CanLooking,
  CanInteracting,
  IsWalking,
  IsSprinting,
  IsEating,
  IsDrinking,
  IsAttacking,
  IsLooking,
  IsInteracting,
}

/// Имя состояния бота
pub enum StateName {
  Walking,
  Sprinting,
  Eating,
  Drinking,
  Attacking,
  Looking,
  Interacting,
}

impl State {
  /// Метод конвертации строки в состояние
  pub fn from_str<'a>(str: impl Into<&'a str>) -> Option<Self> {
    match str.into() {
      "can_walking" => Some(Self::CanWalking),
      "can_sprinting" => Some(Self::CanSprinting),
      "can_eating" => Some(Self::CanEating),
      "can_drinking" => Some(Self::CanDrinking),
      "can_attacking" => Some(Self::CanAttacking),
      "can_looking" => Some(Self::CanLooking),
      "can_interacting" => Some(Self::CanInteracting),
      "is_walking" => Some(Self::IsWalking),
      "is_sprinting" => Some(Self::IsSprinting),
      "is_eating" => Some(Self::IsEating),
      "is_drinking" => Some(Self::IsDrinking),
      "is_attacking" => Some(Self::IsAttacking),
      "is_looking" => Some(Self::IsLooking),
      "is_interacting" => Some(Self::IsInteracting),
      _ => None,
    }
  }

  /// Метод получени индекса состояния
  pub fn index(&self) -> usize {
    match self {
      Self::CanWalking => 0,
      Self::CanSprinting => 1,
      Self::CanEating => 2,
      Self::CanDrinking => 3,
      Self::CanAttacking => 4,
      Self::CanLooking => 5,
      Self::CanInteracting => 6,
      Self::IsWalking => 7,
      Self::IsSprinting => 8,
      Self::IsEating => 9,
      Self::IsDrinking => 10,
      Self::IsAttacking => 11,
      Self::IsLooking => 12,
      Self::IsInteracting => 13,
    }
  }
}

/// Структура состояний бота
pub struct States([u8; 14]);

impl States {
  /// Метод создания состояний по умолчанию
  pub fn new() -> Self {
    Self(DEFAULT_STATES)
  }

  /// Метод получения значения указанного состояния
  pub fn get(&self, state: State) -> bool {
    if let Some(b) = self.0.get(state.index()) {
      return *b == 1;
    }

    false
  }

  /// Метод установки значения для указанного состояния
  pub fn set(&mut self, state: State, value: bool) {
    self.0[state.index()] = if value == true { 1 } else { 0 };
  }
}

/// Система управления состояниями ботов
pub struct StateSystem {
  pub map: RwLock<HashMap<u8, States>>,
}

impl StateSystem {
  /// Метод создания системы состояний
  pub fn new() -> Self {
    Self {
      map: RwLock::new(HashMap::new()),
    }
  }

  /// Метод регистрации бота
  pub async fn register(&self, index: u8) {
    self.map.write().await.insert(index, States::new());
  }

  /// Метод очистки хранилища состояний
  pub async fn clear(&self) {
    self.map.write().await.clear();
    emit_log("Состояния ботов очищены", "system");
  }

  /// Метод сброса состояний указанного бота
  pub async fn reset(&self, index: &u8) {
    let mut guard = self.map.write().await;
    if let Some(s) = guard.get_mut(index) {
      s.0 = DEFAULT_STATES;
    }
  }

  /// Метод удаления всех состояний указанного бота
  pub async fn remove(&self, index: &u8) {
    self.map.write().await.remove(index);
  }
}

/// Вспомогательная функция получения состояния
pub async fn getst(index: &u8, state: State) -> bool {
  let guard = STATE_SYSTEM.map.read().await;
  if let Some(s) = guard.get(index) {
    return s.get(state);
  }

  false
}

/// Вспомогательная функция установки значения для состояния
pub async fn setst(index: &u8, state: State, value: bool) {
  let mut guard = STATE_SYSTEM.map.write().await;
  if let Some(s) = guard.get_mut(index) {
    s.set(state, value);
  }
}

/// Вспомогательная функция взаимной установки значений для `can` и `is` состояний
pub async fn setmst(index: &u8, state_name: StateName, current_value: bool) {
  let (is_state, can_state) = match state_name {
    StateName::Walking => (State::IsWalking, State::CanWalking),
    StateName::Sprinting => (State::IsSprinting, State::CanSprinting),
    StateName::Eating => (State::IsEating, State::CanEating),
    StateName::Drinking => (State::IsDrinking, State::CanDrinking),
    StateName::Attacking => (State::IsAttacking, State::CanAttacking),
    StateName::Looking => (State::IsLooking, State::CanLooking),
    StateName::Interacting => (State::IsInteracting, State::CanInteracting),
  };

  let mut guard = STATE_SYSTEM.map.write().await;
  if let Some(s) = guard.get_mut(index) {
    s.set(is_state, current_value);
    s.set(can_state, !current_value);
  }
}

/// Вспомогательная функция попытки получения состояния
pub fn try_getst(index: &u8, state: State) -> bool {
  if let Ok(states) = STATE_SYSTEM.map.try_read() {
    if let Some(s) = states.get(index) {
      return s.get(state);
    }
  }

  false
}

/// Вспомогательная функция попытки установки значения для состояния
pub fn try_setst(index: &u8, state: State, value: bool) {
  if let Ok(mut states) = STATE_SYSTEM.map.try_write() {
    if let Some(s) = states.get_mut(index) {
      s.set(state, value);
    }
  }
}
