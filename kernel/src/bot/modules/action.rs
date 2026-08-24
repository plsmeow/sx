use std::time::Duration;

use azalea::prelude::*;
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use salarixi_macros::Index;
use tokio::time::sleep;

use crate::bot::extensions::BotDefaultExt;
use crate::bot::systems::tasks::{gettskact, killtsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::tools::*;
use crate::{sleep, take_bot};

#[derive(Index)]
enum Action {
  Jumping = 0,
  Shifting = 1,
  Waving = 2,
}

pub struct ActionOptions {
  action: Action,
  use_sync: bool,
  use_impulsiveness: bool,
  state: u8,
}

impl ActionOptions {
  pub fn from_bytes(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      action: Action::from_index(u8::read(buf)?)?,
      use_sync: bool::read(buf)?,
      use_impulsiveness: bool::read(buf)?,
      state: u8::read(buf)?,
    })
  }
}

pub struct ActionModule;

impl ActionModule {
  async fn jumping(bot: &Client, options: &ActionOptions) {
    if options.use_impulsiveness {
      loop {
        if options.use_sync {
          sleep(Duration::from_millis(1000)).await;
        } else {
          sleep(Duration::from_millis(randnum(900, 1800))).await;
        }

        bot.jump();
      }
    } else {
      if !options.use_sync {
        sleep(Duration::from_millis(randnum(200, 2000))).await;
      }

      bot.set_jumping(true);
    }
  }

  async fn shifting(bot: &Client, options: &ActionOptions) {
    if options.use_impulsiveness {
      loop {
        if options.use_sync {
          sleep!(1000);
        } else {
          sleep!(randnum(900, 1800));
        }

        bot.set_crouching(true);
        sleep!(350);
        bot.set_crouching(false);
      }
    } else {
      if !options.use_sync {
        sleep!(randnum(200, 2000));
      }

      bot.set_crouching(true);
    }
  }

  async fn waving(bot: &Client, options: &ActionOptions) {
    if options.use_impulsiveness {
      loop {
        if options.use_sync {
          sleep!(300);
        } else {
          sleep!(randnum(300, 800));
        }

        bot.swing_arm();
      }
    } else {
      if !options.use_sync {
        sleep!(randnum(200, 2000));
      }

      loop {
        bot.swing_arm();
        sleep!(300);
      }
    }
  }

  async fn start(bot: &Client, options: &ActionOptions) {
    match options.action {
      Action::Jumping => Self::jumping(bot, options).await,
      Action::Shifting => Self::shifting(bot, options).await,
      Action::Waving => Self::waving(bot, options).await,
    }
  }
}

impl SalarixiModule<ActionOptions> for ActionModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: std::sync::Arc<ActionOptions>) -> bool {
    let task_name = match options.action {
      Action::Jumping => TaskName::Jumping,
      Action::Shifting => TaskName::Shifting,
      Action::Waving => TaskName::Waving,
    };

    if options.state == 1 && !gettskact(&index, task_name.clone()).await {
      let root_task = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, &options).await);
      });

      pushrtsk(&index, task_name, root_task).await;
    } else if options.state == 0 && gettskact(&index, task_name.clone()).await {
      killtsk(&index, task_name).await;

      take_bot!(&index, async |bot| match options.action {
        Action::Jumping => bot.set_jumping(false),
        Action::Shifting => bot.set_crouching(false),
        _ => {}
      });
    } else {
      return false;
    }

    true
  }
}
