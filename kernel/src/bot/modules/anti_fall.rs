use std::time::{Duration, Instant};

use azalea::core::position::BlockPos;
use azalea::prelude::*;
use azalea::protocol::common::movements::MoveFlags;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::protocol::packets::game::ServerboundMovePlayerPos;
use azalea::protocol::packets::game::ServerboundUseItem;
use azalea::registry::builtin::ItemKind;
use azalea::Vec3;
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use salarixi_macros::Index;

use crate::bot::common::get_block_state;
use crate::bot::extensions::{BotInventoryExt, BotPhysicsExt};
use crate::bot::systems::tasks::{gettskact, killtsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::tools::*;
use crate::{sleep, take_bot};

#[derive(Index)]
enum Mode {
  Hovering = 0,
  Teleport = 1,
  WaterDrop = 2,
}

pub struct AntiFallOptions {
  mode: Mode,
  distance_to_ground: Option<i32>,
  fall_velocity: Option<f64>,
  delay: Option<u64>,
  state: u8,
}

impl AntiFallOptions {
  pub fn from_bytes(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      mode: Mode::from_index(u8::read(buf)?)?,
      distance_to_ground: Option::read(buf)?,
      fall_velocity: Option::read(buf)?,
      delay: Option::read(buf)?,
      state: u8::read(buf)?,
    })
  }
}

pub struct AntiFallModule;

impl AntiFallModule {
  pub fn new() -> Self {
    Self
  }

  fn exist_blocks_below(bot: &Client, distance_to_ground: i32) -> bool {
    let pos = bot.position();

    for y in 0..=distance_to_ground {
      let block_pos = BlockPos::new(pos.x.floor() as i32, (pos.y.floor() as i32) - y, pos.z.floor() as i32);

      if let Some(state) = get_block_state(bot, block_pos) {
        if !state.is_air() {
          return true;
        }
      }
    }

    false
  }

  fn exist_water_below(bot: &Client, distance_to_ground: i32) -> bool {
    let pos = bot.position();

    for y in 0..=distance_to_ground {
      let block_pos = BlockPos::new(pos.x.floor() as i32, (pos.y.floor() as i32) - y, pos.z.floor() as i32);

      if let Some(state) = get_block_state(bot, block_pos) {
        if !state.is_air() {
          if state.id() == 86 {
            return true;
          }
        }
      }
    }

    false
  }

  async fn take_water_bucket(bot: &Client, index: &u8) -> bool {
    let menu = bot.menu();

    for (slot, item) in menu.slots().iter().enumerate() {
      if !item.is_empty() {
        if item.kind() == ItemKind::WaterBucket {
          bot.take_item(index, slot, true).await;
          return true;
        }
      }
    }

    false
  }

  async fn hovering_anti_fall(bot: &Client, options: &AntiFallOptions) {
    loop {
      let velocity_y = if let Some(physics) = bot.get_physics() {
        physics.velocity.y
      } else {
        0.0
      };

      if velocity_y < options.fall_velocity.unwrap_or(-0.5) {
        if Self::exist_blocks_below(bot, options.distance_to_ground.unwrap_or(4)) {
          let duration = Instant::now() + Duration::from_millis(randnum(300, 450));

          loop {
            if Instant::now() >= duration {
              break;
            }

            bot.set_velocity("y", 0.0);
            bot.set_on_ground(true);

            sleep!(50);

            bot.set_on_ground(false);
          }
        }
      }

      sleep!(options.delay.unwrap_or(50));
    }
  }

  async fn teleport_anti_fall(bot: &Client, options: &AntiFallOptions) {
    loop {
      if let Some(physics) = bot.get_physics() {
        let velocity_y = physics.velocity.y;

        if velocity_y < options.fall_velocity.unwrap_or(-0.5) {
          if Self::exist_blocks_below(bot, options.distance_to_ground.unwrap_or(4)) {
            let pos = bot.position();
            let fake_pos = Vec3::new(pos.x, pos.y + randnum(0.015, 0.022), pos.z);

            bot.write_packet(ServerboundMovePlayerPos {
              pos: fake_pos,
              flags: MoveFlags {
                on_ground: physics.on_ground(),
                horizontal_collision: physics.horizontal_collision,
              },
            });
          }
        }
      }

      sleep!(options.delay.unwrap_or(50));
    }
  }

  async fn water_drop_anti_fall(bot: &Client, index: &u8, options: &AntiFallOptions) {
    let distance_to_ground = options.distance_to_ground.unwrap_or(4);

    loop {
      let velocity_y = if let Some(physics) = bot.get_physics() {
        physics.velocity.y
      } else {
        0.0
      };

      if velocity_y < options.fall_velocity.unwrap_or(-0.5) {
        if Self::exist_blocks_below(bot, distance_to_ground) {
          let y_rot = bot.direction().0;

          bot.set_direction(y_rot, randnum(85.0, 89.0) as f32);

          if Self::take_water_bucket(bot, index).await {
            if !Self::exist_water_below(bot, distance_to_ground) {
              sleep!(50);

              let direction = bot.direction();

              bot.write_packet(ServerboundUseItem {
                hand: InteractionHand::MainHand,
                seq: 0,
                y_rot: direction.0,
                x_rot: direction.1,
              });
            }
          }

          sleep!(50);

          if Self::exist_water_below(bot, distance_to_ground) {
            let direction = bot.direction();

            bot.write_packet(ServerboundUseItem {
              hand: InteractionHand::MainHand,
              seq: 0,
              y_rot: direction.0,
              x_rot: direction.1,
            });
          }
        }
      }

      sleep!(options.delay.unwrap_or(50));
    }
  }

  async fn start(bot: &Client, index: &u8, options: &AntiFallOptions) {
    match options.mode {
      Mode::Hovering => Self::hovering_anti_fall(bot, options).await,
      Mode::Teleport => Self::teleport_anti_fall(bot, options).await,
      Mode::WaterDrop => Self::water_drop_anti_fall(bot, index, options).await,
    }
  }
}

impl SalarixiModule<AntiFallOptions> for AntiFallModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: std::sync::Arc<AntiFallOptions>) -> bool {
    if options.state == 1 && !gettskact(&index, TaskName::AntiFall).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, &index, &options).await);
      });

      pushrtsk(&index, TaskName::AntiFall, task_handle).await;
    } else if options.state == 0 && gettskact(&index, TaskName::AntiFall).await {
      killtsk(&index, TaskName::AntiFall).await;
    } else {
      return false;
    }

    true
  }
}
