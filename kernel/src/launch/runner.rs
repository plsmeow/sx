use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use azalea::app::PluginGroup;
use azalea::auto_reconnect::AutoReconnectDelay;
use azalea::prelude::*;
use azalea::protocol::connect::Proxy;
use azalea::swarm::*;
use azalea::JoinOpts;
use socks5_impl::protocol::UserKey;

use crate::bot::handlers::single::single_handler;
use crate::bot::handlers::swarm::swarm_handler;
use crate::bot::systems::index::INDEX_SYSTEM;
use crate::bot::systems::profile::{ProfileProxy, PROFILE_SYSTEM};
use crate::bot::systems::registry::REGISTRY_SYSTEM;
use crate::bot::systems::states::STATE_SYSTEM;
use crate::bot::systems::tasks::TASK_SYSTEM;
use crate::bot::PLUGINS;
use crate::bot::utils::acaptcha::MAP_CAPTCHA_BYPASS;
use crate::launch::process::update_process_status;
use crate::launch::process::BOTS_WERE_FULLY_CONNECTED;
use crate::launch::process::STOPPING;
use crate::server::transfer::{emit_log, emit_msg};
use crate::sleep;
use crate::take_profile;
use crate::webhook::send_webhook;

use super::generators::{generate_email, generate_unique_username, generate_username_or_password};
use super::options::{AccountOptions, LaunchOptions};
use super::process::{current_options, process_is_active, set_options, set_process_activity};
use azalea_viaversion::ViaVersionPlugin;

struct CustomAccount {
  object: Account,
  options: AccountOptions,
  index: u8,
}

/// Вспомогательная функция очистки прокси
fn clear_proxy(proxy: &str) -> &str {
  proxy
    .trim_start_matches("socks5://")
    .trim_start_matches("socks4://")
    .trim_start_matches("https://")
    .trim_start_matches("http://")
}

/// Функция парсинга и обработки прокси
async fn processs_proxy(index: &u8, proxy: &str, join_opts: JoinOpts) -> JoinOpts {
  let mut profile_proxy = ProfileProxy {
    ip_address: None,
    proxy: None,
    username: None,
    password: None,
  };

  let clean_proxy = clear_proxy(proxy);
  let proxy_parts: Vec<&str> = clean_proxy.split("@").collect();

  if let Some(str_addr) = proxy_parts.get(1) {
    if let Ok(socket_addr) = str_addr.parse::<SocketAddr>() {
      profile_proxy.proxy = Some(str_addr.to_string());

      let mut proxy = Proxy::new(socket_addr, None);

      if let Some(auth) = proxy_parts.get(0) {
        let splited_auth: Vec<&str> = auth.split(":").collect();

        let Some(username) = splited_auth.get(0) else {
          return join_opts;
        };

        let Some(password) = splited_auth.get(1) else {
          return join_opts;
        };

        profile_proxy.username = Some(username.to_string());
        profile_proxy.password = Some(password.to_string());

        proxy.auth = Some(UserKey {
          username: username.to_string(),
          password: password.to_string(),
        });
      }

      let splited_addr: Vec<&str> = str_addr.split(":").collect();
      let Some(ip_addr) = splited_addr.get(0) else {
        return join_opts;
      };

      profile_proxy.ip_address = Some(ip_addr.to_string());

      take_profile!(index, async |profile| {
        profile.proxy = profile_proxy;
      });

      return join_opts.proxy(proxy);
    }
  } else {
    if let Some(str_addr) = proxy_parts.get(0) {
      if let Ok(socket_addr) = str_addr.parse::<SocketAddr>() {
        profile_proxy.proxy = Some(str_addr.to_string());

        let proxy = Proxy::new(socket_addr, None);
        let splited_addr: Vec<&str> = str_addr.split(":").collect();
        let Some(ip_addr) = splited_addr.get(0) else {
          return join_opts;
        };

        profile_proxy.ip_address = Some(ip_addr.to_string());

        take_profile!(index, async |profile| {
          profile.proxy = profile_proxy;
        });

        return join_opts.proxy(proxy);
      }
    }
  }

  join_opts
}

