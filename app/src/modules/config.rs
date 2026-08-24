use std::fs;
use std::path::PathBuf;

use hashbrown::HashMap;
use salarixi_kernel::tools::{randstr, CharClass};

use crate::result::CommandResult;
use crate::{failed, success};

/// Команда инициализации директории пользовательских конфигов
#[tauri::command]
pub fn initialize_configs_dir() -> CommandResult<()> {
  let Some(home_dir) = dirs::home_dir() else {
    failed!("failed to get home directory");
  };

  let salarixi_dir = home_dir.join(".salarixi");
  if !salarixi_dir.exists() {
    if fs::create_dir_all(&salarixi_dir).is_err() {
      failed!("salarixi directory is missing");
    }
  }

  let configs_dir = salarixi_dir.join("configs");
  if !configs_dir.exists() {
    if fs::create_dir_all(configs_dir).is_err() {
      failed!("configs directory is missing");
    }
  }

  success!(());
}

/// Команда загрузки всех пользовательских конфигов
#[tauri::command]
pub fn load_configs() -> CommandResult<HashMap<String, Vec<u8>>> {
  let Some(home_dir) = dirs::home_dir() else {
    failed!("failed to get home directory");
  };

  let salarixi_dir = home_dir.join(".salarixi");
  if !salarixi_dir.exists() {
    if fs::create_dir_all(&salarixi_dir).is_err() {
      failed!("salarixi directory is missing");
    }
  }

  let configs_dir = salarixi_dir.join("configs");
  if !configs_dir.exists() {
    if fs::create_dir_all(&configs_dir).is_err() {
      failed!("configs directory is missing");
    }
  }

  let mut configs = HashMap::new();

  let entries = match configs_dir.read_dir() {
    Ok(e) => e,
    Err(e) => failed!("config directory read error: {}", e),
  };

  for entry_result in entries {
    let Ok(entry) = entry_result else {
      continue;
    };

    let Ok(ty) = entry.file_type() else {
      continue;
    };

    if !ty.is_file() {
      continue;
    }

    let path = entry.path();

    let Some(ext) = path.extension() else {
      continue;
    };

    if ext != "json" {
      continue;
    }

    let os_filename = entry.file_name();
    let Some(str_filename) = os_filename.to_str() else {
      continue;
    };
    let filename = str_filename.to_string();

    let content = match fs::read(path) {
      Ok(c) => c,
      Err(_) => continue,
    };

    configs.insert(filename.to_string(), content);
  }

  success!(configs);
}

/// Команда экспорта публичного конфига
#[tauri::command]
pub fn export_config(directory: String, content: Vec<u8>) -> CommandResult<String> {
  let config_dir = PathBuf::from(directory);
  let mut config_filename = "config.json".to_string();

  if let Ok(entries) = config_dir.read_dir() {
    let mut exist_filenames = Vec::new();
    for entry in entries {
      if let Ok(e) = entry {
        let entry_filename = e.file_name();

        if let Some(str) = entry_filename.to_str() {
          exist_filenames.push(str.to_string());
        }
      }
    }

    for _ in 0..100 {
      let current_filename = format!("config_{}", randstr(CharClass::Multi, 8));
      if !exist_filenames.contains(&current_filename) {
        config_filename = format!("{}.json", current_filename);
        break;
      }
    }
  }

  let config_path = config_dir.join(&config_filename);
  match fs::write(config_path, &content) {
    Ok(_) => {}
    Err(e) => failed!("config export error: {}", e),
  };

  success!(config_filename);
}

/// Команда архивации конфига
#[tauri::command]
pub fn archive_config(content: Vec<u8>) -> CommandResult<String> {
  let Some(home_dir) = dirs::home_dir() else {
    failed!("failed to get home directory");
  };

  let salarixi_dir = home_dir.join(".salarixi");
  if !salarixi_dir.exists() {
    if fs::create_dir_all(&salarixi_dir).is_err() {
      failed!("salarixi directory is missing");
    }
  }

  let configs_dir = salarixi_dir.join("configs");
  if !configs_dir.exists() {
    if fs::create_dir_all(&configs_dir).is_err() {
      failed!("configs directory is missing");
    }
  }

  let mut config_filename = "archive_config.json".to_string();

  if let Ok(entries) = configs_dir.read_dir() {
    let mut exist_filenames = Vec::new();
    for entry in entries {
      if let Ok(e) = entry {
        let entry_filename = e.file_name();

        if let Some(str) = entry_filename.to_str() {
          exist_filenames.push(str.to_string());
        }
      }
    }

    for _ in 0..100 {
      let current_filename = format!("config_{}", randstr(CharClass::Multi, 8));
      if !exist_filenames.contains(&current_filename) {
        config_filename = format!("{}.json", current_filename);
        break;
      }
    }
  }

  let target_path = configs_dir.join(&config_filename);
  match fs::write(&target_path, &content) {
    Ok(_) => {}
    Err(e) => failed!("config save error: {}", e),
  };

  success!(config_filename);
}
