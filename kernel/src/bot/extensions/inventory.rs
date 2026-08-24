use azalea::container::ContainerHandleRef;
use azalea::core::direction::Direction;
use azalea::core::position::BlockPos;
use azalea::entity::inventory::Inventory;
use azalea::inventory::components::{Damage, Enchantments, ItemName, MaxDamage};
use azalea::inventory::operations::ThrowClick;
use azalea::inventory::{ItemStack, Menu};
use azalea::prelude::*;
use azalea::protocol::packets::game::s_player_action::Action;
use azalea::protocol::packets::game::ServerboundPlayerAction;
use azalea::registry::builtin::ItemKind;

use crate::bot::common::convert_inventory_slot_to_hotbar_slot;
use crate::bot::extensions::BotMovementExt;
use crate::bot::systems::states::{getst, setmst, setst, State, StateName};
use crate::sleep;
use crate::tools::randnum;

/// Структура метаданных предмета
pub struct ItemMeta {
  pub kind: String,
  pub count: i32,
  pub current_durability: i32,
  pub max_durability: i32,
  pub enchantments: Vec<(String, i32)>,
}

pub trait BotInventoryExt {
  fn get_selected_slot(&self) -> u8;
  fn get_current_inventory(&self) -> Option<ContainerHandleRef>;
  fn get_inventory_menu(&self) -> Option<Menu>;
  fn get_item_by_slot(&self, slot: usize) -> Option<ItemStack>;
  fn get_item_by_name(&self, name: String) -> Option<(usize, ItemStack)>;
  fn find_empty_slot_in_invenotry(&self) -> Option<usize>;
  fn extract_item_meta(&self, item: &ItemStack) -> Option<ItemMeta>;
  fn start_interacting_with_inventory(&self, index: &u8) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn stop_interacting_with_inventory(&self, index: &u8) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn move_item_to_offhand(&self, kind: ItemKind) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn inventory_click(
    &self,
    index: &u8,
    slot: usize,
    mode: ClickMode,
    lock: bool,
  ) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn inventory_click_on(
    &self,
    index: &u8,
    name: &str,
    mode: ClickMode,
    lock: bool,
  ) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn take_item(
    &self,
    index: &u8,
    source_slot: usize,
    lock: bool,
  ) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn inventory_swap_click(
    &self,
    index: &u8,
    source_slot: usize,
    target_slot: usize,
    lock: bool,
  ) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn inventory_move_item(
    &self,
    index: &u8,
    kind: ItemKind,
    source_slot: usize,
    target_slot: usize,
    lock: bool,
  ) -> impl std::future::Future<Output = ()> + Send + Sync;
}

/// Режим клика в инвентаре
pub enum ClickMode {
  Left,
  Right,
  Shift,
  Drop,
  DropAll,
}

impl From<&str> for ClickMode {
  fn from(value: &str) -> Self {
    match value {
      "left" => Self::Left,
      "right" => Self::Right,
      "drop" => Self::Drop,
      "drop-all" => Self::DropAll,
      _ => Self::Shift,
    }
  }
}

impl BotInventoryExt for Client {
  fn get_current_inventory(&self) -> Option<ContainerHandleRef> {
    if let Some(inventory) = self.get_component::<Inventory>() {
      return Some(ContainerHandleRef::new(inventory.id, self.clone()));
    }

    None
  }

  fn get_selected_slot(&self) -> u8 {
    if let Some(inventory) = self.get_component::<Inventory>() {
      return inventory.selected_hotbar_slot;
    }

    0
  }

  fn get_inventory_menu(&self) -> Option<Menu> {
    if let Some(inventory) = self.get_component::<Inventory>() {
      return Some(inventory.menu().clone());
    }

    None
  }

  fn get_item_by_name(&self, name: String) -> Option<(usize, ItemStack)> {
    let Some(menu) = self.get_inventory_menu() else {
      return None;
    };

    for (slot, item) in menu.slots().iter().enumerate() {
      let Some(item_name) = item.get_component::<ItemName>() else {
        continue;
      };

      if item_name.name.to_string() == name {
        return Some((slot, item.clone()));
      }
    }

    None
  }

  fn get_item_by_slot(&self, slot: usize) -> Option<ItemStack> {
    let Some(menu) = self.get_inventory_menu() else {
      return None;
    };

    menu.slot(slot).cloned()
  }

  fn find_empty_slot_in_invenotry(&self) -> Option<usize> {
    let Some(menu) = self.get_inventory_menu() else {
      return None;
    };

    for (slot, item) in menu.slots().iter().enumerate() {
      if slot > 9 {
        if item.is_empty() {
          return Some(slot);
        }
      }
    }

    None
  }

  fn extract_item_meta(&self, item: &ItemStack) -> Option<ItemMeta> {
    let mut meta = ItemMeta {
      kind: item.kind().to_string(),
      count: item.count(),
      current_durability: -1,
      max_durability: -1,
      enchantments: Vec::new(),
    };

    if let Some(max_durability) = item.get_component::<MaxDamage>() {
      meta.max_durability = max_durability.amount;

      if let Some(damage) = item.get_component::<Damage>() {
        meta.current_durability = max_durability.amount - damage.amount;
      }
    }

    if let Some(enchantments) = item.get_component::<Enchantments>() {
      for (enchantment, level) in &enchantments.levels {
        let Ok(Some(id)) = self.resolve_registry_name(enchantment) else {
          continue;
        };

        meta.enchantments.push((id.to_string(), *level));
      }
    }

    Some(meta)
  }

