use bytes::{Buf, BufMut};

use crate::buffer::BufferExt;

impl<T> BufferExt for Option<T>
where
  T: BufferExt,
{
  fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    let is_some = buf.try_get_u8().ok()? == 1;

    if is_some {
      return Some(T::read(buf));
    }

    // Хахаха, почему-то это выглядит смешно...
    Some(None)
  }

  fn write(&self, buf: &mut bytes::BytesMut) {
    if let Some(ty) = self {
      buf.put_u8(1);
      ty.write(buf);
    } else {
      buf.put_u8(0);
    }
  }
}
