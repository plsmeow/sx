use azalea::core::direction::Direction;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::protocol::packets::game::s_player_action::Action;
use azalea::protocol::packets::game::{ServerboundPlayerAction, ServerboundUseItem};
use azalea::{BlockPos, Client};

pub trait BotInteractExt {
  fn start_use_item_by(&self, hand: InteractionHand);
  fn release_use_item(&self);
}

impl BotInteractExt for Client {
  fn start_use_item_by(&self, hand: InteractionHand) {
    let direction = self.direction().unwrap_or_default();

    self.write_packet(ServerboundUseItem {
      hand: hand,
      y_rot: direction.y_rot(),
      x_rot: direction.x_rot(),
      seq: 0,
    });
  }

  fn release_use_item(&self) {
    self.write_packet(ServerboundPlayerAction {
      action: Action::ReleaseUseItem,
      pos: BlockPos::new(0, 0, 0),
      direction: Direction::Down,
      seq: 0,
    });
  }
}
