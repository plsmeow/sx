use azalea::registry::builtin::ItemKind;
use azalea::SprintDirection;
use azalea::{prelude::*, WalkDirection};
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use salarixi_macros::Index;

use crate::bot::extensions::{BotDefaultExt, BotInventoryExt, BotMovementExt, BotRotationExt, EntityFilter};
use crate::bot::systems::states::{getst, setmst, setst, State, StateName};
use crate::bot::systems::tasks::{gettskact, killtsk, pushetsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::tools::*;
use crate::{sleep, take_bot};

struct Weapon {
  slot: Option<usize>,
  material_priority: u8,
  enchantments_priority: i32,
}

#[derive(PartialEq, Index)]
enum Behavior {
  Moderate = 0,
  Aggressive = 1,
}

#[derive(PartialEq, Index)]
enum Settings {
  Adaptive = 0,
  Manual = 1,
}

#[derive(PartialEq, Index)]
enum WeaponType {
  Sword = 0,
  Axe = 1,
}

pub struct KillauraOptions {
  behavior: Behavior,
  settings: Settings,
  weapon: WeaponType,
  entity_filter: EntityFilter,
  weapon_slot: Option<u8>,
  attack_distance: Option<f64>,
  attack_delay: Option<u64>,
  chase_distance: Option<f64>,
  min_distance_to_target: Option<f64>,
  use_auto_weapon: bool,
  use_dodging: bool,
  use_chase: bool,
  use_critical: bool,
  state: u8,
}

impl KillauraOptions {
  pub fn from_bytes(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      behavior: Behavior::from_index(u8::read(buf)?)?,
      settings: Settings::from_index(u8::read(buf)?)?,
      weapon: WeaponType::from_index(u8::read(buf)?)?,
      entity_filter: EntityFilter::read(buf)?,
      weapon_slot: Option::read(buf)?,
      attack_distance: Option::read(buf)?,
      attack_delay: Option::read(buf)?,
      chase_distance: Option::read(buf)?,
      min_distance_to_target: Option::read(buf)?,
      use_auto_weapon: bool::read(buf)?,
      use_dodging: bool::read(buf)?,
      use_chase: bool::read(buf)?,
      use_critical: bool::read(buf)?,
      state: u8::read(buf)?,
    })
  }
}

struct KillauraConfig {
  weapon_slot: Option<u8>,
  attack_distance: f64,
  attack_delay: u64,
  chase_distance: f64,
  min_distance_to_target: f64,
}

pub struct KillauraModule;

impl KillauraModule {
  pub fn new() -> Self {
    Self
  }

  /// Метод создания конфига
  fn create_config(options: &KillauraOptions) -> KillauraConfig {
    if options.settings == Settings::Adaptive {
      KillauraConfig {
        weapon_slot: options.weapon_slot,
        attack_distance: 3.1,
        attack_delay: if options.behavior == Behavior::Moderate {
          500
        } else {
          350
        },
        chase_distance: options.chase_distance.unwrap_or(10.0),
        min_distance_to_target: options.min_distance_to_target.unwrap_or(3.0),
      }
    } else {
      KillauraConfig {
        weapon_slot: options.weapon_slot,
        attack_distance: options.attack_distance.unwrap_or(3.1),
        attack_delay: options.attack_delay.unwrap_or(500),
        chase_distance: options.chase_distance.unwrap_or(10.0),
        min_distance_to_target: options.min_distance_to_target.unwrap_or(3.0),
      }
    }
  }

  /// Метод вычисления приоретета предмета по чарам
  fn calc_item_enchantments_priority(enchantments: &Vec<(String, i32)>) -> i32 {
    let mut priority = 0;

    for (id, level) in enchantments {
      match id.as_str() {
        "minecraft:fire_aspect" => priority += *level,
        "minecraft:knockback" => priority += *level,
        "minecraft:looting" => priority += *level,
        "minecraft:smite" => priority += *level,
        "minecraft:sweeping_edge" => priority += *level,
        "minecraft:unbreaking" => priority += *level,

        // Никогда не понимал что делает эта чара, хаха,
        // думаю, стоит занизить ей приоретет на единицу
        "minecraft:bane_of_arthropods" => priority += if *level > 0 { *level - 1 } else { 0 },

        // Этим чарам явно требуется повышенный приоретет
        "minecraft:mending" => priority += 2,
        "minecraft:sharpness" => priority += *level + 1,

        _ => {}
      }
    }

    priority
  }

