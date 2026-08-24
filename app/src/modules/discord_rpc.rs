use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::{failed, result::CommandResult, success, version::CLIENT_VERSION_STR};

static DISCORD_RPC: Lazy<DiscordRpc> = Lazy::new(|| DiscordRpc::new());

pub struct DiscordRpc {
  client: RwLock<Option<discord_presence::Client>>,
}

impl DiscordRpc {
  pub fn new() -> Self {
    Self {
      client: RwLock::new(None),
    }
  }

  pub async fn enable(&self) -> CommandResult<()> {
    *self.client.write().await = Some(discord_presence::Client::new(1477312950271213729));

    if let Some(client) = self.client.write().await.as_mut() {
      client.start();
      let _ = client.block_until_event(discord_presence::Event::Ready);

      match client.set_activity(|act| {
        act
          .details(format!("Версия: {}", CLIENT_VERSION_STR))
          .state("Сайт: https://salarixi.freedev.app/")
      }) {
        Ok(_) => success!(()),
        Err(e) => failed!("{}", e),
      }
    }

    failed!("failed to start the client");
  }

  pub async fn disable(&self) -> CommandResult<()> {
    if let Some(client) = self.client.write().await.take() {
      match client.shutdown() {
        Ok(_) => {}
        Err(e) => failed!("{}", e),
      }
    }

    success!(());
  }
}

#[tauri::command]
pub async fn set_discord_rpc(state: bool) -> CommandResult<()> {
  if state {
    DISCORD_RPC.enable().await
  } else {
    DISCORD_RPC.disable().await
  }
}
