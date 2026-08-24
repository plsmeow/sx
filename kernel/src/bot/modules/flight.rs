use std::time::{Duration, Instant};

use azalea::core::position::BlockPos;
use azalea::prelude::*;
use azalea::protocol::common::movements::MoveFlags;
use azalea::protocol::packets::game::ServerboundMovePlayerPos;
use azalea::Vec3;
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use salarixi_macros::Index;

use crate::bot::extensions::{BotDefaultExt, BotPhysicsExt};
use crate::bot::systems::tasks::{gettskact, killtsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::tools::*;
use crate::{sleep, take_bot};

#[derive(Index)]
enum Mode {
  Vanilla = 0,
  JumpFly = 1,
  TeleportFly = 2,
  BugFly = 3,
}

#[derive(PartialEq, Index)]
enum Settings {
  Adaptive = 0,
  Manual = 1,
}

#[derive(Index)]
enum AntiCheat {
  Vulcan = 0,
  Grim = 1,
  Intave = 2,
  None = 3,
}

pub struct FlightOptions {
  mode: Mode,
  settings: Settings,
  anti_cheat: AntiCheat,
  min_delay: Option<u64>,
  max_delay: Option<u64>,
  min_change_y: Option<f64>,
  max_change_y: Option<f64>,
  use_ground_spoof: Option<String>,
  state: u8,
}

impl FlightOptions {
  pub fn from_bytes(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      mode: Mode::from_index(u8::read(buf)?)?,
      settings: Settings::from_index(u8::read(buf)?)?,
      anti_cheat: AntiCheat::from_index(u8::read(buf)?)?,
      min_delay: Option::read(buf)?,
      max_delay: Option::read(buf)?,
      min_change_y: Option::read(buf)?,
      max_change_y: Option::read(buf)?,
      use_ground_spoof: Option::read(buf)?,
      state: u8::read(buf)?,
    })
  }
}

struct FlightConfig {
  min_delay: u64,
  max_delay: u64,
  min_change_y: f64,
  max_change_y: f64,
  use_ground_spoof: bool,
  use_jitter: bool,
}

pub struct FlightModule;

impl FlightModule {
  pub fn new() -> Self {
    Self
  }

  fn create_adaptive_config(anti_cheat: &AntiCheat) -> FlightConfig {
    let config;

    match anti_cheat {
      AntiCheat::Vulcan => {
        config = FlightConfig {
          min_delay: 100,
          max_delay: 250,
          min_change_y: 0.004,
          max_change_y: 0.007,
          use_ground_spoof: true,
          use_jitter: true,
        };
      }
      AntiCheat::Intave => {
        config = FlightConfig {
          min_delay: 200,
          max_delay: 400,
          min_change_y: 0.0025,
          max_change_y: 0.0048,
          use_ground_spoof: true,
          use_jitter: true,
        };
      }
      AntiCheat::Grim => {
        config = FlightConfig {
          min_delay: 100,
          max_delay: 200,
          min_change_y: 0.0011,
          max_change_y: 0.0018,
          use_ground_spoof: false,
          use_jitter: false,
        };
      }
      AntiCheat::None => {
        config = FlightConfig {
          min_delay: 200,
          max_delay: 500,
          min_change_y: 0.065,
          max_change_y: 0.087,
          use_ground_spoof: true,
          use_jitter: true,
        };
      }
    }

    config
  }

  async fn hover(bot: &Client, time: Instant) {
    loop {
      if Instant::now() >= time {
        break;
      }

      bot.set_velocity("y", 0.0);

      sleep!(50);
    }
  }

  pub async fn vanilla_flight(bot: &Client, options: &FlightOptions) {
    let config = if options.settings == Settings::Adaptive {
      Self::create_adaptive_config(&options.anti_cheat)
    } else {
      FlightConfig {
        min_delay: options.min_delay.unwrap_or(200),
        max_delay: options.max_delay.unwrap_or(400),
        min_change_y: options.min_change_y.unwrap_or(0.05),
        max_change_y: options.max_change_y.unwrap_or(0.08),
        use_ground_spoof: if let Some(v) = &options.use_ground_spoof {
          v.as_str() == "true"
        } else {
          true
        },
        use_jitter: false,
      }
    };

    loop {
      if config.use_ground_spoof {
        bot.set_on_ground(true);
      }

      sleep!(randnum(50, 80));

      if config.use_jitter {
        for _ in 0..randnum(4, 6) {
          bot.set_velocity("y", randnum(config.min_change_y, config.max_change_y));
          sleep!(50);
        }
      } else {
        bot.set_velocity("y", randnum(config.min_change_y, config.max_change_y));
      }

      if config.use_ground_spoof {
        bot.set_on_ground(false);
      }

      sleep!(randnum(config.min_delay, config.max_delay));

      Self::hover(bot, Instant::now() + Duration::from_millis(randnum(100, 150))).await;

      if config.use_ground_spoof {
        bot.set_on_ground(false);
      }
    }
  }

