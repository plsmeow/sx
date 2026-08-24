use azalea::Client;
use salarixi_extensions::buffer::BufferExt;

use crate::bot::extensions::{BotInteractExt, BotInventoryExt, ClickMode};
use crate::bot::systems::index::INDEX_SYSTEM;
use crate::bot::traits::SalarixiModule;
use crate::server::transfer::*;
use crate::take_bot;

pub struct InventoryOptions {
  slot: Option<u16>,
  target_slot: Option<u16>,
  state: u8,
}

impl InventoryOptions {
  pub fn from_bytes(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      slot: Option::read(buf)?,
      target_slot: Option::read(buf)?,
      state: u8::read(buf)?,
    })
  }
}

pub struct InventoryModule;

impl InventoryModule {
  pub fn new() -> Self {
    Self
  }

  pub async fn interact(bot: &Client, index: &u8, options: &InventoryOptions) {
    let Some(slot) = options.slot else {
      return;
    };

    let slot = slot as usize;

    match options.state {
      0 => {
        if slot <= 8 {
          bot.set_selected_hotbar_slot(slot as u8);
        } else {
          if let Some(username) = INDEX_SYSTEM.username_by_index(index).await {
            emit_log(
              format!("Бот {} не смог взять слот {} (неверный индекс слота)", username, slot),
              "error",
            );
          }
        }
      }
      1 => {
        bot.inventory_click(index, slot, ClickMode::DropAll, true).await;
      }
      2 => {
        bot.inventory_click(index, slot, ClickMode::Left, true).await;
      }
      3 => {
        bot.inventory_click(index, slot, ClickMode::Right, true).await;
      }
      4 => {
        bot
          .inventory_swap_click(
            index,
            slot,
            if let Some(t) = options.target_slot {
              t as usize
            } else {
              0
            },
            true,
          )
          .await;
      }
      5 => {
        bot.start_use_item();
      }
      6 => {
        bot.release_use_item();
      }
      _ => {}
    }
  }
}

impl SalarixiModule<InventoryOptions> for InventoryModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: std::sync::Arc<InventoryOptions>) -> bool {
    tokio::spawn(async move {
      take_bot!(&index, async |bot| Self::interact(bot, &index, &options).await);
    });

    true
  }
}
