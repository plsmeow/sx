use serde::Serialize;

#[derive(Serialize)]
pub struct CommandResult<T> {
  pub data: Option<T>,
  pub error: Option<String>,
}

/// Вспомогательный макрос создания успешного результата
#[macro_export]
macro_rules! success {
  ($data:expr) => {
    return crate::result::CommandResult {
      data: Some($data),
      error: None,
    }
  };
}

/// Вспомогательный макрос создания неуспешного результата
#[macro_export]
macro_rules! failed {
  ($($arg:tt)*) => {
    return crate::result::CommandResult {
      data: None,
      error: Some(format!($($arg)*)),
    }
  };
}
