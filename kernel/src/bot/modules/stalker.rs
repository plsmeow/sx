use azalea::prelude::*;
use azalea::SprintDirection;
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use salarixi_macros::Index;

use crate::bot::extensions::{go_to, BotDefaultExt, BotMovementExt, EntityFilter};
use crate::bot::systems::states::{getst, setmst, State, StateName};
use crate::bot::systems::tasks::{gettskact, killtsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::{sleep, take_bot};

#[derive(Index)]
enum Mode {
  Default = 0,
  Advanced = 1,
}

pub struct StalkerOptions {
  mode: Mode,
  target_username: String,
  min_distance: Option<f64>,
  max_distance: Option<f64>,
  state: u8,
}

impl StalkerOptions {
  pub fn from_bytes(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      mode: Mode::from_index(u8::read(buf)?)?,
      target_username: String::read(buf)?,
      min_distance: Option::read(buf)?,
      max_distance: Option::read(buf)?,
      state: u8::read(buf)?,
    })
  }
}

pub struct StalkerModule;

impl StalkerModule {
  pub fn new() -> Self {
    Self
  }

  async fn default_stalking(bot: &Client, index: &u8, options: &StalkerOptions) {
    let entity_filter = EntityFilter::Custom(options.target_username.clone());

    loop {
      if let Some(target) = bot.find_nearest_entity(&entity_filter, options.max_distance.unwrap_or(100.0)) {
        let target_pos = bot.get_entity_position(target);
        let min_distance = options.min_distance.unwrap_or(6.0);

        let Some(foot_pos) = bot.foot_pos() else {
          sleep!(200);
          continue;
        };

        if foot_pos.distance_to(target_pos) > min_distance {
          bot.look_at(target_pos);
          bot.start_jumping();
          bot.start_sprinting(index, SprintDirection::Forward).await;
        } else {
          bot.stop_jumping();

          if getst(index, State::IsSprinting).await {
            bot.stop_move(index).await;
          }
        }
      } else {
        bot.stop_jumping();

        if getst(index, State::IsSprinting).await {
          bot.stop_move(index).await;
        }
      }

      sleep!(50);
    }
  }

  async fn advanced_stalking(bot: &Client, index: u8, options: &StalkerOptions) {
    let entity_filter = EntityFilter::Custom(options.target_username.clone());

    loop {
      if let Some(target) = bot.find_nearest_entity(&entity_filter, options.max_distance.unwrap_or(100.0)) {
        let target_pos = bot.get_entity_position(target);
        let min_distance = options.min_distance.unwrap_or(6.0);

        let Some(foot_pos) = bot.foot_pos() else {
          sleep!(200);
          continue;
        };

        if foot_pos.distance_to(target_pos) > min_distance {
          if bot.is_goto_target_reached() {
            go_to(
              index,
              (target_pos.x - min_distance) as i32,
              (target_pos.z - min_distance) as i32,
            );
          }
        }
      }

      sleep!(200);
    }
  }

  async fn start(bot: &Client, index: &u8, options: &StalkerOptions) {
    setmst(index, StateName::Looking, true).await;

    match options.mode {
      Mode::Default => Self::default_stalking(bot, &index, options).await,
      Mode::Advanced => Self::advanced_stalking(bot, *index, options).await,
    }
  }
}

impl SalarixiModule<StalkerOptions> for StalkerModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: std::sync::Arc<StalkerOptions>) -> bool {
    if options.state == 1 && !gettskact(&index, TaskName::Stalker).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, &index, &options).await);
      });

      pushrtsk(&index, TaskName::Stalker, task_handle).await;
    } else if options.state == 0 && gettskact(&index, TaskName::Stalker).await {
      killtsk(&index, TaskName::Stalker).await;

      take_bot!(&index, async |bot| {
        bot.stop_move(&index).await;
        bot.stop_jumping();
      });

      setmst(&index, StateName::Looking, false).await;
    } else {
      return false;
    }

    true
  }
}
