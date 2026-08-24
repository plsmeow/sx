use azalea::ecs::entity::Entity;
use azalea::prelude::*;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::registry::builtin::ItemKind;
use azalea::Vec3;
use salarixi_extensions::buffer::BufferExt;

use crate::bot::extensions::{BotDefaultExt, BotInteractExt, BotInventoryExt, BotMovementExt, EntityFilter};
use crate::bot::systems::states::{getst, setmst, setst, State, StateName};
use crate::bot::systems::tasks::{gettskact, killtsk, pushetsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::tools::*;
use crate::{sleep, take_bot};

pub struct BowAimOptions {
  entity_filter: EntityFilter,
  shoot_delay: Option<u64>,
  max_distance: Option<f64>,
  use_prediction: bool,
  state: u8,
}

impl BowAimOptions {
  pub fn from_bytes(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      entity_filter: EntityFilter::read(buf)?,
      shoot_delay: Option::read(buf)?,
      max_distance: Option::read(buf)?,
      use_prediction: bool::read(buf)?,
      state: u8::read(buf)?,
    })
  }
}

pub struct BowAimModule;

impl BowAimModule {
  pub fn new() -> Self {
    Self
  }

  async fn arrows_and_bow_exist(bot: &Client) -> bool {
    let mut arrows_exits = false;
    let mut bow_exits = false;

    if let Some(menu) = bot.get_inventory_menu() {
      for (_, item) in menu.slots().iter().enumerate() {
        if arrows_exits && bow_exits {
          break;
        }

        match item.kind() {
          ItemKind::Arrow | ItemKind::SpectralArrow => arrows_exits = true,
          ItemKind::Bow => bow_exits = true,
          _ => {}
        }
      }
    }

    arrows_exits && bow_exits
  }

  async fn take_bow(bot: &Client, index: &u8) -> bool {
    if let Some(menu) = bot.get_inventory_menu() {
      for (slot, item) in menu.slots().iter().enumerate() {
        if item.kind() == ItemKind::Bow {
          bot.take_item(index, slot, true).await;
          return true;
        }
      }
    }

    false
  }

  async fn predict_entity_position(bot: &Client, entity: Entity) -> Vec3 {
    let old_pos = bot.get_entity_position(entity);

    sleep!(50);

    let mut predicted_pos = bot.get_entity_position(entity);

    let diff_x = predicted_pos.x - old_pos.x;
    let diff_y = predicted_pos.y - old_pos.y;
    let diff_z = predicted_pos.z - old_pos.z;

    let Some(eye_pos) = bot.eye_pos() else {
      return predicted_pos;
    };

    let distance = eye_pos.distance_to(predicted_pos);

    predicted_pos.x += diff_x.powi(1);
    predicted_pos.y += diff_y.powi(1);
    predicted_pos.z += diff_z.powi(1);

    if diff_x != 0.0 {
      predicted_pos.x += diff_x + distance * ((diff_x / 100.0) * 13.0);
    }

    if diff_y != 0.0 {
      predicted_pos.y += diff_y + distance * ((diff_y / 100.0) * 13.0);
    }

    if diff_z != 0.0 {
      predicted_pos.z += diff_z + distance * ((diff_z / 100.0) * 13.0);
    }

    predicted_pos
  }

  async fn start_aiming(index: u8, entity_filter: EntityFilter, distance: f64, use_prediction: bool) {
    let extra_task = tokio::spawn(async move {
      loop {
        if !getst(&index, State::CanLooking).await {
          sleep!(200);
          continue;
        }

        let mut target_entity_exist = false;

        take_bot!(&index, async |bot| {
          if let Some(entity) = bot.find_nearest_entity(&entity_filter, distance) {
            target_entity_exist = true;

            let dest_pos = if use_prediction {
              Self::predict_entity_position(bot, entity).await
            } else {
              bot.get_entity_position(entity)
            };

            bot.look_at(Vec3::new(
              dest_pos.x,
              dest_pos.y + bot.get_entity_eye_height(entity),
              dest_pos.z,
            ));
          }
        });

        if !target_entity_exist {
          sleep!(200);
        } else {
          sleep!(50);
        }
      }
    });

    pushetsk(&index, TaskName::BowAim, extra_task).await;
  }

