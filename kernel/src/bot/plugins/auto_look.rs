use azalea::prelude::*;

use crate::bot::extensions::{BotDefaultExt, BotRotationExt, EntityFilter};
use crate::bot::plugin_manager::PluginName;
use crate::bot::systems::states::{getst, setmst, State, StateName};
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::tools::randnum;
use crate::{sleep, take_bot};

pub struct AutoLookPlugin;

impl AutoLookPlugin {
  /// Метод обработки тика
  async fn tick(&self, index: &u8) {
    if !getst(index, State::CanLooking).await {
      return;
    }

    setmst(index, StateName::Looking, true).await;

    take_bot!(index, async |bot| {
      if !bot.is_goto_target_reached() {
        return;
      }

      if let Some(entity) = bot.find_nearest_entity(&EntityFilter::Any, 14.0) {
        bot.look_at_entity(entity, true);
      }
    });

    sleep!(randnum(50, 100));

    setmst(index, StateName::Looking, false).await;
  }
}

impl SalarixiPlugin for AutoLookPlugin {
  fn new() -> Self {
    Self
  }

  fn activate(&'static self, index: u8) {
    let task = tokio::spawn(async move {
      loop {
        self.tick(&index).await;
        sleep!(100);
      }
    });

    PLUGINS.push_task(&index, PluginName::AutoLook, task);
  }
}
