use azalea::pathfinder::PathfinderClientExt;
use salarixi_macros::Index;

use crate::bot::extensions::{BotDefaultExt, BotMovementExt};
use crate::bot::systems::index::INDEX_SYSTEM;
use crate::bot::systems::profile::BotStatus;
use crate::bot::systems::states::STATE_SYSTEM;
use crate::bot::systems::tasks::TASK_SYSTEM;
use crate::bot::PLUGINS;
use crate::server::transfer::*;
use crate::{sleep, take_bot, take_profile};

#[derive(Index)]
pub enum QuickAction {
  SendMessage = 0x00,
  Reset = 0x01,
  Disconnect = 0x02,
}

/// Вспомогательная функция отправки сообщения в чат от бота
pub async fn send_message_from_bot(username: String, message: String) {
  let Some(index) = INDEX_SYSTEM.index_by_username(&username).await else {
    return;
  };

  take_bot!(&index, async |bot| bot.chat(message));
}

/// Вспомогательная функция, позволяющая сбросить все задачи и состояния бота
pub async fn reset_bot(username: String) {
  let Some(index) = INDEX_SYSTEM.index_by_username(&username).await else {
    return;
  };

  TASK_SYSTEM.kill_all_tasks_for(&index).await;

  sleep!(1000);

  take_bot!(&index, async |bot| {
    bot.stop_move(&index).await;
    bot.stop_crouching();
    bot.stop_jumping();
    bot.stop_pathfinding();
  });

  STATE_SYSTEM.reset(&index).await;

  emit_log(format!("Все задачи и состояния бота {} сброшены", username), "info");

  emit_msg(
    "Управление",
    format!("Все задачи и состояния бота {} сброшены", username),
  );
}

/// Вспомогательная функция отключения бота
pub async fn disconnect_bot(username: String) {
  let Some(index) = INDEX_SYSTEM.index_by_username(&username).await else {
    return;
  };

  PLUGINS.kill_all_tasks_for(&index).await;
  TASK_SYSTEM.kill_all_tasks_for(&index).await;

  sleep!(2000);

  take_bot!(&index, async |bot| {
    bot.disconnect();
  });

  STATE_SYSTEM.remove(&index).await;
  TASK_SYSTEM.remove(&index).await;

  take_profile!(&index, async |profile| {
    profile.captcha_caught = false;
    profile.status = BotStatus::Disconnected;
  });

  emit_log(format!("Бот {} отключился", username), "info");
  emit_msg("Управление", format!("Бот {} отключился", username));
}
