use azalea::core::position::BlockPos;
use azalea::prelude::*;
use azalea::registry::builtin::{BlockKind, ItemKind};
use azalea::Vec3;
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use salarixi_macros::Index;

use crate::bot::common::get_block_state;
use crate::bot::extensions::{BotDefaultExt, BotInventoryExt, BotRotationExt};
use crate::bot::systems::states::{getst, setmst, setst, State, StateName};
use crate::bot::systems::tasks::{gettskact, killtsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::tools::*;
use crate::{sleep, take_bot};

struct Tool {
  slot: Option<usize>,
  priority: u8,
}

#[derive(PartialEq, Index)]
enum Mode {
  Normal = 0,
  Quick = 1,
  Ultra = 2,
}

pub struct FarmerOptions {
  mode: Mode,
  delay: Option<u64>,
  state: u8,
}

impl FarmerOptions {
  pub fn from_bytes(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      mode: Mode::from_index(u8::read(buf)?)?,
      delay: Option::read(buf)?,
      state: u8::read(buf)?,
    })
  }
}

pub struct FarmerModule;

impl FarmerModule {
  async fn auto_tool(bot: &Client, index: &u8) {
    let mut tools = vec![];

    if let Some(menu) = bot.get_inventory_menu() {
      for (slot, item) in menu.slots().iter().enumerate() {
        if !item.is_empty() {
          match item.kind() {
            ItemKind::WoodenHoe => {
              tools.push(Tool {
                slot: Some(slot),
                priority: 0,
              });
            }
            ItemKind::GoldenHoe => {
              tools.push(Tool {
                slot: Some(slot),
                priority: 1,
              });
            }
            ItemKind::StoneHoe => {
              tools.push(Tool {
                slot: Some(slot),
                priority: 2,
              });
            }
            ItemKind::CopperHoe => {
              tools.push(Tool {
                slot: Some(slot),
                priority: 3,
              });
            }
            ItemKind::IronHoe => {
              tools.push(Tool {
                slot: Some(slot),
                priority: 4,
              });
            }
            ItemKind::DiamondHoe => {
              tools.push(Tool {
                slot: Some(slot),
                priority: 5,
              });
            }
            ItemKind::NetheriteHoe => {
              tools.push(Tool {
                slot: Some(slot),
                priority: 6,
              });
            }
            _ => {}
          }
        }
      }
    }

    let mut best_tool = Tool {
      slot: None,
      priority: 0,
    };

    for w in tools {
      if w.priority > best_tool.priority {
        best_tool = w;
      }
    }

    if let Some(slot) = best_tool.slot {
      bot.take_item(index, slot, false).await;
    }
  }

  fn this_is_grown_plant(id: u16) -> bool {
    return vec![10464, 5117, 14612, 10472].contains(&id);
  }

  fn this_is_plant(kind: BlockKind) -> bool {
    return vec![
      BlockKind::Potatoes,
      BlockKind::Carrots,
      BlockKind::Wheat,
      BlockKind::Beetroots,
    ]
    .contains(&kind);
  }

  fn this_is_farmland_without_plant(bot: &Client, kind: BlockKind, block_pos: BlockPos) -> bool {
    if kind == BlockKind::Farmland {
      let block_above = BlockPos::new(block_pos.x, block_pos.y + 1, block_pos.z);

      if let Some(state) = get_block_state(bot, block_above) {
        if state.is_air() {
          return true;
        }
      }
    }

    false
  }

  async fn fertilize_plant(bot: &Client, index: &u8, mode: &Mode) {
    let mut ferilizer_slot = None;

    if let Some(menu) = bot.get_inventory_menu() {
      for (slot, item) in menu.slots().iter().enumerate() {
        if !item.is_empty() {
          if item.kind() == ItemKind::BoneMeal {
            ferilizer_slot = Some(slot);
            break;
          }
        }
      }
    }

    if let Some(slot) = ferilizer_slot {
      bot.take_item(index, slot, false).await;

      for _ in 0..=4 {
        sleep!(Self::generate_delay(mode));
        bot.start_use_item();
      }
    }
  }

  async fn take_plant(bot: &Client, index: &u8) -> bool {
    let mut plant_slot = None;

    if let Some(menu) = bot.get_inventory_menu() {
      for (slot, item) in menu.slots().iter().enumerate() {
        if !item.is_empty() && plant_slot.is_none() {
          match item.kind() {
            ItemKind::Potato => plant_slot = Some(slot),
            ItemKind::Carrot => plant_slot = Some(slot),
            ItemKind::BeetrootSeeds => plant_slot = Some(slot),
            ItemKind::WheatSeeds => plant_slot = Some(slot),
            _ => {}
          }
        }
      }
    }

    if let Some(slot) = plant_slot {
      bot.take_item(index, slot, false).await;
      return true;
    }

    false
  }

  fn generate_delay(mode: &Mode) -> u64 {
    match mode {
      Mode::Normal => randnum(100, 150),
      Mode::Quick => 50,
      Mode::Ultra => randnum(20, 30),
    }
  }

