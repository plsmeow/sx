use bytes::{Buf, BufMut};

use crate::buffer::BufferExt;

impl BufferExt for i32 {
  fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    buf.try_get_i32().ok()
  }

  fn write(&self, buf: &mut bytes::BytesMut) {
    buf.put_i32(*self);
  }
}
