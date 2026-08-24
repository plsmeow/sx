use std::fs;
use std::path::PathBuf;

use crate::result::CommandResult;
use crate::{failed, success};

/// Вспомогательная функция получения директории кэша
fn cache_dir() -> Option<PathBuf> {
  let home_dir = dirs::home_dir()?;
  let cache_dir = home_dir.join(".salarixi").join("cache");
  if !cache_dir.exists() {
    fs::create_dir_all(&cache_dir).ok()?;
  }

  Some(cache_dir)
}

/// Команда сохранения текущего кэша в файл
#[tauri::command]
pub fn save_cache(cache: Vec<u8>, filename: String, client_version: String, date: String) -> CommandResult<()> {
  let Some(cache_dir) = cache_dir() else {
    failed!("cache directory is missing");
  };

  let current_path = cache_dir.join(&filename);
  if fs::write(current_path, &cache).is_err() {
    failed!("failed to save cache to specified file");
  }

  let filename_split = filename.split('.').collect::<Vec<&str>>();
  let meta_path = cache_dir.join(format!("{}.meta.txt", filename_split[0]));
  if fs::write(meta_path, format!("{} {}", client_version, date).as_bytes()).is_err() {
    failed!("failed to save cache meta to specified file");
  }

  success!(());
}

/// Команда извлечения метаданных кэша
#[tauri::command]
pub fn extract_cache_meta(name: String) -> CommandResult<String> {
  let Some(cache_dir) = cache_dir() else {
    failed!("cache directory is missing");
  };

  let meta_path = cache_dir.join(format!("{}.meta.txt", name));
  let meta = match fs::read(meta_path) {
    Ok(b) => String::from_utf8_lossy(&b).trim().to_string(),
    Err(_) => failed!("could not read the specified file"),
  };

  success!(meta);
}

/// Команда загрузки кэша из файла
#[tauri::command]
pub async fn load_cache(filename: String) -> CommandResult<Vec<u8>> {
  let Some(cache_dir) = cache_dir() else {
    failed!("cache directory is missing");
  };

  let current_path = cache_dir.join(&filename);
  let bytes = match fs::read(current_path) {
    Ok(b) => b,
    Err(_) => failed!("could not read the specified file"),
  };

  if bytes.is_empty() {
    failed!("specified file does not contain anything");
  }

  success!(bytes);
}
