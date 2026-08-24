use azalea::prelude::*;
use azalea::WalkDirection;
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use salarixi_macros::Index;

use crate::bot::extensions::{BotDefaultExt, BotMovementExt};
use crate::bot::systems::states::{setmst, StateName};
use crate::bot::systems::tasks::{gettskact, killtsk, pushetsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::sleep;
use crate::take_bot;
use crate::tools::*;

#[derive(Index)]
enum Mode {
  Minimal = 0,
  Normal = 1,
  Advanced = 2,
}

pub struct AntiAfkOptions {
  mode: Mode,
  min_delay: Option<u64>,
  max_delay: Option<u64>,
  state: u8,
}

impl AntiAfkOptions {
  pub fn from_bytes(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      mode: Mode::from_index(u8::read(buf)?)?,
      min_delay: Option::read(buf)?,
      max_delay: Option::read(buf)?,
      state: u8::read(buf)?,
    })
  }
}

pub struct AntiAfkModule;

impl AntiAfkModule {
  pub fn new() -> Self {
    Self
  }

  async fn minimal(bot: &Client, options: &AntiAfkOptions) {
    let min_delay = options.min_delay.unwrap_or(1000);
    let max_delay = options.max_delay.unwrap_or(2500);

    loop {
      let original_direction = bot.direction();

      bot.set_direction(
        original_direction.0 + randnum(-50.0, 50.0) as f32,
        original_direction.1 + randnum(-12.0, 12.0) as f32,
      );

      if randchance(0.4) {
        bot.swing_arm();
      }

      sleep!(randnum(400, 800));

      bot.set_direction(
        original_direction.0 + randnum(-10.0, 10.0) as f32,
        original_direction.1 + randnum(-4.5, 4.5) as f32,
      );

      sleep!(randnum(min_delay, max_delay));
    }
  }

  async fn normal(bot: &Client, index: &u8, options: &AntiAfkOptions) {
    let min_delay = options.min_delay.unwrap_or(1000);
    let max_delay = options.max_delay.unwrap_or(2500);

    loop {
      let walk_directions = vec![
        WalkDirection::Forward,
        WalkDirection::Backward,
        WalkDirection::Left,
        WalkDirection::Right,
        WalkDirection::ForwardLeft,
        WalkDirection::ForwardRight,
        WalkDirection::BackwardLeft,
        WalkDirection::BackwardRight,
      ];

      let walk_direction = randelem(&walk_directions);

      bot.start_walking(index, *walk_direction).await;

      let direction = bot.direction();

      bot.set_direction(
        direction.0 + randnum(-35.0, 35.0) as f32,
        direction.1 + randnum(-20.0, 20.0) as f32,
      );

      sleep!(randnum(150, 400));

      bot.stop_move(index).await;

      sleep!(randnum(min_delay, max_delay));
    }
  }

  async fn advanced(bot: &Client, index: u8, options: &AntiAfkOptions) {
    let min_delay = options.min_delay.unwrap_or(1000);
    let max_delay = options.max_delay.unwrap_or(2500);

    let extra_task = tokio::spawn(async move {
      loop {
        take_bot!(&index, async |bot| {
          let direction = bot.direction();

          bot.set_direction(
            direction.0 + randnum(-35.0, 35.0) as f32,
            direction.1 + randnum(-10.0, 10.0) as f32,
          );
        });

        sleep!(randnum(100, 400));
      }
    });

    pushetsk(&index, TaskName::AntiAfk, extra_task).await;

    loop {
      bot.start_walking(&index, WalkDirection::Forward).await;

      if randchance(0.4) {
        bot.swing_arm();
      }

      if randchance(0.4) {
        bot.start_crouching();
        sleep!(randnum(300, 500));
        bot.stop_crouching();
      }

      if randchance(0.3) {
        bot.jump();
      }

      sleep!(randnum(min_delay, max_delay));
    }
  }

  async fn start(bot: &Client, index: &u8, options: &AntiAfkOptions) {
    match options.mode {
      Mode::Minimal => Self::minimal(bot, options).await,
      Mode::Normal => Self::normal(bot, index, options).await,
      Mode::Advanced => Self::advanced(bot, *index, options).await,
    }
  }
}

impl SalarixiModule<AntiAfkOptions> for AntiAfkModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: std::sync::Arc<AntiAfkOptions>) -> bool {
    if options.state == 1 && !gettskact(&index, TaskName::AntiAfk).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, &index, &options).await);
      });

      pushrtsk(&index, TaskName::AntiAfk, task_handle).await;
    } else if options.state == 0 && gettskact(&index, TaskName::AntiAfk).await {
      killtsk(&index, TaskName::AntiAfk).await;

      take_bot!(&index, async |bot| {
        bot.stop_move(&index).await;
        bot.stop_crouching();
      });

      setmst(&index, StateName::Looking, false).await;
    } else {
      return false;
    }

    true
  }
}