  async fn shoot(bot: &Client, index: &u8, entity_filter: &EntityFilter, distance: f64, use_prediction: bool) {
    if !getst(index, State::CanInteracting).await || !getst(index, State::CanLooking).await {
      return;
    }

    if !Self::arrows_and_bow_exist(bot).await {
      return;
    }

    let Some(entity) = bot.find_nearest_entity(entity_filter, distance) else {
      return;
    };

    if !Self::take_bow(bot, index).await {
      return;
    }

    Self::start_use_bow(bot, index).await;
    sleep!(randnum(900, 1100));

    let dest_pos = if use_prediction {
      Self::predict_entity_position(bot, entity).await
    } else {
      bot.get_entity_position(entity)
    };

    let Some(eye_pos) = bot.eye_pos() else {
      Self::release_use_bow(bot, index).await;
      return;
    };

    let distance = eye_pos.distance_to(dest_pos);

    bot.look_at(Vec3::new(dest_pos.x, dest_pos.y + distance * 0.09, dest_pos.z));

    if distance > 40.0 {
      bot.jump();

      sleep!(100);

      let dest_pos = if use_prediction {
        Self::predict_entity_position(bot, entity).await
      } else {
        bot.get_entity_position(entity)
      };

      let Some(eye_pos) = bot.eye_pos() else {
        Self::release_use_bow(bot, index).await;
        return;
      };

      let distance = eye_pos.distance_to(dest_pos);

      bot.look_at(Vec3::new(dest_pos.x, dest_pos.y + distance * 0.146, dest_pos.z));
    }

    sleep!(50);

    Self::release_use_bow(bot, index).await;
  }

  async fn start_use_bow(bot: &Client, index: &u8) {
    bot.freeze_move(index).await;
    setst(index, State::CanAttacking, false).await;
    setst(index, State::CanDrinking, false).await;
    setst(index, State::CanEating, false).await;
    setst(index, State::CanLooking, false).await;
    setmst(index, StateName::Interacting, true).await;

    sleep!(50);
    bot.start_use_item_by(InteractionHand::MainHand);
  }

  async fn release_use_bow(bot: &Client, index: &u8) {
    bot.release_use_item();
    setmst(index, StateName::Interacting, false).await;
    bot.unfreeze_move(index).await;
    setst(index, State::CanAttacking, true).await;
    setst(index, State::CanDrinking, true).await;
    setst(index, State::CanEating, true).await;
    setst(index, State::CanLooking, true).await;
  }

  async fn start(bot: &Client, index: &u8, options: &BowAimOptions) {
    let distance = options.max_distance.unwrap_or(70.0);

    Self::start_aiming(*index, options.entity_filter.clone(), distance, options.use_prediction).await;

    loop {
      Self::shoot(bot, index, &options.entity_filter, distance, options.use_prediction).await;
      sleep!(options.shoot_delay.unwrap_or(50));
    }
  }
}

impl SalarixiModule<BowAimOptions> for BowAimModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: std::sync::Arc<BowAimOptions>) -> bool {
    if options.state == 1 && !gettskact(&index, TaskName::BowAim).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, &index, &options).await);
      });

      pushrtsk(&index, TaskName::BowAim, task_handle).await;
    } else if options.state == 0 && gettskact(&index, TaskName::BowAim).await {
      killtsk(&index, TaskName::BowAim).await;

      take_bot!(&index, async |bot| {
        bot.release_use_item();
        bot.unfreeze_move(&index).await;
      });

      setmst(&index, StateName::Looking, false).await;
      setmst(&index, StateName::Interacting, false).await;
      setst(&index, State::CanAttacking, true).await;
      setst(&index, State::CanDrinking, true).await;
      setst(&index, State::CanEating, true).await;
      setst(&index, State::CanLooking, true).await;
    } else {
      return false;
    }

    true
  }
}
