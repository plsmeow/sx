use azalea::core::position::BlockPos;
use azalea::prelude::*;
use azalea::{Vec3, WalkDirection};
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use salarixi_macros::Index;

use crate::bot::common::{
  convert_hotbar_slot_to_inventory_slot, convert_inventory_slot_to_hotbar_slot, get_block_state, this_is_solid_block,
};
use crate::bot::extensions::{BotDefaultExt, BotInventoryExt, BotMovementExt, BotPhysicsExt};
use crate::bot::systems::states::{getst, setmst, State, StateName};
use crate::bot::systems::tasks::{gettskact, killtsk, pushetsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::tools::*;
use crate::{sleep, take_bot};

#[derive(Index)]
enum Mode {
  NoobBridge = 0,
  NinjaBridge = 1,
  GodBridge = 2,
  JumpBridge = 3,
}

pub struct ScaffoldOptions {
  mode: Mode,
  delay: Option<u64>,
  min_gaze_degree_x: Option<f32>,
  max_gaze_degree_x: Option<f32>,
  state: u8,
}

impl ScaffoldOptions {
  pub fn from_bytes(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      mode: Mode::from_index(u8::read(buf)?)?,
      delay: Option::read(buf)?,
      min_gaze_degree_x: Option::read(buf)?,
      max_gaze_degree_x: Option::read(buf)?,
      state: u8::read(buf)?,
    })
  }
}

pub struct ScaffoldModule;

impl ScaffoldModule {
  async fn take_block(bot: &Client, index: &u8) -> bool {
    if let Some(menu) = bot.get_inventory_menu() {
      if let Some(item) = menu.slot(convert_hotbar_slot_to_inventory_slot(bot.get_selected_slot())) {
        if this_is_solid_block(item.kind()) {
          return true;
        }
      }

      let mut block_slot = None;

      for (slot, item) in menu.slots().iter().enumerate() {
        if this_is_solid_block(item.kind()) {
          for s in 36..=44 {
            if let Some(i) = menu.slot(s) {
              if this_is_solid_block(i.kind()) {
                if let Some(hotbar_slot) = convert_inventory_slot_to_hotbar_slot(s) {
                  if bot.get_selected_slot() == hotbar_slot {
                    return true;
                  }
                }
              }
            }
          }

          if let Some(hotbar_slot) = convert_inventory_slot_to_hotbar_slot(slot) {
            if bot.get_selected_slot() == hotbar_slot {
              return true;
            }
          }

          block_slot = Some(slot);
          break;
        }
      }

      if let Some(slot) = block_slot {
        bot.take_item(index, slot, false).await;
        return true;
      }
    }

    false
  }

  fn simulate_inaccuracy(bot: &Client, y_rot: f32, x_rot: f32) {
    let _ = bot.set_direction(y_rot + randnum(-0.08, 0.08) as f32, x_rot + randnum(-0.08, 0.08) as f32);
  }

  fn direct_gaze(bot: &Client, min_x_rot: Option<f32>, max_x_rot: Option<f32>) {
    let direction = bot.direction().unwrap_or_default();

    let min_x = if let Some(rot) = min_x_rot { rot } else { 80.0 } as f64;
    let max_x = if let Some(rot) = max_x_rot { rot } else { 83.0 } as f64;

    let _ = bot.set_direction(direction.y_rot(), randnum(min_x, max_x) as f32);
  }

  async fn go_back(index: u8) {
    let extra_task = tokio::spawn(async move {
      loop {
        take_bot!(&index, async |bot| {
          bot.start_walking(&index, WalkDirection::Backward).await;
        });

        sleep!(100);
      }
    });

    pushetsk(&index, TaskName::Scaffold, extra_task).await;
  }

  async fn noob_bridge_scaffold(bot: &Client, index: &u8, options: &ScaffoldOptions) {
    let delay = options.delay.unwrap_or(50);

    loop {
      if !getst(index, State::CanLooking).await
        || !getst(index, State::CanInteracting).await
        || !Self::take_block(bot, index).await
      {
        sleep!(100);
        continue;
      }

      setmst(&index, StateName::Looking, true).await;
      setmst(&index, StateName::Interacting, true).await;

      if !bot.crouching() {
        let _ = bot.set_crouching(true);
      }

      Self::direct_gaze(bot, options.min_gaze_degree_x, options.max_gaze_degree_x);

      let Ok(pos) = bot.position() else {
        continue;
      };

      let block_under = BlockPos::new(pos.x.floor() as i32, (pos.y - 0.5).floor() as i32, pos.z.floor() as i32);

      let is_air = if let Some(state) = get_block_state(bot, block_under) {
        state.is_air()
      } else {
        false
      };

      if is_air {
        bot.block_interact(block_under);
        bot.swing_arm();
        sleep!(randnum(50, 100));
        let dir = bot.direction().unwrap_or_default();

        Self::simulate_inaccuracy(bot, dir.y_rot(), dir.x_rot());
        sleep!(randnum(100, 150));
      }

      setmst(&index, StateName::Looking, false).await;
      setmst(&index, StateName::Interacting, false).await;

      sleep!(delay);
    }
  }

