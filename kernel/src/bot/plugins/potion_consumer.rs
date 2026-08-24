use azalea::entity::{LookDirection, Physics};
use azalea::inventory::components::PotionContents;
use azalea::inventory::ItemStack;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::registry::builtin::ItemKind;
use azalea::registry::builtin::Potion as PotionKind;

use crate::bot::extensions::BotInteractExt;
use crate::bot::extensions::{BotDefaultExt, BotInventoryExt, BotMovementExt, BotPhysicsExt};
use crate::bot::plugin_manager::PluginName;
use crate::bot::systems::states::{getst, setmst, setst, State, StateName};
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::sleep;
use crate::take_bot;
use crate::tools::*;

/// Структура информации и зелье
#[derive(Clone)]
struct Potion {
  kind: String,
  slot: usize,
  name: PotionKind,
}

pub struct PotionConsumerPlugin;

impl PotionConsumerPlugin {
  /// Метод обработки тика
  async fn tick(&self, index: &u8) {
    let mut health = 20.0;

    take_bot!(index, async |bot| {
      health = bot.get_health();
    });

    if health >= 20.0 {
      return;
    }

    let potions = self.find_potion_in_inventory(index).await;

    if potions.len() == 0 {
      return;
    }

    let mut current_physics = None;

    take_bot!(index, async |bot| {
      current_physics = bot.get_physics();
    });

    let Some(physics) = current_physics else {
      return;
    };

    let Some(best_potion) = self.get_best_potion(health, physics, potions) else {
      return;
    };

    if !getst(index, State::CanDrinking).await {
      return;
    }

    let mut should_drink = true;

    if getst(index, State::IsAttacking).await {
      should_drink = !randchance(health as f64 / 20.0);
    }

    if !should_drink {
      return;
    }

    setst(index, State::CanEating, false).await;
    setst(index, State::CanAttacking, false).await;
    setst(index, State::CanInteracting, false).await;

    self.use_potion(index, best_potion).await;

    setst(index, State::CanEating, true).await;
    setst(index, State::CanInteracting, true).await;
    setst(index, State::CanAttacking, true).await;
  }

  /// Метод отправки пакета об использовании зелья
  async fn use_potion(&self, index: &u8, potion: Potion) {
    take_bot!(index, async |bot| {
      bot.take_item(index, potion.slot, false).await;
    });

    match potion.kind.as_str() {
      "default" => {
        setmst(index, StateName::Drinking, true).await;

        take_bot!(index, async |bot| {
          bot.freeze_move(index).await;
          bot.start_use_item_by(InteractionHand::MainHand);
        });

        sleep!(2600);

        take_bot!(index, async |bot| {
          bot.unfreeze_move(index).await;
        });

        setmst(index, StateName::Drinking, false).await;
      }
      "splash" => {
        let mut direction = LookDirection::default();

        take_bot!(index, async |bot| {
          direction = bot.direction().unwrap_or_default();

          let _ = bot.set_direction(direction.y_rot() + randnum(-5.5, 5.5) as f32, randnum(87.0, 90.0) as f32);
        });

        sleep!(randnum(400, 600));

        take_bot!(index, async |bot| {
          bot.start_use_item_by(InteractionHand::MainHand);
        });

        sleep!(randnum(300, 500));

        take_bot!(index, async |bot| {
          let _ = bot.set_direction(
            direction.y_rot() + randnum(-2.5, 2.5) as f32,
            direction.x_rot() + randnum(-2.5, 2.5) as f32,
          );
        });
      }
      _ => {}
    }
  }

