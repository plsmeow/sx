pub trait IndexExt
where
  Self: Sized,
{
  fn from_index(index: u8) -> Option<Self>;
}
