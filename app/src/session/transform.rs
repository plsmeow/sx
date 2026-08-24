pub fn transform_id(id: u8) -> &'static str {
  match id {
    0x00 => "system:log",
    0x01 => "system:message",
    0x02 => "session:chat",
    0x03 => "process:display-status",
    0x04 => "monitoring:chat",
    0x05 => "captcha:new-web",
    0x06 => "captcha:new-map",
    0x07 => "captcha:remove",
    0x08 => "monitoring:update-profile",
    0x09 => "status:launch",
    0x0A => "status:stop",
    0x0B => "radar:target-info",
    0x0C => "process:synchronize",
    _ => "unknown",
  }
}
