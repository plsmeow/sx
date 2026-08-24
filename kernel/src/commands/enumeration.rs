use salarixi_macros::Index;

#[derive(Debug, Index)]
pub enum ClientCommand {
  SessionChat = 0x00,
  LaunchBots = 0x01,
  StopBots = 0x02,
  Synchronize = 0x03,
  RemoveCaptcha = 0x04,
  QuickAction = 0x05,
  SetGroup = 0x06,
  ChangeModuleState = 0x07,
  FindRadarTarget = 0x08,
  SaveRadarData = 0x09,
  FollowRadarTarget = 0x0A,
  QuickTask = 0x0B,
  ExecuteScript = 0x0C,
  StopScript = 0x0D,
}