  /// Метод получения лучшего зелья из списка
  fn get_best_potion(&self, health: f32, physics: Physics, potions: Vec<Potion>) -> Option<Potion> {
    let is_in_lava = physics.is_in_lava();
    let is_burning = physics.remaining_fire_ticks > 0;
    let is_in_water = physics.is_in_water();
    let on_ground = physics.on_ground();
    let velocity_y = physics.velocity.y;

    let mut best_potion = None;

    for p in potions {
      if best_potion
        .as_ref()
        .unwrap_or(&Potion {
          kind: "deafult".to_string(),
          slot: 0,
          name: PotionKind::Awkward,
        })
        .kind
        .as_str()
        != "splash"
      {
        if best_potion.is_none() && is_in_lava || is_burning {
          match p.name {
            PotionKind::FireResistance => {
              best_potion = Some(p.clone());
            }
            PotionKind::LongFireResistance => {
              best_potion = Some(p.clone());
            }
            _ => {}
          }
        }

        if best_potion.is_none() && is_in_water {
          match p.name {
            PotionKind::WaterBreathing => {
              best_potion = Some(p.clone());
            }
            PotionKind::LongWaterBreathing => {
              best_potion = Some(p.clone());
            }
            _ => {}
          }
        }

        if best_potion.is_none() && !on_ground && velocity_y < -0.5 {
          match p.name {
            PotionKind::SlowFalling => {
              best_potion = Some(p.clone());
            }
            PotionKind::LongSlowFalling => {
              best_potion = Some(p.clone());
            }
            _ => {}
          }
        }

        if best_potion.is_none() && health <= 8.0 {
          match p.name {
            PotionKind::TurtleMaster => {
              best_potion = Some(p.clone());
            }
            PotionKind::LongTurtleMaster => {
              best_potion = Some(p.clone());
            }
            PotionKind::StrongTurtleMaster => {
              best_potion = Some(p.clone());
            }
            _ => {}
          }
        }

        if best_potion.is_none() && health <= 15.0 {
          match p.name {
            PotionKind::Regeneration => {
              best_potion = Some(p);
              break;
            }
            PotionKind::LongRegeneration => {
              best_potion = Some(p);
              break;
            }
            PotionKind::StrongRegeneration => {
              best_potion = Some(p);
              break;
            }
            PotionKind::Healing => {
              best_potion = Some(p);
              break;
            }
            PotionKind::StrongHealing => {
              best_potion = Some(p);
              break;
            }
            _ => {}
          }
        }

        if p.kind.as_str() == "splash" {
          break;
        }
      }
    }

    best_potion
  }

  /// Метод поиска зелья в инвентаре
  async fn find_potion_in_inventory(&self, index: &u8) -> Vec<Potion> {
    let mut potion_list = vec![];

    take_bot!(index, async |bot| {
      let Some(menu) = bot.get_inventory_menu() else {
        return;
      };

      menu.slots().iter().enumerate().for_each(|(slot, item)| {
        if let Some(potion) = self.is_potion(slot, item) {
          potion_list.push(potion);
        }
      });
    });

    potion_list
  }

  /// Метод проверки предмета, является ли тот зельем
  fn is_potion(&self, slot: usize, item: &ItemStack) -> Option<Potion> {
    match item.kind() {
      ItemKind::Potion => {
        if let Some(contents) = item.get_component::<PotionContents>() {
          if let Some(potion) = contents.potion {
            return Some(Potion {
              kind: "default".to_string(),
              slot: slot,
              name: potion,
            });
          }
        }
      }
      ItemKind::SplashPotion => {
        if let Some(contents) = item.get_component::<PotionContents>() {
          if let Some(potion) = contents.potion {
            return Some(Potion {
              kind: "splash".to_string(),
              slot: slot,
              name: potion,
            });
          }
        }
      }
      _ => {}
    }

    None
  }
}

impl SalarixiPlugin for PotionConsumerPlugin {
  fn new() -> Self {
    Self
  }

  fn activate(&'static self, index: u8) {
    let task = tokio::spawn(async move {
      loop {
        self.tick(&index).await;
        sleep!(200);
      }
    });

    PLUGINS.push_task(&index, PluginName::PotionConsumer, task);
  }
}
