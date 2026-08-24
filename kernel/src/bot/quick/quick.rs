use std::f32::consts::PI;

use azalea::core::position::BlockPos;
use azalea::prelude::PathfinderClientExt;
use azalea::WalkDirection;
use once_cell::sync::Lazy;
use salarixi_extensions::index::IndexExt;
use salarixi_macros::Index;

use crate::bot::common::{get_average_coordinates_of_bots, this_is_solid_block};
use crate::bot::extensions::{
  go_to, BotDefaultExt, BotInventoryExt, BotMovementExt, BotPhysicsExt, BotRotationExt, ClickMode,
};
use crate::bot::systems::profile::{BotStatus, PROFILE_SYSTEM};
use crate::bot::systems::registry::REGISTRY_SYSTEM;
use crate::bot::systems::states::{getst, setmst, State, StateName, STATE_SYSTEM};
use crate::bot::systems::tasks::TASK_SYSTEM;
use crate::bot::PLUGINS;
use crate::launch::process::current_options;
use crate::server::transfer::{emit_log, emit_msg};
use crate::webhook::send_webhook;
use crate::{sleep, take_bot};
use crate::{take_profile, tools::*};

pub static QUICK_TASKS: Lazy<QuickTaskManager> = Lazy::new(|| QuickTaskManager::new());

#[derive(Clone, Index)]
enum QuickTask {
  ClearInventory = 0x00,
  MoveForward = 0x01,
  MoveBackward = 0x02,
  MoveLeft = 0x03,
  MoveRight = 0x04,
  Jump = 0x05,
  Shift = 0x06,
  Fly = 0x07,
  Quit = 0x08,
  Rise = 0x09,
  Capsule = 0x0A,
  Unite = 0x0B,
  Turn = 0x0C,
  Zero = 0x0D,
  FormCircle = 0x0E,
  FormLine = 0x0F,
  StopPathfinder = 0x10,
}

impl QuickTask {
  fn to_str(&self) -> &str {
    match self {
      Self::ClearInventory => "clear-inventory",
      Self::MoveForward => "move-forward",
      Self::MoveBackward => "move-backward",
      Self::MoveLeft => "move-left",
      Self::MoveRight => "move-right",
      Self::Jump => "jump",
      Self::Shift => "shift",
      Self::Fly => "fly",
      Self::Quit => "quit",
      Self::Rise => "rise",
      Self::Capsule => "capsule",
      Self::Unite => "unite",
      Self::Turn => "turn",
      Self::Zero => "zero",
      Self::FormCircle => "form-circle",
      Self::FormLine => "form-line",
      Self::StopPathfinder => "stop-pathfinder",
    }
  }
}

pub struct QuickTaskManager;

impl QuickTaskManager {
  pub fn new() -> Self {
    Self
  }

