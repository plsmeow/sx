use azalea::auto_tool::best_tool_in_hotbar_for_block;
use azalea::core::position::BlockPos;
use azalea::prelude::*;
use azalea::Vec3;
use azalea::WalkDirection;
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use salarixi_macros::Index;

use crate::bot::common::get_block_state;
use crate::bot::extensions::{BotDefaultExt, BotInventoryExt, BotMovementExt};
use crate::bot::systems::states::{setmst, StateName};
use crate::bot::systems::tasks::{gettskact, killtsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::sleep;
use crate::take_bot;
use crate::tools::*;

#[derive(Index)]
enum Mode {
  Default = 0,
  Extended = 1,
}

#[derive(Index)]
enum Tunnel {
  Tun2x2x2 = 0,
  Tun3x2x2 = 1,
  Tun2x3x3 = 2,
  Tun3x3x3 = 3,
}

#[derive(PartialEq, Index)]
enum Look {
  Smooth = 0,
  Cutting = 1,
}

pub struct MinerOptions {
  mode: Mode,
  tunnel: Tunnel,
  look: Look,
  direction_x: Option<f32>,
  slot: Option<u8>,
  delay: Option<u64>,
  state: u8,
}

impl MinerOptions {
  pub fn from_bytes(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      mode: Mode::from_index(u8::read(buf)?)?,
      tunnel: Tunnel::from_index(u8::read(buf)?)?,
      look: Look::from_index(u8::read(buf)?)?,
      direction_x: Option::read(buf)?,
      slot: Option::read(buf)?,
      delay: Option::read(buf)?,
      state: u8::read(buf)?,
    })
  }
}

pub struct MinerModule;

impl MinerModule {
  pub fn new() -> Self {
    Self
  }

  fn is_breakable_block(block_id: u16) -> bool {
    return vec![86, 88, 87, 89, 94, 0, 110, 104, 106, 102, 108].contains(&block_id);
  }

  fn can_reach_block(eye_pos: Vec3, block_pos: BlockPos) -> bool {
    if eye_pos.distance_to(Vec3::new(block_pos.x as f64, block_pos.y as f64, block_pos.z as f64)) < 4.5 {
      return true;
    }

    false
  }

  async fn micro_offset(bot: &Client) {
    if randchance(0.8) {
      let walk_directions = vec![
        WalkDirection::Left,
        WalkDirection::Right,
        WalkDirection::ForwardLeft,
        WalkDirection::ForwardRight,
        WalkDirection::BackwardLeft,
        WalkDirection::BackwardRight,
      ];

      let walk_direction = randelem(walk_directions.as_ref());

      bot.walk(*walk_direction);
      sleep!(randnum(150, 250));
      bot.walk(WalkDirection::None);
    }
  }

  async fn look_at_block(bot: &Client, block_pos: BlockPos, look: &Look) {
    let mut center = block_pos.center();

    match look {
      Look::Smooth => {
        let pre = Vec3::new(
          (block_pos.x as f64) + randnum(-0.05, 0.05),
          (block_pos.y as f64) + randnum(-0.05, 0.05),
          (block_pos.z as f64) + randnum(-0.05, 0.05),
        );

        bot.look_at(pre);

        sleep!(randnum(50, 150));
      }
      Look::Cutting => {}
    }

    if randchance(0.7) {
      center.x += randnum(-0.03, 0.03);
    }

    if randchance(0.7) {
      center.y += randnum(-0.03, 0.03);
    }

    if randchance(0.7) {
      center.z += randnum(-0.03, 0.03);
    }

    bot.look_at(center);
  }

  async fn move_forward(bot: &Client, territory: Vec<BlockPos>) {
    for block_pos in territory {
      if let Some(state) = get_block_state(bot, block_pos) {
        if !state.is_air() && Self::is_breakable_block(state.id()) {
          return;
        }
      }
    }

    bot.walk(WalkDirection::Forward);
    sleep!(randnum(50, 150));
    bot.walk(WalkDirection::None);
  }

  fn get_territory(pos: Vec3, tunnel: &Tunnel) -> Vec<BlockPos> {
    let x = pos.x.floor() as i32;
    let y = pos.y.floor() as i32;
    let z = pos.z.floor() as i32;

    let territory;

    match tunnel {
      Tunnel::Tun2x2x2 => {
        territory = vec![
          BlockPos::new(x + 1, y, z),
          BlockPos::new(x - 1, y, z),
          BlockPos::new(x, y, z + 1),
          BlockPos::new(x, y, z - 1),
        ];
      }
      Tunnel::Tun3x2x2 => {
        territory = vec![
          BlockPos::new(x + 1, y, z),
          BlockPos::new(x - 1, y, z),
          BlockPos::new(x, y, z + 1),
          BlockPos::new(x, y, z - 1),
        ];
      }
      Tunnel::Tun2x3x3 => {
        territory = vec![
          BlockPos::new(x + 1, y, z),
          BlockPos::new(x - 1, y, z),
          BlockPos::new(x, y, z + 1),
          BlockPos::new(x, y, z - 1),
          BlockPos::new(x + 2, y, z),
          BlockPos::new(x - 2, y, z),
          BlockPos::new(x, y, z + 2),
          BlockPos::new(x, y, z - 2),
        ];
      }
      Tunnel::Tun3x3x3 => {
        territory = vec![
          BlockPos::new(x + 1, y, z),
          BlockPos::new(x - 1, y, z),
          BlockPos::new(x, y, z + 1),
          BlockPos::new(x, y, z - 1),
          BlockPos::new(x + 2, y, z),
          BlockPos::new(x - 2, y, z),
          BlockPos::new(x, y, z + 2),
          BlockPos::new(x, y, z - 2),
          BlockPos::new(x + 3, y, z),
          BlockPos::new(x - 3, y, z),
          BlockPos::new(x, y, z + 3),
          BlockPos::new(x, y, z - 3),
        ];
      }
    };

    territory
  }