  async fn start_interacting_with_inventory(&self, index: &u8) {
    self.freeze_move(index).await;

    setst(index, State::CanEating, false).await;
    setst(index, State::CanDrinking, false).await;
    setst(index, State::CanAttacking, false).await;

    setmst(index, StateName::Interacting, true).await;
  }

  async fn stop_interacting_with_inventory(&self, index: &u8) {
    setmst(index, StateName::Interacting, false).await;

    self.unfreeze_move(index).await;

    setst(index, State::CanEating, true).await;
    setst(index, State::CanDrinking, true).await;
    setst(index, State::CanAttacking, true).await;
  }

  async fn take_item(&self, index: &u8, source_slot: usize, lock: bool) {
    if let Some(hotbar_slot) = convert_inventory_slot_to_hotbar_slot(source_slot) {
      if self.get_selected_slot() != hotbar_slot {
        self.set_selected_hotbar_slot(hotbar_slot);
      }
    } else {
      if lock {
        if !getst(index, State::CanInteracting).await {
          return;
        }

        self.start_interacting_with_inventory(index).await;
      }

      if let Some(empty_slot) = self.find_empty_slot_in_invenotry() {
        self
          .inventory_swap_click(index, source_slot, empty_slot as usize, false)
          .await;

        if let Some(slot) = convert_inventory_slot_to_hotbar_slot(empty_slot as usize) {
          if self.get_selected_slot() != slot {
            sleep!(50);
            self.set_selected_hotbar_slot(slot);
          }
        }
      } else {
        let random_slot = randnum(36, 44) as usize;

        self.inventory_click(index, random_slot, ClickMode::Shift, false).await;
        sleep!(50);
        self.inventory_swap_click(index, source_slot, random_slot, false).await;

        sleep!(50);

        let hotbar_slot = convert_inventory_slot_to_hotbar_slot(random_slot).unwrap_or(0);

        if self.get_selected_slot() != hotbar_slot {
          self.set_selected_hotbar_slot(hotbar_slot);
        }
      }

      if lock {
        self.stop_interacting_with_inventory(index).await;
      }
    }
  }

  async fn move_item_to_offhand(&self, kind: ItemKind) {
    if let Some(menu) = self.get_inventory_menu() {
      if let Some(item) = menu.slot(45) {
        if item.kind() == kind {
          return;
        }
      }
    }

    self.write_packet(ServerboundPlayerAction {
      action: Action::SwapItemWithOffhand,
      pos: BlockPos::new(0, 0, 0),
      direction: Direction::Down,
      seq: 0,
    });
  }

  async fn inventory_swap_click(&self, index: &u8, source_slot: usize, target_slot: usize, lock: bool) {
    let Some(inventory) = self.get_current_inventory() else {
      return;
    };

    if lock {
      if !getst(&index, State::CanInteracting).await {
        return;
      }

      self.start_interacting_with_inventory(index).await;
      sleep!(50);
    }

    if let Some(menu) = self.get_inventory_menu() {
      if let Some(item) = menu.slot(target_slot) {
        if !item.is_empty() {
          if let Some(empty_slot) = self.find_empty_slot_in_invenotry() {
            inventory.left_click(target_slot);
            sleep!(50);
            inventory.left_click(empty_slot);
          } else {
            inventory.click(ThrowClick::All {
              slot: target_slot as u16,
            });
          }

          sleep!(50);
        }
      }
    }

    inventory.left_click(source_slot);
    sleep!(50);
    inventory.left_click(target_slot);

    if lock {
      self.stop_interacting_with_inventory(index).await;
      sleep!(50);
    }
  }

  async fn inventory_click(&self, index: &u8, slot: usize, mode: ClickMode, lock: bool) {
    if let Some(inventory) = self.get_current_inventory() {
      if lock {
        if !getst(&index, State::CanInteracting).await {
          return;
        }

        self.start_interacting_with_inventory(index).await;
        sleep!(50);
      }

      match mode {
        ClickMode::Left => {
          inventory.left_click(slot);
        }
        ClickMode::Right => {
          inventory.right_click(slot);
        }
        ClickMode::Shift => {
          inventory.shift_click(slot);
        }
        ClickMode::Drop => {
          inventory.click(ThrowClick::Single { slot: slot as u16 });
        }
        ClickMode::DropAll => {
          inventory.click(ThrowClick::All { slot: slot as u16 });
        }
      }

      if lock {
        self.stop_interacting_with_inventory(index).await;
        sleep!(50);
      }
    }
  }

  async fn inventory_click_on(&self, index: &u8, name: &str, mode: ClickMode, lock: bool) {
    if let Some((slot, _)) = self.get_item_by_name(name.to_string()) {
      self.inventory_click(index, slot, mode, lock).await;
    }
  }

  async fn inventory_move_item(&self, index: &u8, kind: ItemKind, source_slot: usize, target_slot: usize, lock: bool) {
    let Some(item) = self.get_item_by_slot(target_slot) else {
      return;
    };

    if item.kind() == kind {
      return;
    }

    self.inventory_swap_click(index, source_slot, target_slot, lock).await;
  }
}