  /// Метод взятия оружия в руку
  async fn take_weapon(bot: &Client, index: &u8, weapon_type: &WeaponType) {
    let Some(menu) = bot.get_inventory_menu() else {
      return;
    };

    let mut best_weapon = Weapon {
      slot: None,
      material_priority: 0,
      enchantments_priority: 0,
    };

    for (slot, item) in menu.slots().iter().enumerate() {
      if item.is_empty() {
        continue;
      }

      let Some(meta) = bot.extract_item_meta(item) else {
        continue;
      };

      let mut material_priority = None;

      match weapon_type {
        WeaponType::Sword => match item.kind() {
          ItemKind::WoodenSword => {
            material_priority = Some(0);
          }
          ItemKind::GoldenSword => {
            material_priority = Some(1);
          }
          ItemKind::StoneSword => {
            material_priority = Some(2);
          }
          ItemKind::CopperSword => {
            material_priority = Some(3);
          }
          ItemKind::IronSword => {
            material_priority = Some(4);
          }
          ItemKind::DiamondSword => {
            material_priority = Some(5);
          }
          ItemKind::NetheriteSword => {
            material_priority = Some(6);
          }
          _ => {}
        },
        WeaponType::Axe => match item.kind() {
          ItemKind::WoodenAxe => {
            material_priority = Some(0);
          }
          ItemKind::GoldenAxe => {
            material_priority = Some(1);
          }
          ItemKind::StoneAxe => {
            material_priority = Some(2);
          }
          ItemKind::CopperAxe => {
            material_priority = Some(3);
          }
          ItemKind::IronAxe => {
            material_priority = Some(4);
          }
          ItemKind::DiamondAxe => {
            material_priority = Some(5);
          }
          ItemKind::NetheriteAxe => {
            material_priority = Some(6);
          }
          _ => {}
        },
      }

      let Some(mp) = material_priority else {
        continue;
      };

      let ep = Self::calc_item_enchantments_priority(&meta.enchantments);

      let material_diff = if best_weapon.material_priority > mp {
        best_weapon.material_priority - mp
      } else {
        mp - best_weapon.material_priority
      };

      if material_diff >= 2 {
        best_weapon = Weapon {
          slot: Some(slot),
          material_priority: mp,
          enchantments_priority: ep,
        };

        continue;
      }

      let enchantments_diff = if best_weapon.enchantments_priority > ep {
        best_weapon.enchantments_priority - ep
      } else {
        ep - best_weapon.enchantments_priority
      };

      if enchantments_diff >= 4 {
        best_weapon = Weapon {
          slot: Some(slot),
          material_priority: mp,
          enchantments_priority: ep,
        };

        continue;
      }

      if mp > best_weapon.material_priority {
        best_weapon = Weapon {
          slot: Some(slot),
          material_priority: mp,
          enchantments_priority: ep,
        };

        continue;
      }

      if ep > best_weapon.enchantments_priority {
        best_weapon = Weapon {
          slot: Some(slot),
          material_priority: mp,
          enchantments_priority: ep,
        };
      }
    }

    if let Some(slot) = best_weapon.slot {
      bot.take_item(index, slot, false).await;
    }
  }

  /// Метод включения преследования
  async fn start_chase(index: u8, entity_filter: EntityFilter, distance: f64, min_distance_to_target: f64) {
    let extra_task = tokio::spawn(async move {
      loop {
        let mut target_entity_exist = false;

        take_bot!(&index, async |bot| {
          if let Some(entity) = bot.find_nearest_entity(&entity_filter, distance) {
            let Some(eye_pos) = bot.eye_pos() else {
              return;
            };

            if eye_pos.distance_to(bot.get_entity_position(entity)) > min_distance_to_target {
              target_entity_exist = true;

              if eye_pos.distance_to(bot.get_entity_position(entity)) - min_distance_to_target
                > min_distance_to_target * 1.5
              {
                bot.jump();
              }

              bot.start_sprinting(&index, SprintDirection::Forward).await;

              if getst(&index, State::CanLooking).await {
                setmst(&index, StateName::Looking, true).await;
                sleep!(50);
                bot.look_at_entity(entity, false);
                sleep!(50);
                setmst(&index, StateName::Looking, false).await;
              }
            } else {
              if getst(&index, State::IsSprinting).await {
                bot.stop_move(&index).await;
              }
            }
          } else {
            if getst(&index, State::IsSprinting).await {
              bot.stop_move(&index).await;
            }
          }
        });

        if target_entity_exist {
          sleep!(50);
        } else {
          sleep!(200);
        }
      }
    });

    pushetsk(&index, TaskName::Killaura, extra_task).await;
  }

