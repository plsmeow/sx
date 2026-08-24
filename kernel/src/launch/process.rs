use std::sync::atomic::{AtomicBool, Ordering};

use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::bot::systems::index::INDEX_SYSTEM;
use crate::bot::systems::profile::PROFILE_SYSTEM;
use crate::server::transfer::{ProcessStatusPayload, TransferEvent, TRANSFER};

use super::options::LaunchOptions;

pub static PROCESS_ACTIVITY: AtomicBool = AtomicBool::new(false);
pub static BOTS_WERE_FULLY_CONNECTED: AtomicBool = AtomicBool::new(false);
pub static STOPPING: AtomicBool = AtomicBool::new(false);
pub static CURRENT_OPTIONS: Lazy<RwLock<Option<LaunchOptions>>> = Lazy::new(|| RwLock::new(None));

/// Функция установки опций
pub async fn set_options(options: LaunchOptions) {
  *CURRENT_OPTIONS.write().await = Some(options);
}

/// Функция получения текущих опций
pub async fn current_options() -> Option<LaunchOptions> {
  CURRENT_OPTIONS.read().await.clone()
}

/// Функция установки значения активности процесса
pub fn set_process_activity(value: bool) {
  PROCESS_ACTIVITY.store(value, Ordering::SeqCst);
}

/// Функция проверки активности основного процесса
pub fn process_is_active() -> bool {
  PROCESS_ACTIVITY.load(Ordering::SeqCst)
}

/// Вспомогательная функция формирования данных о статусе процесса
pub async fn process_status_payload() -> ProcessStatusPayload {
  let connected_bots = PROFILE_SYSTEM.get_connected_count().await as u8;
  let total_bots = INDEX_SYSTEM.map.read().await.len() as u8;
  let status_id;

  if STOPPING.load(Ordering::SeqCst) {
    status_id = 0x00;
  } else {
    if process_is_active() {
      if !BOTS_WERE_FULLY_CONNECTED.load(Ordering::SeqCst) && connected_bots == total_bots && total_bots > 0 {
        BOTS_WERE_FULLY_CONNECTED.store(true, Ordering::SeqCst);
      }

      if BOTS_WERE_FULLY_CONNECTED.load(Ordering::SeqCst) {
        status_id = 0x01;
      } else {
        status_id = 0x02;
      }
    } else {
      status_id = 0x03;
    }
  }

  ProcessStatusPayload {
    status_id,
    connected_bots,
    total_bots,
  }
}

/// Функция обновления статуса процесса
pub async fn update_process_status() {
  TRANSFER.emit(TransferEvent::ProcessStatus(process_status_payload().await));
}
