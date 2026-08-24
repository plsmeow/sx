use hashbrown::HashMap;
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use salarixi_macros::Index;

#[derive(Clone)]
pub struct LaunchOptions {
  pub basic: BasicOptions,
  pub accounts: HashMap<String, AccountOptions>,
  pub plugins: PluginOptions,
  pub captcha_bypass: CaptchaBypassOptions,
  pub webhook: WebhookOptions,
}

impl LaunchOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    let basic = BasicOptions::read(buf)?;

    let account_count = u8::read(buf)?;
    let mut accounts = HashMap::with_capacity(account_count as usize);

    for _ in 0..account_count {
      let username = String::read(buf)?;
      let options = AccountOptions::read(buf)?;
      accounts.insert(username, options);
    }

    let plugins = PluginOptions::read(buf)?;
    let captcha_bypass = CaptchaBypassOptions::read(buf)?;
    let webhook = WebhookOptions::read(buf)?;

    Some(Self {
      basic,
      accounts,
      plugins,
      captcha_bypass,
      webhook,
    })
  }
}

#[derive(Clone)]
pub struct BasicOptions {
  pub address: String,
  pub bots_count: u8,
  pub join_delay: u32,
  pub nickname_type: NicknameType,
  pub password_type: PasswordType,
  pub email_type: EmailType,
  pub nickname_template: String,
  pub password_template: String,
  pub register_mode: AuthMode,
  pub register_command: String,
  pub register_template: String,
  pub register_min_delay: u32,
  pub register_max_delay: u32,
  pub register_trigger: String,
  pub login_mode: AuthMode,
  pub login_command: String,
  pub login_template: String,
  pub login_min_delay: u32,
  pub login_max_delay: u32,
  pub login_trigger: String,
  pub rejoin_delay: u32,
  pub monitoring_update_rate: u32,
  pub view_distance: u8,
  pub humanoid_arm: Option<String>,
  pub use_auto_rejoin: bool,
  pub use_auto_register: bool,
  pub use_double_auth: bool,
  pub use_auto_login: bool,
  pub use_auto_respawn: bool,
  pub use_accept_rp: bool,
  pub use_pathfinder: bool,
  pub use_auto_script: bool,
  pub use_proxy: bool,
  pub use_accounts: bool,
  pub use_anti_captcha: bool,
  pub use_webhook: bool,
  pub monitoring_optimization: bool,
  pub proxy_list: Option<String>,
  pub script: Option<String>,
}

impl BasicOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      address: String::read(buf)?,
      bots_count: u8::read(buf)?,
      join_delay: u32::read(buf)?,
      nickname_type: NicknameType::from_index(u8::read(buf)?)?,
      password_type: PasswordType::from_index(u8::read(buf)?)?,
      email_type: EmailType::from_index(u8::read(buf)?)?,
      nickname_template: String::read(buf)?,
      password_template: String::read(buf)?,
      register_mode: AuthMode::from_index(u8::read(buf)?)?,
      register_command: String::read(buf)?,
      register_template: String::read(buf)?,
      register_min_delay: u32::read(buf)?,
      register_max_delay: u32::read(buf)?,
      register_trigger: String::read(buf)?,
      login_mode: AuthMode::from_index(u8::read(buf)?)?,
      login_command: String::read(buf)?,
      login_template: String::read(buf)?,
      login_min_delay: u32::read(buf)?,
      login_max_delay: u32::read(buf)?,
      login_trigger: String::read(buf)?,
      rejoin_delay: u32::read(buf)?,
      monitoring_update_rate: u32::read(buf)?,
      view_distance: u8::read(buf)?,
      humanoid_arm: Option::read(buf)?,
      use_auto_rejoin: bool::read(buf)?,
      use_auto_register: bool::read(buf)?,
      use_double_auth: bool::read(buf)?,
      use_auto_login: bool::read(buf)?,
      use_auto_respawn: bool::read(buf)?,
      use_accept_rp: bool::read(buf)?,
      use_pathfinder: bool::read(buf)?,
      use_auto_script: bool::read(buf)?,
      use_proxy: bool::read(buf)?,
      use_accounts: bool::read(buf)?,
      use_anti_captcha: bool::read(buf)?,
      use_webhook: bool::read(buf)?,
      monitoring_optimization: bool::read(buf)?,
      proxy_list: Option::read(buf)?,
      script: Option::read(buf)?,
    })
  }
}

#[derive(Clone, Index)]
pub enum NicknameType {
  Legit = 0x00,
  Random = 0x01,
  Custom = 0x02,
}

