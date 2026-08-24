use bytes::{Buf, BufMut};

use crate::buffer::BufferExt;

impl BufferExt for bool {
  fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(buf.try_get_u8().ok()? == 1)
  }

  fn write(&self, buf: &mut bytes::BytesMut) {
    buf.put_u8(if *self { 1 } else { 0 });
  }
}
