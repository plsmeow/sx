use bytes::{Bytes, BytesMut};

pub trait BufferExt
where
  Self: Sized,
{
  fn read(buf: &mut Bytes) -> Option<Self>;
  fn write(&self, buf: &mut BytesMut);
}
