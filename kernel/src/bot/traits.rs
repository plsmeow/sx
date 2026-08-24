/// Трейт модуля
pub trait SalarixiModule<T> {
  fn new() -> Self;
  fn switch(&self, index: u8, options: std::sync::Arc<T>) -> impl std::future::Future<Output = bool> + Send + Sync;
}

/// Трейт плагина
pub trait SalarixiPlugin {
  fn new() -> Self;
  fn activate(&'static self, index: u8);
}