  /// Метод выполненя быстрой задачи
  pub async fn execute(&self, id: u8) {
    let Some(task) = QuickTask::from_index(id) else {
      return;
    };

    let connected_bots = PROFILE_SYSTEM.get_connected_count().await;
    let str_task = task.to_str();

    if let Some(opts) = current_options().await {
      if opts.basic.use_webhook && opts.webhook.send_actions {
        send_webhook(
          opts.webhook.url,
          format!("{} ботов получили быструю задачу \"{}\"", connected_bots, str_task),
        );
      }
    }

    emit_log(
      format!("{} ботов получили быструю задачу \"{}\"", connected_bots, str_task),
      "info",
    );

    emit_msg("Быстрая задача", format!("Быстрая задача \"{}\"", str_task));

    for (number, index) in PROFILE_SYSTEM.get_all_connected().await.into_keys().enumerate() {
      let task_clone = task.clone();

      // TODO: Разбить все задачи по отдельным функциям
      tokio::spawn(async move {
        take_bot!(&index, async |bot| match task_clone {
          QuickTask::ClearInventory => {
            if let Some(menu) = bot.get_inventory_menu() {
              for (slot, _) in menu.slots().iter().enumerate() {
                bot.inventory_click(&index, slot, ClickMode::DropAll, false).await;
              }
            }
          }
          QuickTask::MoveForward => {
            bot.start_walking(&index, WalkDirection::Forward).await;
            sleep!(200);
            bot.stop_move(&index).await;
          }
          QuickTask::MoveBackward => {
            bot.start_walking(&index, WalkDirection::Backward).await;
            sleep!(200);
            bot.stop_move(&index).await;
          }
          QuickTask::MoveLeft => {
            bot.start_walking(&index, WalkDirection::Right).await;
            sleep!(200);
            bot.stop_move(&index).await;
          }
          QuickTask::MoveRight => {
            bot.start_walking(&index, WalkDirection::Left).await;
            sleep!(200);
            bot.stop_move(&index).await;
          }
          QuickTask::Jump => {
            bot.jump();
          }
          QuickTask::Shift => {
            bot.set_crouching(true);
            sleep!(200);
            bot.set_crouching(false);
          }
          QuickTask::Fly => {
            for i in 0..randnum(3, 5) {
              bot.set_velocity("y", randnum(0.022 * i as f64, 0.031 * i as f64));
              sleep!(50);
            }
          }
          QuickTask::Quit => {
            PLUGINS.kill_all_tasks_for(&index).await;
            TASK_SYSTEM.kill_all_tasks_for(&index).await;

            sleep!(2000);

            bot.disconnect();

            STATE_SYSTEM.remove(&index).await;
            TASK_SYSTEM.remove(&index).await;

            take_profile!(&index, async |profile| {
              profile.status = BotStatus::Disconnected;
            });
          }
          QuickTask::Rise => {
            if getst(&index, State::CanLooking).await || getst(&index, State::CanInteracting).await {
              let mut block_slot = None;

              if let Some(menu) = bot.get_inventory_menu() {
                for (slot, item) in menu.slots().iter().enumerate() {
                  if !item.is_empty() {
                    if this_is_solid_block(item.kind()) {
                      block_slot = Some(slot);
                      break;
                    }
                  }
                }
              }

              if let Some(slot) = block_slot {
                setmst(&index, StateName::Looking, true).await;
                setmst(&index, StateName::Interacting, true).await;

                bot.take_item(&index, slot, false).await;

                let initial_dir = bot.direction().unwrap_or_default();

                let _ = bot.set_direction(initial_dir.y_rot() + randnum(-5.0, 5.0) as f32, randnum(40.0, 58.0) as f32);

                sleep!(randnum(50, 100));

                bot.jump();

                let _ = bot.set_direction(
                  bot.direction().map(|d| d.y_rot()).unwrap_or_default() + randnum(-5.0, 5.0) as f32,
                  randnum(86.0, 90.0) as f32,
                );

                sleep!(randnum(250, 300));

                if let Some(foot_pos) = bot.foot_pos() {
                  bot.block_interact(BlockPos::from(foot_pos));

                  sleep!(randnum(100, 150));

                  let _ = bot.set_direction(initial_dir.y_rot(), initial_dir.x_rot());

                  setmst(&index, StateName::Looking, false).await;
                  setmst(&index, StateName::Interacting, false).await;
                }
              }
            }
          }
          QuickTask::Capsule => {
            if let Some(foot_pos) = bot.foot_pos() {
              let block_positions = vec![
                BlockPos {
                  x: (foot_pos.x - 1.0).floor() as i32,
                  y: foot_pos.y.floor() as i32,
                  z: foot_pos.z.floor() as i32,
                },
                BlockPos {
                  x: (foot_pos.x + 1.0).floor() as i32,
                  y: foot_pos.y.floor() as i32,
                  z: foot_pos.z.floor() as i32,
                },
                BlockPos {
                  x: foot_pos.x.floor() as i32,
                  y: foot_pos.y.floor() as i32,
                  z: (foot_pos.z - 1.0).floor() as i32,
                },
                BlockPos {
                  x: foot_pos.x.floor() as i32,
                  y: foot_pos.y.floor() as i32,
                  z: (foot_pos.z + 1.0).floor() as i32,
                },
                BlockPos {
                  x: (foot_pos.x - 1.0).floor() as i32,
                  y: (foot_pos.y + 1.0).floor() as i32,
                  z: foot_pos.z.floor() as i32,
                },
                BlockPos {
                  x: (foot_pos.x + 1.0).floor() as i32,
                  y: (foot_pos.y + 1.0).floor() as i32,
                  z: foot_pos.z.floor() as i32,
                },
                BlockPos {
                  x: foot_pos.x.floor() as i32,
                  y: (foot_pos.y + 1.0).floor() as i32,
                  z: (foot_pos.z - 1.0).floor() as i32,
                },
                BlockPos {
                  x: foot_pos.x.floor() as i32,
                  y: (foot_pos.y + 1.0).floor() as i32,
                  z: (foot_pos.z + 1.0).floor() as i32,
                },
                BlockPos {
                  x: (foot_pos.x - 1.0).floor() as i32,
                  y: (foot_pos.y + 2.0).floor() as i32,
                  z: foot_pos.z.floor() as i32,
                },
                BlockPos {
                  x: foot_pos.x.floor() as i32,
                  y: (foot_pos.y + 2.0).floor() as i32,
                  z: foot_pos.z.floor() as i32,
                },
              ];

              let mut count = 0;

              for pos in block_positions {
                let mut block_slot = None;

                if let Some(menu) = bot.get_inventory_menu() {
                  for (slot, item) in menu.slots().iter().enumerate() {
                    if !item.is_empty() {
                      if this_is_solid_block(item.kind()) {
                        block_slot = Some(slot);
                        break;
                      }
                    }
                  }
                }

                if let Some(slot) = block_slot {
                  if getst(&index, State::CanLooking).await && getst(&index, State::CanInteracting).await {
                    setmst(&index, StateName::Looking, true).await;
                    setmst(&index, StateName::Interacting, true).await;

                    bot.take_item(&index, slot, false).await;

                    count = count + 1;

                    if count == 10 {
                      sleep!(randnum(150, 200));
                    }

                    if count == 9 {
                      bot.jump();
                      sleep!(50);
                      bot.set_crouching(true);
                      sleep!(randnum(100, 150));
                    }

                    bot.look_at_block(pos, false).await;
                    sleep!(randnum(50, 100));
                    bot.block_interact(pos);

                    if count == 10 {
                      bot.set_crouching(false);
                    }

                    setmst(&index, StateName::Looking, false).await;
                    setmst(&index, StateName::Interacting, false).await;

                    sleep!(randnum(100, 150));
                  }
                }
              }
            }
          }
          QuickTask::Unite => {
            let mut positions = vec![];

            for username in PROFILE_SYSTEM.get_all_connected().await.keys() {
              let Some(b) = REGISTRY_SYSTEM.get_bot(username) else {
                continue;
              };

              let Some(foot_pos) = b.foot_pos() else {
                continue;
              };

              positions.push(foot_pos);
            }

            let average_cords = get_average_coordinates_of_bots(&positions);

            go_to(index, average_cords.0 as i32, average_cords.2 as i32);
          }
          QuickTask::Turn => {
            let direction = bot.direction().unwrap_or_default();
            let _ = bot.set_direction(direction.y_rot() - 90.0, direction.x_rot());
          }
          QuickTask::Zero => {
            let _ = bot.set_direction(0.0, 0.0);
          }
          QuickTask::FormCircle => {
            let mut positions = vec![];

            for username in PROFILE_SYSTEM.get_all_connected().await.keys() {
              let Some(b) = REGISTRY_SYSTEM.get_bot(username) else {
                continue;
              };

              let Some(foot_pos) = b.foot_pos() else {
                continue;
              };

              positions.push(foot_pos);
            }

            let average_cords = get_average_coordinates_of_bots(&positions);

            let angle = 2.0 * PI * (number as f32) / (PROFILE_SYSTEM.get_connected_count().await as f32);
            let x = average_cords.0 + positions.len() as f64 * 0.5 * angle.cos() as f64;
            let z = average_cords.2 + positions.len() as f64 * 0.5 * angle.sin() as f64;

            go_to(index, x as i32, z as i32);
          }
          QuickTask::FormLine => {
            let mut positions = vec![];

            for username in PROFILE_SYSTEM.get_all_connected().await.keys() {
              let Some(b) = REGISTRY_SYSTEM.get_bot(username) else {
                continue;
              };

              let Some(foot_pos) = b.foot_pos() else {
                continue;
              };

              positions.push(foot_pos);
            }

            let average_cords = get_average_coordinates_of_bots(&positions);

            let x = average_cords.0 + 1.0 * (number as f64 * 1.0);
            let z = average_cords.2 * (number as f64 * 1.0);

            go_to(index, x as i32, z as i32);
          }
          QuickTask::StopPathfinder => {
            bot.stop_pathfinding();
          }
        });
      });
    }
  }
}