  async fn default_mine(bot: &Client, options: &MinerOptions) {
    bot.left_click_mine(true);
    bot.walk(WalkDirection::Forward);
    bot.set_direction(options.direction_x.unwrap_or(0.0), 40.0 + randnum(-3.5, 3.5) as f32);
  }

  async fn extended_mine(bot: &Client, options: &MinerOptions) {
    'circles: loop {
      let Some(foot_pos) = bot.foot_pos() else {
        sleep!(200);
        continue;
      };

      let territory = Self::get_territory(foot_pos, &options.tunnel);

      for pos in territory.clone() {
        let heights = match options.tunnel {
          Tunnel::Tun3x2x2 | Tunnel::Tun3x3x3 => vec![2, 1, 0],
          _ => vec![1, 0],
        };

        for height in heights {
          let block_pos = BlockPos::new(pos.x, pos.y + height, pos.z);

          let Some(eye_pos) = bot.eye_pos() else {
            continue 'circles;
          };

          if let Some(state) = get_block_state(bot, block_pos) {
            if !state.is_air() && Self::is_breakable_block(state.id()) && Self::can_reach_block(eye_pos, block_pos) {
              let should_shift = randchance(0.2);

              if should_shift {
                bot.set_crouching(true);
              }

              Self::look_at_block(bot, block_pos, &options.look).await;

              sleep!(randnum(100, 200));

              if let Some(slot) = options.slot {
                bot.set_selected_hotbar_slot(slot);
              } else {
                if let Some(menu) = bot.get_inventory_menu() {
                  let best_tool = best_tool_in_hotbar_for_block(state, &menu).index;
                  bot.set_selected_hotbar_slot(best_tool as u8);
                }
              }

              if should_shift {
                bot.set_crouching(false);
              }

              Self::micro_offset(bot).await;

              bot.start_mining(block_pos);

              loop {
                if let Some(s) = get_block_state(bot, block_pos) {
                  if state.is_air() || !Self::is_breakable_block(s.id()) {
                    break;
                  }
                } else {
                  break;
                }

                sleep!(50);
              }

              Self::micro_offset(bot).await;

              sleep!(randnum(50, 100));

              if let Some(s) = get_block_state(bot, block_pos) {
                if !s.is_air() || Self::is_breakable_block(s.id()) {
                  bot.walk(WalkDirection::Backward);
                  sleep!(randnum(50, 100));
                  bot.walk(WalkDirection::None);
                }
              }
            }
          }
        }
      }

      let direction = bot.direction().unwrap_or_default();

      if randchance(0.5) {
        let _ = bot.set_direction(
          options.direction_x.unwrap_or(0.0) + randnum(-2.5, 2.5) as f32,
          direction.x_rot() + randnum(-2.5, 2.5) as f32,
        );
      } else {
        if randchance(0.3) {
          let _ = bot.set_direction(
            options.direction_x.unwrap_or(0.0) + randnum(-1.3, 1.3) as f32,
            direction.x_rot() + randnum(-1.3, 1.3) as f32,
          );
        } else {
          let _ = bot.set_direction(options.direction_x.unwrap_or(0.0), direction.x_rot());
        }
      }

      sleep!(randnum(50, 100));

      Self::move_forward(bot, territory).await;

      sleep!(options.delay.unwrap_or(200));
    }
  }

  async fn start(bot: &Client, options: &MinerOptions) {
    match options.mode {
      Mode::Default => Self::default_mine(bot, &options).await,
      Mode::Extended => Self::extended_mine(bot, &options).await,
    }
  }
}

impl SalarixiModule<MinerOptions> for MinerModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: std::sync::Arc<MinerOptions>) -> bool {
    if options.state == 1 && !gettskact(&index, TaskName::Miner).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, &options).await);
      });

      pushrtsk(&index, TaskName::Miner, task_handle).await;
    } else if options.state == 0 && gettskact(&index, TaskName::Miner).await {
      killtsk(&index, TaskName::Miner).await;

      take_bot!(&index, async |bot| {
        bot.left_click_mine(false);
        bot.set_crouching(false);
        bot.stop_move(&index).await;
      });

      setmst(&index, StateName::Interacting, false).await;
      setmst(&index, StateName::Looking, false).await;
    } else {
      return false;
    }

    true
  }
}