impl NicknameType {
  pub fn to_str(&self) -> &str {
    match self {
      Self::Legit => "legit",
      Self::Random => "random",
      Self::Custom => "custom",
    }
  }
}

#[derive(Clone, Index)]
pub enum PasswordType {
  Without = 0x00,
  Legit = 0x01,
  Random = 0x02,
  Custom = 0x03,
}

impl PasswordType {
  pub fn to_str(&self) -> &str {
    match self {
      Self::Without => "without",
      Self::Legit => "legit",
      Self::Random => "random",
      Self::Custom => "custom",
    }
  }
}

#[derive(Clone, Index)]
pub enum EmailType {
  Without = 0x00,
  Random = 0x01,
}

#[derive(Clone, PartialEq, Index)]
pub enum AuthMode {
  Default = 0x00,
  Trigger = 0x01,
}

#[derive(Clone)]
pub struct AccountOptions {
  pub initial_group: Option<String>,
  pub password: Option<String>,
  pub email: Option<String>,
  pub proxy: Option<String>,
}

impl AccountOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      initial_group: Option::read(buf)?,
      password: Option::read(buf)?,
      email: Option::read(buf)?,
      proxy: Option::read(buf)?,
    })
  }
}

#[derive(Clone)]
pub struct PluginOptions {
  pub instant_armor_equip: bool,
  pub auto_totem: bool,
  pub auto_eat: bool,
  pub potion_consumer: bool,
  pub auto_look: bool,
  pub auto_shield: bool,
  pub auto_mending: bool,
  pub pearl_leave: bool,
}

impl PluginOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      instant_armor_equip: bool::read(buf)?,
      auto_totem: bool::read(buf)?,
      auto_eat: bool::read(buf)?,
      potion_consumer: bool::read(buf)?,
      auto_look: bool::read(buf)?,
      auto_shield: bool::read(buf)?,
      auto_mending: bool::read(buf)?,
      pearl_leave: bool::read(buf)?,
    })
  }
}

#[derive(Clone)]
pub struct CaptchaBypassOptions {
  pub captcha_type: CaptchaType,
  pub captcha_subtype: CaptchaSubtype,
  pub solve_mode: CaptchaSolveMode,
  pub captcha_size: CaptchaSize,
  pub regex: String,
  pub required_url_part: Option<String>,
  pub number_of_columns: u32,
  pub number_of_rows: u32,
  pub max_pause: u32,
  pub user_id: Option<String>,
  pub api_key: Option<String>,
  pub api_service: CaptchaApiService,
}

impl CaptchaBypassOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      captcha_type: CaptchaType::from_index(u8::read(buf)?)?,
      captcha_subtype: CaptchaSubtype::from_index(u8::read(buf)?)?,
      solve_mode: CaptchaSolveMode::from_index(u8::read(buf)?)?,
      captcha_size: CaptchaSize::from_index(u8::read(buf)?)?,
      regex: String::read(buf)?,
      required_url_part: Option::read(buf)?,
      number_of_columns: u32::read(buf)?,
      number_of_rows: u32::read(buf)?,
      max_pause: u32::read(buf)?,
      user_id: Option::read(buf)?,
      api_key: Option::read(buf)?,
      api_service: CaptchaApiService::from_index(u8::read(buf)?)?,
    })
  }
}

#[derive(Clone, PartialEq, Index)]
pub enum CaptchaType {
  Web = 0x00,
  Map = 0x01,
}

#[derive(Clone, PartialEq, Index)]
pub enum CaptchaSubtype {
  Inventory = 0x00,
  Frame = 0x01,
}

#[derive(Clone, PartialEq, Index)]
pub enum CaptchaSolveMode {
  Manual = 0x00,
  Auto = 0x01,
}

#[derive(Clone, PartialEq, Index)]
pub enum CaptchaSize {
  Fixed = 0x00,
  Dynamic = 0x01,
}

#[derive(Clone, PartialEq, Index)]
pub enum CaptchaApiService {
  TwoCaptcha = 0x00,
  TrueCaptcha = 0x01,
}

#[derive(Clone)]
pub struct WebhookOptions {
  pub url: Option<String>,
  pub send_information: bool,
  pub send_data: bool,
  pub send_actions: bool,
}

impl WebhookOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      url: Option::read(buf)?,
      send_information: bool::read(buf)?,
      send_data: bool::read(buf)?,
      send_actions: bool::read(buf)?,
    })
  }
}
