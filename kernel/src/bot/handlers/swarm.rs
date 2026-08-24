use azalea::swarm::*;

use crate::bot::systems::registry::REGISTRY_SYSTEM;

/// Обработчик для роя ботов
pub async fn swarm_handler(swarm: Swarm, event: SwarmEvent, _state: NoSwarmState) {
  match event {
    SwarmEvent::Init => {
      REGISTRY_SYSTEM.set_swarm(swarm).await;
    }
    _ => {}
  }
}
