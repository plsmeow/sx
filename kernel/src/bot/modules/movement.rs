use azalea::prelude::*;
use azalea::WalkDirection;
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use salarixi_macros::Index;

use crate::bot::extensions::{go_to, BotMovementExt};
use crate::bot::systems::tasks::{gettskact, killtsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::sleep;
use crate::take_bot;
use crate::tools::*;

#[derive(Index)]
enum Mode {
  Default = 0,
  Pathfinder = 1,
}

#[derive(Index)]
enum Direction {
  Forward = 0,
  Backward = 1,
  Left = 2,
  Right = 3,
  ForwardLeft = 4,
  ForwardRight = 5,
  BackwardLeft = 6,
  BackwardRight = 7,
}

pub struct MovementOptions {
  mode: Mode,
  direction: Direction,
  x: Option<i32>,
  z: Option<i32>,
  use_sync: bool,
  use_impulsiveness: bool,
  state: u8,
}

impl MovementOptions {
  pub fn from_bytes(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      mode: Mode::from_index(u8::read(buf)?)?,
      direction: Direction::from_index(u8::read(buf)?)?,
      x: Option::read(buf)?,
      z: Option::read(buf)?,
      use_sync: bool::read(buf)?,
      use_impulsiveness: bool::read(buf)?,
      state: u8::read(buf)?,
    })
  }
}

pub struct MovementModule;

impl MovementModule {
  pub fn new() -> Self {
    Self
  }

  async fn default_move(bot: &Client, index: &u8, options: &MovementOptions) {
    if options.use_impulsiveness {
      if !options.use_sync {
        sleep!(randnum(500, 2000));
      }

      loop {
        let direction = match options.direction {
          Direction::Forward => WalkDirection::Forward,
          Direction::Backward => WalkDirection::Backward,
          Direction::Left => WalkDirection::Left,
          Direction::Right => WalkDirection::Right,
          Direction::ForwardLeft => WalkDirection::ForwardLeft,
          Direction::ForwardRight => WalkDirection::ForwardRight,
          Direction::BackwardLeft => WalkDirection::BackwardLeft,
          Direction::BackwardRight => WalkDirection::BackwardRight,
        };

        bot.start_walking(index, direction).await;

        if options.use_sync {
          sleep!(1200);
        } else {
          sleep!(randnum(800, 1800));
        }

        bot.stop_move(index).await;
      }
    } else {
      if !options.use_sync {
        sleep!(randnum(500, 2000));
      }

      let direction = match options.direction {
        Direction::Forward => WalkDirection::Forward,
        Direction::Backward => WalkDirection::Backward,
        Direction::Left => WalkDirection::Left,
        Direction::Right => WalkDirection::Right,
        Direction::ForwardLeft => WalkDirection::ForwardLeft,
        Direction::ForwardRight => WalkDirection::ForwardRight,
        Direction::BackwardLeft => WalkDirection::BackwardLeft,
        Direction::BackwardRight => WalkDirection::BackwardRight,
      };

      bot.start_walking(index, direction).await;
    }
  }

  async fn pathfinder_move(index: u8, options: &MovementOptions) {
    if let Some(x) = options.x {
      if let Some(z) = options.z {
        if !options.use_sync {
          sleep!(randnum(500, 2000));
        }

        go_to(index, x, z);
      }
    }
  }

  async fn start(bot: &Client, index: &u8, options: &MovementOptions) {
    match options.mode {
      Mode::Default => Self::default_move(bot, index, &options).await,
      Mode::Pathfinder => Self::pathfinder_move(*index, &options).await,
    }
  }
}

impl SalarixiModule<MovementOptions> for MovementModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: std::sync::Arc<MovementOptions>) -> bool {
    if options.state == 1 && !gettskact(&index, TaskName::Movement).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, &index, &options).await);
      });

      pushrtsk(&index, TaskName::Movement, task_handle).await;
    } else if options.state == 0 && gettskact(&index, TaskName::Movement).await {
      killtsk(&index, TaskName::Movement).await;

      take_bot!(&index, async |bot| {
        bot.stop_pathfinding();
        bot.stop_move(&index).await;
      });
    } else {
      return false;
    }

    true
  }
}