  pub async fn jump_flight(bot: &Client, options: &FlightOptions) {
    let config = if options.settings == Settings::Adaptive {
      Self::create_adaptive_config(&options.anti_cheat)
    } else {
      FlightConfig {
        min_delay: options.min_delay.unwrap_or(200),
        max_delay: options.max_delay.unwrap_or(350),
        min_change_y: options.min_change_y.unwrap_or(0.006),
        max_change_y: options.max_change_y.unwrap_or(0.008),
        use_ground_spoof: if let Some(v) = &options.use_ground_spoof {
          v.as_str() == "true"
        } else {
          true
        },
        use_jitter: true,
      }
    };

    let mut counter = 0;

    loop {
      let Some(physics) = bot.get_physics() else {
        continue;
      };

      counter += 1;

      if config.use_ground_spoof {
        bot.set_on_ground(true);
      }

      if counter == 2 {
        bot.jump();
        counter = 0;
      }

      sleep!(randnum(50, 80));

      let Some(foot_pos) = bot.foot_pos() else {
        continue;
      };

      let packet = ServerboundMovePlayerPos {
        pos: Vec3::new(
          foot_pos.x,
          foot_pos.y + randnum(config.min_change_y + 0.8, config.max_change_y + 0.8),
          foot_pos.z,
        ),
        flags: MoveFlags {
          on_ground: physics.on_ground(),
          horizontal_collision: physics.horizontal_collision,
        },
      };

      if config.use_jitter {
        for _ in 0..randnum(4, 6) {
          bot.write_packet(packet.clone());
          sleep!(50);
        }
      } else {
        bot.write_packet(packet);
      }

      sleep!(randnum(config.min_delay, config.max_delay));

      Self::hover(bot, Instant::now() + Duration::from_millis(randnum(100, 150))).await;

      if config.use_ground_spoof {
        bot.set_on_ground(false);
      }
    }
  }

  pub async fn teleport_flight(bot: &Client, options: &FlightOptions) {
    let config = if options.settings == Settings::Adaptive {
      Self::create_adaptive_config(&options.anti_cheat)
    } else {
      FlightConfig {
        min_delay: options.min_delay.unwrap_or(150),
        max_delay: options.max_delay.unwrap_or(400),
        min_change_y: options.min_change_y.unwrap_or(0.08),
        max_change_y: options.max_change_y.unwrap_or(0.09),
        use_ground_spoof: if let Some(v) = &options.use_ground_spoof {
          v.as_str() == "true"
        } else {
          true
        },
        use_jitter: false,
      }
    };

    loop {
      sleep!(randnum(50, 100));

      let Some(physics) = bot.get_physics() else {
        continue;
      };

      if config.use_ground_spoof {
        bot.set_on_ground(true);
      }

      let Some(foot_pos) = bot.foot_pos() else {
        continue;
      };

      let packet = ServerboundMovePlayerPos {
        pos: Vec3::new(
          foot_pos.x,
          foot_pos.y + randnum(config.min_change_y + 0.8, config.max_change_y + 0.8),
          foot_pos.z,
        ),
        flags: MoveFlags {
          on_ground: physics.on_ground(),
          horizontal_collision: physics.horizontal_collision,
        },
      };

      if config.use_jitter {
        for _ in 0..randnum(4, 6) {
          bot.write_packet(packet.clone());
          sleep!(50);
        }
      } else {
        bot.write_packet(packet);
      }

      sleep!(randnum(config.min_delay, config.max_delay));

      Self::hover(bot, Instant::now() + Duration::from_millis(randnum(100, 150))).await;

      if config.use_ground_spoof {
        bot.set_on_ground(false);
      }
    }
  }

  pub async fn bug_flight(bot: &Client, options: &FlightOptions) {
    let config = if options.settings == Settings::Adaptive {
      Self::create_adaptive_config(&options.anti_cheat)
    } else {
      FlightConfig {
        min_delay: options.min_delay.unwrap_or(200),
        max_delay: options.max_delay.unwrap_or(400),
        min_change_y: options.min_change_y.unwrap_or(0.02),
        max_change_y: options.max_change_y.unwrap_or(0.05),
        use_ground_spoof: if let Some(v) = &options.use_ground_spoof {
          v.as_str() == "true"
        } else {
          true
        },
        use_jitter: true,
      }
    };

    loop {
      if config.use_ground_spoof {
        bot.set_on_ground(true);
      }

      let Some(foot_pos) = bot.foot_pos() else {
        continue;
      };

      let block_under = BlockPos::new(foot_pos.x as i32, (foot_pos.y - 1.0) as i32, foot_pos.z as i32);

      let distance_vec = Vec3::new(
        foot_pos.x - (block_under.x as f64 + 0.5),
        foot_pos.y - (block_under.y as f64 + 0.5),
        foot_pos.z - (block_under.z as f64 + 0.5),
      );

      let distance = distance_vec.length().max(0.1);
      let direction = distance_vec.normalize();

      let strength = (4.0 / distance).min(2.0);

      let variation = randnum(-0.2, 0.2);
      let final_strength = strength + variation;

      if config.use_jitter {
        for _ in 0..randnum(4, 6) {
          bot.set_velocity("y", direction.y.abs() * final_strength * 0.2);
          sleep!(50);
        }
      } else {
        bot.set_velocity("y", direction.y.abs() * final_strength * 0.2);
      }

      sleep!(randnum(config.min_delay, config.max_delay));

      Self::hover(bot, Instant::now() + Duration::from_millis(randnum(600, 800))).await;

      if config.use_ground_spoof {
        bot.set_on_ground(false);
      }
    }
  }

  async fn start(bot: &Client, options: &FlightOptions) {
    match options.mode {
      Mode::Vanilla => Self::vanilla_flight(bot, options).await,
      Mode::JumpFly => Self::jump_flight(bot, options).await,
      Mode::TeleportFly => Self::teleport_flight(bot, options).await,
      Mode::BugFly => Self::bug_flight(bot, options).await,
    }
  }
}

impl SalarixiModule<FlightOptions> for FlightModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: std::sync::Arc<FlightOptions>) -> bool {
    if options.state == 1 && !gettskact(&index, TaskName::Flight).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, &options).await);
      });

      pushrtsk(&index, TaskName::Flight, task_handle).await;
    } else if options.state == 0 && gettskact(&index, TaskName::Flight).await {
      killtsk(&index, TaskName::Flight).await;
    } else {
      return false;
    }

    true
  }
}