  async fn plow_block(bot: &Client, index: &u8, block_pos: BlockPos, mode: &Mode) {
    if let Some(state) = get_block_state(bot, block_pos) {
      Self::auto_tool(bot, index).await;
      sleep!(50);

      let kind = BlockKind::from(state);

      if kind == BlockKind::CoarseDirt || kind == BlockKind::RootedDirt {
        bot.start_use_item();
        sleep!(Self::generate_delay(mode));
        bot.start_use_item();
      } else {
        bot.start_use_item();
      }
    }
  }

  fn block_can_plowed(bot: &Client, kind: BlockKind, block_pos: BlockPos) -> bool {
    if let Some(state) = get_block_state(bot, BlockPos::new(block_pos.x, block_pos.y + 1, block_pos.z)) {
      return vec![
        BlockKind::Dirt,
        BlockKind::DirtPath,
        BlockKind::RootedDirt,
        BlockKind::CoarseDirt,
      ]
      .contains(&kind)
        && state.is_air();
    }

    false
  }

  async fn interact_with_block(bot: &Client, index: &u8, block_pos: BlockPos, options: &FarmerOptions) {
    if !getst(index, State::CanInteracting).await {
      return;
    }

    let Some(state) = get_block_state(bot, block_pos) else {
      return;
    };

    setst(index, State::CanAttacking, false).await;
    setst(index, State::CanDrinking, false).await;
    setst(index, State::CanEating, false).await;
    setmst(index, StateName::Interacting, true).await;

    let kind = BlockKind::from(state);

    if Self::this_is_farmland_without_plant(bot, kind, block_pos) {
      bot.look_at_block(block_pos, true).await;

      if options.mode != Mode::Ultra {
        sleep!(Self::generate_delay(&options.mode));
      }

      if Self::take_plant(bot, index).await {
        sleep!(Self::generate_delay(&options.mode));
        bot.start_use_item();
        sleep!(Self::generate_delay(&options.mode));
      }
    } else {
      if Self::this_is_plant(kind) {
        bot.look_at_block(block_pos, true).await;

        sleep!(Self::generate_delay(&options.mode));

        if Self::this_is_grown_plant(state.id()) {
          Self::auto_tool(bot, index).await;
          sleep!(Self::generate_delay(&options.mode));
          bot.mine(block_pos).await;
          sleep!(50);
        } else {
          Self::fertilize_plant(bot, index, &options.mode).await;

          if let Some(new_state) = get_block_state(bot, block_pos) {
            sleep!(50);

            if Self::this_is_grown_plant(new_state.id()) {
              Self::auto_tool(bot, index).await;
              sleep!(Self::generate_delay(&options.mode));
              bot.mine(block_pos).await;
              sleep!(50);
            }
          }
        }

        if options.mode != Mode::Ultra {
          sleep!(Self::generate_delay(&options.mode));
        }
      } else {
        if Self::block_can_plowed(bot, kind, block_pos) {
          bot.look_at_block(block_pos, true).await;
          Self::plow_block(bot, index, block_pos, &options.mode).await;
        }
      }
    }

    setmst(index, StateName::Interacting, false).await;
    setst(index, State::CanAttacking, true).await;
    setst(index, State::CanDrinking, true).await;
    setst(index, State::CanEating, true).await;
  }

  async fn start(bot: &Client, index: &u8, options: &FarmerOptions) {
    loop {
      setmst(index, StateName::Looking, true).await;

      for y in -1..=1 {
        let Some(foot_pos) = bot.foot_pos() else {
          continue;
        };

        let block_pos = BlockPos::from(Vec3::new(foot_pos.x, foot_pos.y + y as f64, foot_pos.z));

        Self::interact_with_block(bot, index, block_pos, &options).await;
      }

      for x in -1..=1 {
        for y in -1..=1 {
          for z in -1..=1 {
            let Some(foot_pos) = bot.foot_pos() else {
              continue;
            };

            let block_pos = BlockPos::from(Vec3::new(
              foot_pos.x + x as f64,
              foot_pos.y + y as f64,
              foot_pos.z + z as f64,
            ));

            if block_pos != BlockPos::from(Vec3::new(foot_pos.x, foot_pos.y + y as f64, foot_pos.z)) {
              Self::interact_with_block(bot, index, block_pos, &options).await;
            }
          }
        }
      }

      sleep!(options.delay.unwrap_or(100));

      setmst(index, StateName::Looking, false).await;
    }
  }
}

impl SalarixiModule<FarmerOptions> for FarmerModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: std::sync::Arc<FarmerOptions>) -> bool {
    if options.state == 1 && !gettskact(&index, TaskName::Farmer).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, &index, &options).await);
      });

      pushrtsk(&index, TaskName::Farmer, task_handle).await;
    } else if options.state == 0 && gettskact(&index, TaskName::Farmer).await {
      killtsk(&index, TaskName::Farmer).await;
      setmst(&index, StateName::Looking, false).await;
      setmst(&index, StateName::Interacting, false).await;
    } else {
      return false;
    }

    true
  }
}