/// Функция запуска ботов на сервер
pub async fn launch_bots_on_server(options: LaunchOptions) -> u8 {
  emit_msg("Система", "Запуск ботов...");
  emit_log(format!("Запуск ботов на сервер {}...", options.basic.address), "info");

  // При ошибке логи и сообщения выше придут на фронт позже, чем
  // само сообщение об ошибке, пока пусть будет такая задержка
  sleep!(500);

  if process_is_active() {
    return 0x02;
  }

  let total_bots;

  if options.basic.use_accounts {
    total_bots = options.accounts.len() as u8;
  } else {
    total_bots = options.basic.bots_count;
  }

  if total_bots < 1 {
    return 0x03;
  }

  emit_log("Подготовка к запуску...", "system");

  set_options(options.clone()).await;
  set_process_activity(true);
  update_process_status().await;

  if options.basic.use_webhook {
    send_webhook(
      options.webhook.url.clone(),
      format!("Запуск ботов на сервер {}...", options.basic.address),
    );
  }

  thread::spawn(move || {
    let rt = match tokio::runtime::Runtime::new() {
      Ok(rt) => rt,
      Err(e) => {
        emit_log(format!("Ошибка запуска ботов: {}", e), "error");
        return;
      }
    };

    rt.block_on(async move {
      let local_set = tokio::task::LocalSet::new();

      let mut swarm_plugins = azalea::DefaultPlugins.build();

      if !options.basic.use_auto_rejoin {
        swarm_plugins = swarm_plugins.disable::<azalea::auto_reconnect::AutoReconnectPlugin>();
      } else {
        AutoReconnectDelay::new(Duration::from_millis(options.basic.rejoin_delay.into()));
      }

      let mut bot_plugins = azalea::bot::DefaultBotPlugins.build();

      if !options.basic.use_auto_respawn {
        bot_plugins = bot_plugins.disable::<azalea::auto_respawn::AutoRespawnPlugin>();
      }

      if !options.basic.use_accept_rp {
        bot_plugins = bot_plugins.disable::<azalea::accept_resource_packs::AcceptResourcePacksPlugin>();
      }

      if !options.basic.use_pathfinder {
        bot_plugins = bot_plugins.disable::<azalea::pathfinder::PathfinderPlugin>();
      }

      local_set.spawn_local(async move {
        let mut swarm = SwarmBuilder::new_without_plugins()
          .add_plugins(swarm_plugins)
          .add_plugins(bot_plugins)
          .add_plugins(azalea::swarm::DefaultSwarmPlugins)
          .join_delay(Duration::from_millis(options.basic.join_delay.into()))
          .set_swarm_handler(swarm_handler)
          .set_handler(single_handler);

        if let Some(version) = options.basic.target_version.clone().filter(|v| !v.is_empty()) {
          emit_log(
            format!("Подключение ViaProxy, целевая версия сервера: {}", version),
            "system",
          );

          let proxies_used = options.basic.use_proxy
            || options.accounts.values().any(|account| account.proxy.is_some());

          if proxies_used {
            emit_log(
              "Внимание: прокси и ViaVersion одновременно не поддерживаются, прокси будут проигнорированы",
              "warning",
            );
          }

          swarm = swarm.add_plugins(ViaVersionPlugin::start(version).await);
        }

        let mut accounts = Vec::new();

        if options.basic.use_accounts {
          for (username, opts) in &options.accounts {
            let index = INDEX_SYSTEM.register(username).await;

            accounts.push(CustomAccount {
              object: Account::offline(username),
              options: opts.clone(),
              index,
            });

            PROFILE_SYSTEM
              .register(index, opts.password.clone(), opts.email.clone())
              .await;
          }
        } else {
          for _ in 0..options.basic.bots_count {
            let Some(username) =
              generate_unique_username(&options.basic.nickname_type, &options.basic.nickname_template).await
            else {
              continue;
            };

            let index = INDEX_SYSTEM.register(&username).await;

            let password = generate_username_or_password(
              "password",
              options.basic.password_type.to_str(),
              &options.basic.password_template,
            );

            let email = generate_email(&options.basic.email_type);

            accounts.push(CustomAccount {
              object: Account::offline(&username),
              options: AccountOptions {
                initial_group: None,
                password: None,
                email: None,
                proxy: None,
              },
              index,
            });

            PROFILE_SYSTEM.register(index, password, email).await;
          }
        }

        if options.basic.use_proxy || options.basic.use_accounts {
          let mut accounts_with_opts = Vec::new();

          if options.basic.use_accounts {
            for account in accounts.into_iter() {
              if let Some(initial_group) = account.options.initial_group.clone() {
                take_profile!(&account.index, async |profile| {
                  profile.group = initial_group;
                });
              }

              let mut join_opts = JoinOpts::new();

              if let Some(proxy) = &account.options.proxy {
                join_opts = processs_proxy(&account.index, proxy, join_opts).await;
              }

              accounts_with_opts.push((account, join_opts));
            }
          } else {
            for (i, account) in accounts.into_iter().enumerate() {
              let mut join_opts = JoinOpts::new();

              if let Some(proxy_list) = &options.basic.proxy_list {
                let list: Vec<&str> = proxy_list.split("\n").collect();

                if !list.is_empty() {
                  let proxy = list[i % list.len()];
                  join_opts = processs_proxy(&account.index, proxy, join_opts).await;
                }
              }

              accounts_with_opts.push((account, join_opts));
            }
          }

          for (account, opts) in accounts_with_opts {
            swarm = swarm.add_account_with_opts(account.object, opts);
          }
        } else {
          for account in accounts {
            swarm = swarm.add_account(account.object);
          }
        }

        PROFILE_SYSTEM
          .activate(
            options.basic.monitoring_update_rate as u64,
            options.basic.monitoring_optimization,
          )
          .await;

        emit_log("Подготовка окончена", "system");

        let _ = swarm.start(options.basic.address).await;
      });

      local_set.await;
    });
  });

  0x01
}

/// Функция остановки ботов
pub async fn stop_bots_and_destroy_data(logging: bool) -> u8 {
  let total_bots = PROFILE_SYSTEM.len().await;

  if logging {
    emit_msg("Система", "Остановка ботов...");
    emit_log(format!("Остановка {} ботов...", total_bots), "info");
  }

  sleep!(500);

  if !process_is_active() {
    return 0x02;
  }

  if STOPPING.load(Ordering::SeqCst) {
    return 0x03;
  }

  STOPPING.store(true, Ordering::SeqCst);
  update_process_status().await;

  PLUGINS.clear().await;
  TASK_SYSTEM.clear().await;

  sleep!(1000);

  REGISTRY_SYSTEM.destroy_swarm().await;
  STATE_SYSTEM.clear().await;
  PROFILE_SYSTEM.shutdown().await;
  MAP_CAPTCHA_BYPASS.shutdown().await;
  REGISTRY_SYSTEM.clear().await;
  INDEX_SYSTEM.clear().await;

  set_process_activity(false);

  STOPPING.store(false, Ordering::SeqCst);
  BOTS_WERE_FULLY_CONNECTED.store(false, Ordering::SeqCst);

  sleep!(300);

  update_process_status().await;

  if logging {
    if let Some(options) = current_options().await {
      if options.basic.use_webhook {
        send_webhook(
          options.webhook.url.clone(),
          format!("{} ботов было остановлено", total_bots),
        );
      }
    }
  }

  0x01
}
