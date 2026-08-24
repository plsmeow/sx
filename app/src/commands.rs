use std::fs;
use std::time::Duration;

use salarixi_kernel::sleep;
use systemstat::Platform;

use crate::result::CommandResult;
use crate::session::SESSION;

#[tauri::command]
pub async fn setup_default_session() -> CommandResult<(String, String)> {
  SESSION.run_local_server().await
}

#[tauri::command]
pub async fn change_session(address: String, password: Option<String>) -> CommandResult<()> {
  SESSION.connect_to_remote_server(address, password, true).await
}

#[tauri::command]
pub async fn send_command(data: Vec<u8>) -> CommandResult<()> {
  SESSION.send_command(data).await
}

#[tauri::command]
pub async fn get_ram_usage() -> f64 {
  if let Some(usage) = memory_stats::memory_stats() {
    return (usage.physical_mem as f64 / 1024.0) / 1024.0;
  }

  0.0
}

#[tauri::command]
pub async fn get_cpu_usage() -> f32 {
  let system = systemstat::System::new();

  if let Ok(cpu) = system.cpu_load_aggregate() {
    sleep!(1000);

    if let Ok(load) = cpu.done() {
      return load.user * 100.0;
    }
  }

  0.0
}

#[tauri::command]
pub async fn open_url(url: String) {
  let _ = open::that(url);
}

#[tauri::command]
pub async fn download_json(url: String) -> Option<serde_json::Value> {
  let client = reqwest::Client::new();

  let resp = client
    .get(url)
    .timeout(Duration::from_secs(10))
    .header("Content-Type", "application/json")
    .send()
    .await
    .ok()?;

  if resp.status() != reqwest::StatusCode::OK {
    return None;
  }

  resp.json().await.ok()
}

#[tauri::command]
pub async fn download_text(url: String) -> Option<String> {
  let client = reqwest::Client::new();

  let resp = client.get(url).timeout(Duration::from_secs(10)).send().await.ok()?;

  if resp.status() != reqwest::StatusCode::OK {
    return None;
  }

  resp.text().await.ok()
}

#[tauri::command]
pub fn read_text_file(path: String) -> Vec<u8> {
  match fs::read(&path) {
    Ok(b) => b,
    Err(_) => Vec::new(),
  }
}

#[tauri::command]
pub fn exit() {
  std::process::exit(0x00);
}

#[tauri::command]
pub fn is_dev_mode() -> bool {
  let dev_flag = "dev".to_string();
  std::env::args().collect::<Vec<String>>().contains(&dev_flag)
}
