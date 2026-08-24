use bytes::{Buf, BufMut};

use crate::buffer::BufferExt;

impl BufferExt for u8 {
  fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    buf.try_get_u8().ok()
  }

  fn write(&self, buf: &mut bytes::BytesMut) {
    buf.put_u8(*self);
  }
}

impl BufferExt for u16 {
  fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    buf.try_get_u16().ok()
  }

  fn write(&self, buf: &mut bytes::BytesMut) {
    buf.put_u16(*self);
  }
}

impl BufferExt for u32 {
  fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    buf.try_get_u32().ok()
  }

  fn write(&self, buf: &mut bytes::BytesMut) {
    buf.put_u32(*self);
  }
}

impl BufferExt for u64 {
  fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    buf.try_get_u64().ok()
  }

  fn write(&self, buf: &mut bytes::BytesMut) {
    buf.put_u64(*self);
  }
}