  /// Метод включения автоматической наводки
  async fn start_aiming(index: u8, entity_filter: EntityFilter, distance: f64) {
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

            setmst(&index, StateName::Looking, true).await;
            sleep!(50);
            bot.look_at_entity(entity, false);
            sleep!(50);
            setmst(&index, StateName::Looking, false).await;
          }
        });

        if !target_entity_exist {
          sleep!(200);
        }
      }
    });

    pushetsk(&index, TaskName::Killaura, extra_task).await;
  }

  /// Метод уклонения от атаки целевой сущности
  async fn dodge(bot: &Client, index: &u8) {
    let num = randnum(0, 2);

    match num {
      0 => {
        let direction = randelem(&[
          WalkDirection::Backward,
          WalkDirection::BackwardLeft,
          WalkDirection::BackwardRight,
        ]);

        bot.start_walking(index, *direction).await;
        sleep!(randnum(200, 300));
        bot.stop_move(index).await;
      }
      1 => {
        bot.start_crouching();
        sleep!(randnum(300, 400));
        bot.stop_crouching();
      }
      2 => {
        let direction = randelem(&[WalkDirection::ForwardLeft, WalkDirection::ForwardRight]);

        bot.start_walking(index, *direction).await;
        sleep!(randnum(300, 500));
        bot.stop_move(index).await;
      }
      _ => {}
    }
  }

  /// Метод атаки указанной сущности
  async fn attack(bot: &Client, index: &u8, options: &KillauraOptions, config: &KillauraConfig) {
    if !getst(index, State::CanAttacking).await {
      return;
    }

    let Some(entity) = bot.find_nearest_entity(&options.entity_filter, config.attack_distance) else {
      return;
    };

    setst(index, State::CanDrinking, false).await;
    setst(index, State::CanInteracting, false).await;
    setst(index, State::CanEating, false).await;
    setmst(index, StateName::Attacking, true).await;

    if options.use_auto_weapon {
      Self::take_weapon(bot, index, &options.weapon).await;
    } else {
      if let Some(slot) = config.weapon_slot {
        if slot <= 8 {
          bot.set_selected_hotbar_slot(slot);
        }
      }
    }

    if options.use_critical {
      bot.jump();

      sleep!(randnum(400, 500));

      let Some(eye_pos) = bot.eye_pos() else {
        return;
      };

      let entity_pos = bot.get_entity_position(entity);
      let distance = eye_pos.distance_to(entity_pos);

      if distance <= config.attack_distance {
        bot.attack(entity);
        return;
      }
    } else {
      bot.attack(entity);
    }

    if options.use_dodging {
      Self::dodge(bot, index).await;
    }

    setmst(index, StateName::Attacking, false).await;
    setst(index, State::CanDrinking, true).await;
    setst(index, State::CanInteracting, true).await;
    setst(index, State::CanEating, true).await;
  }

  /// Метод запуска умеренной киллауры
  async fn moderate_killaura(bot: &Client, index: u8, options: &KillauraOptions) {
    let config = Self::create_config(options);

    Self::start_aiming(index, options.entity_filter.clone(), config.attack_distance).await;

    if options.use_chase {
      Self::start_chase(
        index,
        options.entity_filter.clone(),
        config.chase_distance,
        config.min_distance_to_target,
      )
      .await;
    }

    loop {
      Self::attack(bot, &index, options, &config).await;
      sleep!(config.attack_delay);
    }
  }

  /// Метод запуска агрессивной киллауры
  async fn aggressive_killaura(bot: &Client, index: u8, options: &KillauraOptions) {
    let config = Self::create_config(options);

    if options.use_chase {
      Self::start_chase(
        index,
        options.entity_filter.clone(),
        config.chase_distance,
        config.min_distance_to_target,
      )
      .await;
    }

    loop {
      Self::attack(bot, &index, options, &config).await;
      sleep!(config.attack_delay);
    }
  }

  /// Метод запуска киллауры
  async fn start(bot: &Client, index: u8, options: &KillauraOptions) {
    match options.behavior {
      Behavior::Moderate => Self::moderate_killaura(bot, index, &options).await,
      Behavior::Aggressive => Self::aggressive_killaura(bot, index, &options).await,
    }
  }
}

impl SalarixiModule<KillauraOptions> for KillauraModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: std::sync::Arc<KillauraOptions>) -> bool {
    if options.state == 1 && !gettskact(&index, TaskName::Killaura).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, index, &options).await);
      });

      pushrtsk(&index, TaskName::Killaura, task_handle).await;
    } else if options.state == 0 && gettskact(&index, TaskName::Killaura).await {
      killtsk(&index, TaskName::Killaura).await;

      take_bot!(&index, async |bot| {
        bot.stop_move(&index).await;
        bot.stop_crouching();
      });

      setmst(&index, StateName::Looking, false).await;
      setmst(&index, StateName::Attacking, false).await;
    } else {
      return false;
    }

    true
  }
}
