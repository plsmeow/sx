use bytes::{Buf, BufMut};

use crate::buffer::BufferExt;

impl BufferExt for String {
  fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    // Долго над этим не думал, пусть будет u16, маловероятно
    // что кому-то не хватит на одну строку 65535 символа
    let len = buf.try_get_u16().ok()?;
    let mut arr = Vec::with_capacity(len.into());

    for _ in 0..len {
      let byte = buf.try_get_u8().ok()?;
      arr.push(byte);
    }

    let string = String::from_utf8(arr).ok()?;

    Some(string)
  }

  fn write(&self, buf: &mut bytes::BytesMut) {
    let len = self.len();
    buf.put_u16(len as u16);
    buf.put_slice(self.as_bytes());
  }
}
