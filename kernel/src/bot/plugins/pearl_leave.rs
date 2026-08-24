use std::time::Duration;

use azalea::prelude::*;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::registry::builtin::ItemKind;

use crate::bot::extensions::{BotDefaultExt, BotInteractExt, BotInventoryExt};
use crate::bot::plugin_manager::PluginName;
use crate::bot::systems::states::{getst, setmst, State, StateName};
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::tools::randnum;
use crate::{sleep, take_bot};

pub struct PearlLeavePlugin;

impl PearlLeavePlugin {
  /// Метод обработки тика
  async fn tick(&self, index: &u8) {
    take_bot!(index, async |bot| {
      let health = bot.get_health();

      if health < 6.0 || health > 10.0 {
        return;
      }

      if !self.take_pearl(index).await {
        return;
      }

      self.throw_pearl(index).await;
    });
  }

  /// Метод взятия жемчуга в руку
  async fn take_pearl(&self, index: &u8) -> bool {
    let mut pearl_taken = false;

    take_bot!(index, async |bot| {
      let Some(menu) = bot.get_inventory_menu() else {
        return;
      };

      for (slot, item) in menu.slots().iter().enumerate() {
        if item.kind() != ItemKind::EnderPearl {
          continue;
        }

        bot.take_item(index, slot, true).await;

        pearl_taken = true;
      }
    });

    pearl_taken
  }

  /// Метод броска жемчуга
  async fn throw_pearl(&self, index: &u8) {
    if !getst(index, State::CanLooking).await {
      return;
    }

    setmst(index, StateName::Looking, true).await;

    let y_rot = randnum(-180.0, 180.0);
    let x_rot = randnum(-35.0, -30.0);

    take_bot!(index, async |bot| {
      let _ = bot.set_direction(y_rot, x_rot);
    });

    setmst(index, StateName::Looking, false).await;

    sleep!(randnum(100, 150));

    take_bot!(index, async |bot| {
      bot.jump();
    });

    sleep!(randnum(100, 150));

    if !getst(index, State::CanInteracting).await {
      return;
    }

    setmst(index, StateName::Interacting, true).await;

    take_bot!(index, async |bot| {
      bot.start_use_item_by(InteractionHand::MainHand);
    });

    setmst(index, StateName::Interacting, false).await;

    tokio::time::sleep(Duration::from_secs(8)).await;
  }
}

impl SalarixiPlugin for PearlLeavePlugin {
  fn new() -> Self {
    Self
  }

  fn activate(&'static self, index: u8) {
    let task = tokio::spawn(async move {
      loop {
        self.tick(&index).await;
        sleep!(200);
      }
    });

    PLUGINS.push_task(&index, PluginName::PearlLeave, task);
  }
}
