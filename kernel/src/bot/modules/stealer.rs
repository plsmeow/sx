use azalea::container::ContainerHandle;
use azalea::core::position::BlockPos;
use azalea::prelude::*;
use azalea::Vec3;
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use salarixi_macros::Index;

use crate::bot::common::get_block_state;
use crate::bot::extensions::BotMovementExt;
use crate::bot::systems::states::{getst, setmst, setst, State, StateName};
use crate::bot::systems::tasks::{gettskact, killtsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::tools::*;
use crate::{sleep, take_bot};

#[derive(Index)]
enum Target {
  Chest = 0,
  Barrel = 1,
  Shulker = 2,
}

pub struct StealerOptions {
  target: Target,
  radius: Option<i32>,
  delay: Option<u64>,
  state: u8,
}

impl StealerOptions {
  pub fn from_bytes(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      target: Target::from_index(u8::read(buf)?)?,
      radius: Option::read(buf)?,
      delay: Option::read(buf)?,
      state: u8::read(buf)?,
    })
  }
}

pub struct StealerModule;

impl StealerModule {
  pub fn new() -> Self {
    Self
  }

  fn check_block_id(block_id: u16, target: &Target) -> bool {
    match target {
      Target::Chest => vec![3793, 3787, 3805, 3799].contains(&block_id),
      Target::Barrel => vec![20547, 20543, 20541, 20549, 20545].contains(&block_id),
      Target::Shulker => vec![
        14666, 14672, 14678, 14720, 14756, 14714, 14762, 14744, 14702, 14696, 14708, 14750, 14684, 14726, 14738, 14732,
      ]
      .contains(&block_id),
    }
  }

  fn find_nearest_targets(bot: &Client, center: Vec3, target: &Target, radius: i32) -> Vec<azalea::BlockPos> {
    let mut positions = Vec::new();

    for x in -radius..=radius {
      for y in -radius..=radius {
        for z in -radius..=radius {
          let block_pos = BlockPos::new(
            (center.x as i32 + x) as i32,
            (center.y as i32 + y) as i32,
            (center.z as i32 + z) as i32,
          );

          if let Some(state) = get_block_state(bot, block_pos) {
            if Self::check_block_id(state.id(), target) {
              positions.push(block_pos);
            }
          }
        }
      }
    }

    positions
  }

  async fn extract_all_items(bot: &Client, index: &u8, container: &ContainerHandle) {
    if let Some(menu) = container.menu() {
      bot.freeze_move(index).await;

      setst(index, State::CanAttacking, false).await;
      setst(index, State::CanInteracting, false).await;
      setst(index, State::CanEating, false).await;
      setst(index, State::CanDrinking, false).await;

      for slot in 0..=26 {
        if let Some(item) = menu.slot(slot) {
          if !item.is_empty() {
            container.shift_click(slot);
          }
        }
      }

      bot.unfreeze_move(index).await;

      setst(index, State::CanAttacking, true).await;
      setst(index, State::CanInteracting, true).await;
      setst(index, State::CanEating, true).await;
      setst(index, State::CanDrinking, true).await;
    }
  }

  async fn start(bot: &Client, index: &u8, options: &StealerOptions) {
    loop {
      let position = bot.position();
      let direction = bot.direction();

      let target_positions = Self::find_nearest_targets(bot, position, &options.target, options.radius.unwrap_or(5));

      for pos in target_positions {
        if getst(index, State::CanLooking).await && getst(index, State::CanInteracting).await {
          setmst(index, StateName::Looking, true).await;
          setmst(index, StateName::Interacting, true).await;

          bot.look_at(pos.center());

          sleep!(randnum(50, 100));

          if let Some(container) = bot.open_container_at(pos).await {
            Self::extract_all_items(bot, index, &container).await;
            container.close();
            sleep!(randnum(200, 350));
          }

          setmst(index, StateName::Looking, false).await;
          setmst(index, StateName::Interacting, false).await;
        }
      }

      if getst(index, State::CanLooking).await {
        bot.set_direction(
          direction.0 + randnum(-2.5, 2.5) as f32,
          direction.1 + randnum(-2.5, 2.5) as f32,
        );
      }

      sleep!(options.delay.unwrap_or(1000));
    }
  }
}

impl SalarixiModule<StealerOptions> for StealerModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: std::sync::Arc<StealerOptions>) -> bool {
    if options.state == 1 && !gettskact(&index, TaskName::Stealer).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, &index, &options).await);
      });

      pushrtsk(&index, TaskName::Stealer, task_handle).await;
    } else if options.state == 0 && gettskact(&index, TaskName::Stealer).await {
      killtsk(&index, TaskName::Stealer).await;
    } else {
      return false;
    }

    true
  }
}
