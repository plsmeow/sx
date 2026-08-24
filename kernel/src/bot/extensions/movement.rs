use std::time::Duration;

use azalea::pathfinder::astar::PathfinderTimeout;
use azalea::pathfinder::goals::XZGoal;
use azalea::pathfinder::moves::basic::basic_move;
use azalea::pathfinder::PathfinderOpts;
use azalea::prelude::*;
use azalea::{SprintDirection, StartSprintEvent, StartWalkEvent, WalkDirection};

use crate::bot::systems::states::{getst, setmst, setst, State, StateName};
use crate::take_bot;

pub trait BotMovementExt {
  fn start_walking(&self, index: &u8, direction: WalkDirection) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn start_sprinting(
    &self,
    index: &u8,
    direction: SprintDirection,
  ) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn stop_move(&self, index: &u8) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn freeze_move(&self, index: &u8) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn unfreeze_move(&self, index: &u8) -> impl std::future::Future<Output = ()> + Send + Sync;
}

impl BotMovementExt for Client {
  async fn start_walking(&self, index: &u8, direction: WalkDirection) {
    if getst(index, State::CanWalking).await {
      setmst(index, StateName::Walking, true).await;

      self.ecs.write().write_message(StartWalkEvent {
        entity: self.entity,
        direction: direction,
      });
    }
  }

  async fn start_sprinting(&self, index: &u8, direction: SprintDirection) {
    if getst(index, State::CanSprinting).await {
      setmst(index, StateName::Sprinting, true).await;

      self.ecs.write().write_message(StartSprintEvent {
        entity: self.entity,
        direction: direction,
      });
    }
  }

  async fn stop_move(&self, index: &u8) {
    if !getst(index, State::IsWalking).await && !getst(index, State::IsSprinting).await {
      return;
    }

    self.ecs.write().write_message(StartWalkEvent {
      entity: self.entity,
      direction: WalkDirection::None,
    });

    setmst(index, StateName::Walking, false).await;
    setmst(index, StateName::Sprinting, false).await;
  }

  async fn freeze_move(&self, index: &u8) {
    self.ecs.write().write_message(StartWalkEvent {
      entity: self.entity,
      direction: WalkDirection::None,
    });

    setst(index, State::IsWalking, false).await;
    setst(index, State::IsSprinting, false).await;
    setst(index, State::CanWalking, false).await;
    setst(index, State::CanSprinting, false).await;
  }

  async fn unfreeze_move(&self, index: &u8) {
    setst(index, State::CanWalking, true).await;
    setst(index, State::CanSprinting, true).await;
  }
}

/// Функция безопасного задания координат по X и Z для бота
pub fn go_to(index: u8, x: i32, z: i32) {
  tokio::spawn(async move {
    if !getst(&index, State::CanWalking).await || !getst(&index, State::CanSprinting).await {
      return;
    }

    take_bot!(&index, async |bot| {
      setmst(&index, StateName::Looking, true).await;
      setmst(&index, StateName::Sprinting, true).await;
      setmst(&index, StateName::Walking, true).await;
      setst(&index, State::CanEating, false).await;
      setst(&index, State::CanDrinking, false).await;
      setst(&index, State::CanInteracting, false).await;

      let goal = XZGoal { x: x, z: z };
      let opts = PathfinderOpts::new()
        .min_timeout(PathfinderTimeout::Time(Duration::from_millis(500)))
        .max_timeout(PathfinderTimeout::Time(Duration::from_millis(1000)))
        .allow_mining(false)
        .successors_fn(basic_move);

      bot.goto_with_opts(goal, opts).await;

      setmst(&index, StateName::Looking, false).await;
      setmst(&index, StateName::Sprinting, false).await;
      setmst(&index, StateName::Walking, false).await;
      setst(&index, State::CanEating, true).await;
      setst(&index, State::CanDrinking, true).await;
      setst(&index, State::CanInteracting, true).await;
    });
  });
}
