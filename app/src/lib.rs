#![feature(ip_as_octets)]

mod commands;
mod emit;
mod modules;
mod result;
mod session;
mod version;

use commands::*;
use modules::cache::{extract_cache_meta, load_cache, save_cache};
use modules::config::{archive_config, export_config, initialize_configs_dir, load_configs};
use modules::discord_rpc::set_discord_rpc;
use modules::ping::get_server_info;
use modules::proxy::{check_proxies, collect_proxies};
use modules::scanner::{start_network_scanning, stop_network_scanning};
use modules::storage::{open_directory, read_file, remove_file, save_file};

use crate::emit::emit_event_loop;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_dialog::init())
    .setup(|app| {
      let handle = app.handle().clone();

      std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(emit_event_loop(handle));
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      setup_default_session,
      change_session,
      send_command,
      exit,
      is_dev_mode,
      get_ram_usage,
      get_cpu_usage,
      get_server_info,
      open_url,
      download_json,
      download_text,
      read_text_file,
      set_discord_rpc,
      collect_proxies,
      check_proxies,
      extract_cache_meta,
      save_cache,
      load_cache,
      initialize_configs_dir,
      load_configs,
      export_config,
      archive_config,
      read_file,
      save_file,
      remove_file,
      open_directory,
      start_network_scanning,
      stop_network_scanning,
    ])
    .run(tauri::generate_context!())
    .expect("error while running client");
}
