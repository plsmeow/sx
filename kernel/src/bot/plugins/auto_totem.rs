use azalea::registry::builtin::ItemKind;

use crate::bot::extensions::BotInventoryExt;
use crate::bot::plugin_manager::PluginName;
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::{sleep, take_bot};

pub struct AutoTotemPlugin;

impl AutoTotemPlugin {
  /// Метод обработки тика
  async fn tick(&self, index: &u8) {
    take_bot!(index, async |bot| {
      let Some(menu) = bot.get_inventory_menu() else {
        return;
      };

      if let Some(offhand_item) = menu.slot(45) {
        if !offhand_item.is_empty() && offhand_item.kind() != ItemKind::Shield {
          sleep!(300);
          return;
        }
      }

      for (slot, item) in menu.slots().iter().enumerate() {
        if slot == 45 {
          continue;
        }

        if item.kind() == ItemKind::TotemOfUndying {
          bot
            .inventory_move_item(index, ItemKind::TotemOfUndying, slot, 45, true)
            .await;
          sleep!(800);
        }
      }
    });
  }
}

impl SalarixiPlugin for AutoTotemPlugin {
  fn new() -> Self {
    Self
  }

  fn activate(&'static self, index: u8) {
    let task = tokio::spawn(async move {
      loop {
        self.tick(&index).await;
        sleep!(150);
      }
    });

    PLUGINS.push_task(&index, PluginName::AutoTotem, task);
  }
}
