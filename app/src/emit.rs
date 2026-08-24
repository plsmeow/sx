use once_cell::sync::Lazy;
use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast;

pub static EMIT_MANAGER: Lazy<EmitManager> = Lazy::new(|| EmitManager::new());

pub struct EmitManager {
  tx: broadcast::Sender<(&'static str, Vec<u8>)>,
}

impl EmitManager {
  pub fn new() -> Self {
    let (tx, _) = broadcast::channel(255);

    Self { tx }
  }

  /// Метод отправки данных на фронт
  pub fn emit(&self, id: &'static str, data: Vec<u8>) {
    let _ = self.tx.send((id, data));
  }
}

pub async fn emit_event_loop(handle: AppHandle) {
  let mut rx = EMIT_MANAGER.tx.subscribe();

  while let Ok((id, data)) = rx.recv().await {
    let _ = handle.emit(id, data);
  }
}
