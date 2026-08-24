use crate::bot::systems::index::INDEX_SYSTEM;
use crate::bot::systems::profile::PROFILE_SYSTEM;
use crate::launch::options::AuthMode;
use crate::launch::process::current_options;
use crate::server::transfer::emit_log;
use crate::{sleep, take_bot};
use crate::{take_profile, tools::*};

const DEFAULT_AUTH_TEMPLATE: &str = "$cmd $pass";

/// Функция default-авторизации бота
pub async fn default_authorize(index: &u8) {
  let Some(opts) = current_options().await else {
    return;
  };

  let Some(profile) = PROFILE_SYSTEM.get(index).await else {
    return;
  };

  let mut min_delay = 2000;
  let mut max_delay = 4000;
  let mut cmd = "";
  let mut action = "";
  let mut template = DEFAULT_AUTH_TEMPLATE;
  let mut registered = false;
  let mut logined = false;

  if !profile.registered {
    if opts.basic.use_auto_register && opts.basic.register_mode == AuthMode::Default {
      cmd = opts.basic.register_command.trim();
      template = opts.basic.register_template.trim();
      min_delay = opts.basic.register_min_delay;
      max_delay = opts.basic.register_max_delay;
      action = "зарегистрировался";
      registered = true;

      if !opts.basic.use_double_auth {
        logined = true;
      }
    }
  } else if !profile.logined {
    if opts.basic.use_auto_login && opts.basic.login_mode == AuthMode::Default {
      cmd = opts.basic.login_command.trim();
      template = opts.basic.login_template.trim();
      min_delay = opts.basic.login_min_delay;
      max_delay = opts.basic.login_max_delay;
      action = "залогинился";
      logined = true;
    }
  }

  let password = profile.password.clone();
  let email = profile.email.clone();

  drop(profile);

  if cmd.is_empty() {
    return;
  }

  sleep!(randnum(min_delay, max_delay).into());

  let msg = template
    .replace("$cmd", &cmd)
    .replace("$pass", &password.unwrap_or(String::new()))
    .replace("$email", &email.unwrap_or(String::new()));

  take_bot!(index, async |bot| {
    bot.chat(&msg);
  });

  // Нельзя просто поставить значение registered напрямую, так как
  // текущее значения может быть true (то есть бот уже зарегистрирован),
  // и при этом оно сбросится на false, что будет некорректно
  if registered {
    take_profile!(index, async |profile| {
      profile.registered = true;
    });
  }

  if logined {
    take_profile!(index, async |profile| {
      profile.logined = true;
    });
  }

  if let Some(username) = INDEX_SYSTEM.username_by_index(index).await {
    emit_log(format!("Бот {} {}: {}", &username, action, &msg), "info");
  }
}

/// Функция trigger-авторизации бота
pub async fn trigger_authorize(index: &u8, message: String) {
  let Some(opts) = current_options().await else {
    return;
  };

  let Some(profile) = PROFILE_SYSTEM.get(index).await else {
    return;
  };

  let pat = if !profile.registered {
    opts.basic.register_trigger
  } else {
    opts.basic.login_trigger
  };

  if !message.to_lowercase().contains(&pat) {
    return;
  }

  let mut cmd = "";
  let mut action = "";
  let mut template = DEFAULT_AUTH_TEMPLATE;
  let mut registered = false;
  let mut logined = false;

  if !profile.registered {
    if opts.basic.use_auto_register && opts.basic.register_mode == AuthMode::Trigger {
      cmd = opts.basic.register_command.trim();
      template = opts.basic.register_template.trim();
      action = "зарегистрировался";
      registered = true;

      if !opts.basic.use_double_auth {
        logined = true;
      }
    }
  } else if !profile.logined && opts.basic.login_mode == AuthMode::Trigger {
    if opts.basic.use_auto_login {
      cmd = opts.basic.login_command.trim();
      template = opts.basic.login_template.trim();
      action = "залогинился";
      logined = true;
    }
  }

  let password = profile.password.clone();
  let email = profile.email.clone();

  drop(profile);

  if cmd.is_empty() {
    return;
  }

  let msg = template
    .replace("$cmd", &cmd)
    .replace("$pass", &password.unwrap_or(String::new()))
    .replace("$email", &email.unwrap_or(String::new()));

  take_bot!(index, async |bot| {
    bot.chat(&msg);
  });

  if registered {
    take_profile!(index, async |profile| {
      profile.registered = true;
    });
  }

  if logined {
    take_profile!(index, async |profile| {
      profile.logined = true;
    });
  }

  if let Some(username) = INDEX_SYSTEM.username_by_index(index).await {
    emit_log(format!("Бот {} {}: {}", &username, action, &msg), "info");
  }
}
