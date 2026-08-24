use bytes::{Buf, BufMut};

use crate::buffer::BufferExt;

impl BufferExt for f32 {
  fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    buf.try_get_f32().ok()
  }

  fn write(&self, buf: &mut bytes::BytesMut) {
    buf.put_f32(*self);
  }
}

impl BufferExt for f64 {
  fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    buf.try_get_f64().ok()
  }

  fn write(&self, buf: &mut bytes::BytesMut) {
    buf.put_f64(*self);
  }
}
