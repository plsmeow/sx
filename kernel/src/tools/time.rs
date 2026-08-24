/// Вспомогательный макрос остановки кода на указанное количество миллисекунд
#[macro_export]
macro_rules! sleep {
  ($value:expr) => {
    tokio::time::sleep(std::time::Duration::from_millis($value)).await
  };
}