  async fn ninja_bridge_scaffold(bot: &Client, index: &u8, options: &ScaffoldOptions) {
    let delay = options.delay.unwrap_or(50);

    loop {
      if !getst(index, State::CanLooking).await
        || !getst(index, State::CanInteracting).await
        || !Self::take_block(bot, index).await
      {
        sleep!(100);
        continue;
      }

      setmst(&index, StateName::Looking, true).await;
      setmst(&index, StateName::Interacting, true).await;

      Self::direct_gaze(bot, options.min_gaze_degree_x, options.max_gaze_degree_x);

      let Ok(pos) = bot.position() else {
        continue;
      };

      let block_under = BlockPos::new(pos.x.floor() as i32, (pos.y - 0.5).floor() as i32, pos.z.floor() as i32);

      let is_air = if let Some(state) = get_block_state(bot, block_under) {
        state.is_air()
      } else {
        false
      };

      if is_air {
        let _ = bot.set_crouching(true);

        bot.block_interact(block_under);
        bot.swing_arm();
        sleep!(randnum(50, 100));
        let dir = bot.direction().unwrap_or_default();

        Self::simulate_inaccuracy(bot, dir.y_rot(), dir.x_rot());
        sleep!(50);
        let _ = bot.set_crouching(false);
      }

      setmst(&index, StateName::Looking, false).await;
      setmst(&index, StateName::Interacting, false).await;

      sleep!(delay);
    }
  }

  async fn god_bridge_scaffold(bot: &Client, index: &u8, options: &ScaffoldOptions) {
    let delay = options.delay.unwrap_or(50);

    loop {
      if !getst(index, State::CanLooking).await
        || !getst(index, State::CanInteracting).await
        || !Self::take_block(bot, index).await
      {
        sleep!(100);
        continue;
      }

      setmst(&index, StateName::Looking, true).await;
      setmst(&index, StateName::Interacting, true).await;

      Self::direct_gaze(bot, options.min_gaze_degree_x, options.max_gaze_degree_x);

      let Ok(pos) = bot.position() else {
        continue;
      };

      let block_under = BlockPos::new(pos.x.floor() as i32, (pos.y - 0.5).floor() as i32, pos.z.floor() as i32);

      let is_air = if let Some(state) = get_block_state(bot, block_under) {
        state.is_air()
      } else {
        false
      };

      if is_air {
        bot.block_interact(block_under);
        bot.swing_arm();
        let dir = bot.direction().unwrap_or_default();

        Self::simulate_inaccuracy(bot, dir.y_rot(), dir.x_rot());
      }

      setmst(&index, StateName::Looking, false).await;
      setmst(&index, StateName::Interacting, false).await;

      sleep!(delay);
    }
  }

  async fn jump_bridge_scaffold(bot: &Client, index: &u8, options: &ScaffoldOptions) {
    let delay = options.delay.unwrap_or(50);

    loop {
      if !getst(index, State::CanLooking).await
        || !getst(index, State::CanInteracting).await
        || !Self::take_block(bot, index).await
      {
        sleep!(100);
        continue;
      }

      setmst(&index, StateName::Looking, true).await;
      setmst(&index, StateName::Interacting, true).await;

      Self::direct_gaze(bot, options.min_gaze_degree_x, options.max_gaze_degree_x);

      let velocity = if let Some(physics) = bot.get_physics() {
        physics.velocity
      } else {
        Vec3::ZERO
      };

      let Ok(pos) = bot.position() else {
        continue;
      };

      let block_under = BlockPos::new(
        pos.x.floor() as i32,
        (if velocity.y != 0.0 { pos.y - 1.0 } else { pos.y - 0.5 }).floor() as i32,
        pos.z.floor() as i32,
      );

      let is_air = if let Some(state) = get_block_state(bot, block_under) {
        state.is_air()
      } else {
        false
      };

      if is_air {
        bot.jump();
        sleep!(50);
        bot.block_interact(block_under);
        bot.swing_arm();
        let dir = bot.direction().unwrap_or_default();

        Self::simulate_inaccuracy(bot, dir.y_rot(), dir.x_rot());
      }

      setmst(&index, StateName::Looking, false).await;
      setmst(&index, StateName::Interacting, false).await;

      sleep!(delay);
    }
  }

  async fn start(bot: &Client, index: u8, options: &ScaffoldOptions) {
    Self::go_back(index).await;

    match options.mode {
      Mode::NoobBridge => Self::noob_bridge_scaffold(bot, &index, &options).await,
      Mode::NinjaBridge => Self::ninja_bridge_scaffold(bot, &index, &options).await,
      Mode::GodBridge => Self::god_bridge_scaffold(bot, &index, &options).await,
      Mode::JumpBridge => Self::jump_bridge_scaffold(bot, &index, &options).await,
    }
  }
}

impl SalarixiModule<ScaffoldOptions> for ScaffoldModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: std::sync::Arc<ScaffoldOptions>) -> bool {
    if options.state == 1 && !gettskact(&index, TaskName::Scaffold).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, index, &options).await);
      });

      pushrtsk(&index, TaskName::Scaffold, task_handle).await;
    } else if options.state == 0 && gettskact(&index, TaskName::Scaffold).await {
      killtsk(&index, TaskName::Scaffold).await;

      take_bot!(&index, async |bot| {
        bot.stop_move(&index).await;
        let _ = bot.set_crouching(false);
      });

      setmst(&index, StateName::Looking, false).await;
      setmst(&index, StateName::Interacting, false).await;
    } else {
      return false;
    }

    true
  }
}
