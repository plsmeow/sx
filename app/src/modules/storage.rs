use std::fs;
use std::process::Command;

use crate::result::CommandResult;
use crate::{failed, success};

/// Команда удаления указанного файла из хранилища
#[tauri::command]
pub fn remove_file(path: String) -> CommandResult<()> {
  let Some(home_dir) = dirs::home_dir() else {
    failed!("failed to get home directory");
  };

  let salarixi_dir = home_dir.join(".salarixi");
  if !salarixi_dir.exists() {
    if fs::create_dir_all(&salarixi_dir).is_err() {
      failed!("salarixi directory is missing");
    }
  }

  let target_path = salarixi_dir.join(path);
  if !target_path.exists() {
    success!(());
  }

  match fs::remove_file(target_path) {
    Err(e) => failed!("file deletion error: {}", e),
    _ => {}
  };

  success!(());
}

/// Команда чтения указанного файла из хранилища
#[tauri::command]
pub fn read_file(path: String) -> CommandResult<Vec<u8>> {
  let Some(home_dir) = dirs::home_dir() else {
    failed!("failed to get home directory");
  };

  let salarixi_dir = home_dir.join(".salarixi");
  if !salarixi_dir.exists() {
    if fs::create_dir_all(&salarixi_dir).is_err() {
      failed!("salarixi directory is missing");
    }
  }

  let target_path = salarixi_dir.join(path);
  if !target_path.exists() {
    success!(Vec::new());
  }

  let content = match fs::read(target_path) {
    Ok(b) => b,
    Err(e) => failed!("file read error: {}", e),
  };

  success!(content);
}

/// Команда сохранения указанного файла в хранилище
#[tauri::command]
pub fn save_file(path: String, content: Vec<u8>) -> CommandResult<()> {
  let Some(home_dir) = dirs::home_dir() else {
    failed!("failed to get home directory");
  };

  let salarixi_dir = home_dir.join(".salarixi");
  if !salarixi_dir.exists() {
    if fs::create_dir_all(&salarixi_dir).is_err() {
      failed!("salarixi directory is missing");
    }
  }

  let target_path = salarixi_dir.join(path);
  match fs::write(target_path, &content) {
    Ok(_) => {}
    Err(e) => failed!("file write error: {}", e),
  };

  success!(());
}

/// Команда открытия указанной директории хранилища
#[tauri::command]
pub fn open_directory(dir: String) -> CommandResult<()> {
  let Some(home_dir) = dirs::home_dir() else {
    failed!("failed to get home directory");
  };

  let salarixi_dir = home_dir.join(".salarixi");
  if !salarixi_dir.exists() {
    if fs::create_dir_all(&salarixi_dir).is_err() {
      failed!("salarixi directory is missing");
    }
  }

  let target_path = salarixi_dir.join(&dir);

  if cfg!(target_os = "windows") {
    match Command::new("explorer").arg(&target_path).spawn() {
      Ok(_) => {}
      Err(e) => failed!("error opening directory \"{}\" in storage: {}", dir, e),
    }
  } else if cfg!(target_os = "linux") {
    match Command::new("xdg-open").arg(&target_path).spawn() {
      Ok(_) => {}
      Err(e) => failed!("error opening directory \"{}\" in storage: {}", dir, e),
    }
  }
  if cfg!(target_os = "macos") {
    match Command::new("open").arg(&target_path).spawn() {
      Ok(_) => {}
      Err(e) => failed!("error opening directory \"{}\" in storage: {}", dir, e),
    }
  }

  success!(());
}
