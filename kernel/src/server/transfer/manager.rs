use once_cell::sync::Lazy;
use tokio::sync::broadcast;

use crate::server::transfer::{
  AntiMapCaptchaPayload, AntiWebCaptchaPayload, BotChatPayload, LogPayload, MessagePayload, ProcessStatusPayload,
  SessionChatPayload, UpdateBotProfilePayload,
};

pub static TRANSFER: Lazy<TransferManager> = Lazy::new(|| TransferManager::new());

#[derive(Clone)]
pub enum TransferEvent {
  Log(LogPayload),
  Message(MessagePayload),
  SessionChat(SessionChatPayload),
  BotChat(BotChatPayload),
  AntiWebCaptcha(AntiWebCaptchaPayload),
  AntiMapCaptcha(AntiMapCaptchaPayload),
  UpdateBotProfile(UpdateBotProfilePayload),
  ProcessStatus(ProcessStatusPayload),
}

pub struct TransferManager {
  pub tx: broadcast::Sender<TransferEvent>,
}

impl TransferManager {
  pub fn new() -> Self {
    let (tx, _) = broadcast::channel(255);

    Self { tx }
  }

  pub fn emit(&self, event: TransferEvent) {
    if self.tx.receiver_count() > 0 {
      let _ = self.tx.send(event);
    }
  }
}

/// Вспомогательная функция отправки лога
pub fn emit_log(text: impl Into<String>, class: impl Into<String>) {
  TRANSFER.emit(TransferEvent::Log(LogPayload {
    text: text.into(),
    class: class.into(),
  }));
}

/// Вспомогательная функция отправки сообщения
pub fn emit_msg(name: impl Into<String>, content: impl Into<String>) {
  TRANSFER.emit(TransferEvent::Message(MessagePayload {
    name: name.into(),
    content: content.into(),
  }));
}
