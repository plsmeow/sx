use std::cmp::Ordering;
use std::f64::consts::PI;
use std::time::{Duration, Instant};

use azalea::entity::LookDirection;
use azalea::protocol::packets::game::{ClientboundAddEntity, ClientboundMapItemData};
use azalea::registry::builtin::EntityKind;
use azalea::Client;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use hashbrown::HashMap;
use image::{ImageBuffer, ImageFormat, Rgb};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{json, Value};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::bot::extensions::BotDefaultExt;
use crate::bot::systems::profile::PROFILE_SYSTEM;
use crate::launch::options::{
  CaptchaApiService, CaptchaBypassOptions, CaptchaSize, CaptchaSolveMode, CaptchaSubtype, CaptchaType,
};
use crate::launch::process::current_options;
use crate::server::transfer::{emit_log, AntiMapCaptchaPayload, TransferEvent, TRANSFER};
use crate::webhook::send_webhook;
use crate::{sleep, take_bot, take_profile};

pub static WEB_CAPTCHA_BYPASS: Lazy<WebCaptchaBypass> = Lazy::new(|| WebCaptchaBypass::new());
pub static MAP_CAPTCHA_BYPASS: Lazy<MapCaptchaBypass> = Lazy::new(|| MapCaptchaBypass::new());

pub struct WebCaptchaBypass;

impl WebCaptchaBypass {
  pub fn new() -> Self {
    Self
  }

  /// Метод поимки URL капчи из сообщения
  pub fn catch_url_from_message(
    &self,
    message: String,
    regex: &str,
    required_url_part: Option<String>,
  ) -> Option<String> {
    let re = Regex::new(regex).unwrap();

    for link_to_captcha in re.find_iter(&message) {
      if !link_to_captcha.is_empty() {
        if let Some(required) = required_url_part.clone() {
          if link_to_captcha.as_str().contains(required.as_str()) {
            return Some(link_to_captcha.as_str().to_string());
          }
        } else {
          return Some(link_to_captcha.as_str().to_string());
        }
      }
    }

    None
  }
}

pub struct MapCaptchaBypass {
  frame_collector: FrameCollector,
  solver_tasks: RwLock<HashMap<u8, JoinHandle<()>>>,
}

impl MapCaptchaBypass {
  pub fn new() -> Self {
    Self {
      frame_collector: FrameCollector::new(),
      solver_tasks: RwLock::new(HashMap::new()),
    }
  }

  /// Метод конвертации ID цвета Minecraft в RGB код
  fn convert_id_to_rgb_color(&self, id: u8) -> (u8, u8, u8) {
    match id {
      0 => (255, 255, 255),
      1 => (255, 255, 255),
      2 => (255, 255, 255),
      3 => (255, 255, 255),
      4 => (89, 125, 39),
      5 => (109, 153, 48),
      6 => (127, 178, 56),
      7 => (67, 94, 29),
      8 => (174, 164, 115),
      9 => (213, 201, 140),
      10 => (247, 233, 163),
      11 => (130, 123, 86),
      12 => (140, 140, 140),
      13 => (171, 171, 171),
      14 => (199, 199, 199),
      15 => (105, 105, 105),
      16 => (180, 0, 0),
      17 => (220, 0, 0),
      18 => (255, 0, 0),
      19 => (135, 0, 0),
      20 => (112, 112, 180),
      21 => (138, 138, 220),
      22 => (160, 160, 255),
      23 => (84, 84, 135),
      24 => (117, 117, 117),
      25 => (144, 144, 144),
      26 => (167, 167, 167),
      27 => (88, 88, 88),
      28 => (0, 87, 0),
      29 => (0, 106, 0),
      30 => (0, 124, 0),
      31 => (0, 65, 0),
      32 => (180, 180, 180),
      33 => (220, 220, 220),
      34 => (255, 255, 255),
      35 => (135, 135, 135),
      36 => (115, 118, 129),
      37 => (141, 144, 158),
      38 => (164, 168, 184),
      39 => (86, 88, 97),
      40 => (106, 76, 54),
      41 => (130, 94, 66),
      42 => (151, 109, 77),
      43 => (79, 57, 40),
      44 => (79, 79, 79),
      45 => (96, 96, 96),
      46 => (112, 112, 112),
      47 => (59, 59, 59),
      48 => (45, 45, 180),
      49 => (55, 55, 220),
      50 => (64, 64, 255),
      51 => (33, 33, 135),
      52 => (100, 84, 50),
      53 => (123, 102, 62),
      54 => (143, 119, 72),
      55 => (75, 63, 38),
      56 => (180, 177, 172),
      57 => (220, 217, 211),
      58 => (255, 252, 245),
      59 => (135, 133, 129),
      60 => (152, 89, 36),
      61 => (186, 109, 44),
      62 => (216, 127, 51),
      63 => (114, 67, 27),
      64 => (125, 53, 152),
      65 => (153, 65, 186),
      66 => (178, 76, 216),
      67 => (94, 40, 114),
      68 => (72, 108, 152),
      69 => (88, 132, 186),
      70 => (102, 153, 216),
      71 => (54, 81, 114),
      72 => (161, 161, 36),
      73 => (197, 197, 44),
      74 => (229, 229, 51),
      75 => (121, 121, 27),
      76 => (89, 144, 17),
      77 => (109, 176, 21),
      78 => (127, 204, 25),
      79 => (67, 108, 13),
      80 => (170, 89, 116),
      81 => (208, 109, 142),
      82 => (242, 127, 165),
      83 => (128, 67, 87),
      84 => (53, 53, 53),
      85 => (65, 65, 65),
      86 => (76, 76, 76),
      87 => (40, 40, 40),
      88 => (108, 108, 108),
      89 => (132, 132, 132),
      90 => (153, 153, 153),
      91 => (81, 81, 81),
      92 => (53, 89, 108),
      93 => (65, 109, 132),
      94 => (76, 127, 153),
      95 => (40, 67, 81),
      96 => (89, 44, 125),
      97 => (109, 54, 153),
      98 => (127, 63, 178),
      99 => (67, 33, 94),
      100 => (36, 53, 125),
      101 => (44, 65, 153),
      102 => (51, 76, 178),
      103 => (27, 40, 94),
      104 => (72, 53, 36),
      105 => (88, 65, 44),
      106 => (102, 76, 51),
      107 => (54, 40, 27),
      108 => (72, 89, 36),
      109 => (88, 109, 44),
      110 => (102, 127, 51),
      111 => (54, 67, 27),
      112 => (108, 36, 36),
      113 => (132, 44, 44),
      114 => (153, 51, 51),
      115 => (81, 27, 27),
      116 => (17, 17, 17),
      117 => (21, 21, 21),
      118 => (25, 25, 25),
      119 => (13, 13, 13),
      120 => (176, 168, 54),
      121 => (215, 205, 66),
      122 => (250, 238, 77),
      123 => (132, 126, 40),
      124 => (64, 154, 150),
      125 => (79, 188, 183),
      126 => (92, 219, 213),
      127 => (48, 115, 112),
      128 => (52, 90, 180),
      129 => (63, 110, 220),
      130 => (74, 128, 255),
      131 => (39, 67, 135),
      132 => (0, 153, 40),
      133 => (0, 187, 50),
      134 => (0, 217, 58),
      135 => (0, 114, 30),
      136 => (91, 60, 34),
      137 => (111, 74, 42),
      138 => (129, 86, 49),
      139 => (68, 45, 25),
      140 => (79, 1, 0),
      141 => (96, 1, 0),
      142 => (112, 2, 0),
      143 => (59, 1, 0),
      144 => (147, 124, 113),
      145 => (180, 152, 138),
      146 => (209, 177, 161),
      147 => (110, 93, 85),
      148 => (112, 57, 25),
      149 => (137, 70, 31),
      150 => (159, 82, 36),
      151 => (84, 43, 19),
      152 => (105, 61, 76),
      153 => (128, 75, 93),
      154 => (149, 87, 108),
      155 => (78, 46, 57),
      156 => (79, 76, 97),
      157 => (96, 93, 119),
      158 => (112, 108, 138),
      159 => (59, 57, 73),
      160 => (131, 93, 25),
      161 => (160, 114, 31),
      162 => (186, 133, 36),
      163 => (98, 70, 19),
      164 => (72, 82, 37),
      165 => (88, 100, 45),
      166 => (103, 117, 53),
      167 => (54, 61, 28),
      168 => (112, 54, 55),
      169 => (138, 66, 67),
      170 => (160, 77, 78),
      171 => (84, 40, 41),
      172 => (40, 28, 24),
      173 => (49, 35, 30),
      174 => (57, 41, 35),
      175 => (30, 21, 18),
      176 => (95, 75, 69),
      177 => (116, 92, 84),
      178 => (135, 107, 98),
      179 => (71, 56, 51),
      180 => (61, 64, 64),
      181 => (75, 79, 79),
      182 => (87, 92, 92),
      183 => (46, 48, 48),
      184 => (86, 51, 62),
      185 => (105, 62, 75),
      186 => (122, 73, 88),
      187 => (64, 38, 46),
      188 => (53, 43, 64),
      189 => (65, 53, 79),
      190 => (76, 62, 92),
      191 => (40, 32, 48),
      192 => (53, 35, 24),
      193 => (65, 43, 30),
      194 => (76, 50, 35),
      195 => (40, 26, 18),
      196 => (53, 57, 29),
      197 => (65, 70, 36),
      198 => (76, 82, 42),
      199 => (40, 43, 22),
      200 => (100, 42, 32),
      201 => (122, 51, 39),
      202 => (142, 60, 46),
      203 => (75, 31, 24),
      204 => (26, 15, 11),
      205 => (31, 18, 13),
      206 => (37, 22, 16),
      207 => (19, 11, 8),
      _ => (255, 255, 255),
    }
  }

  /// Метод создания PNG картинки
  pub fn create_png_image(&self, width: u32, height: u32, map: &Vec<u8>) -> String {
    let mut img = ImageBuffer::new(width, height);

    for (i, &id) in map.iter().enumerate() {
      let rgb = self.convert_id_to_rgb_color(id);
      let x = i as u32 % width;
      let y = i as u32 / width;

      img.put_pixel(x, y, Rgb([rgb.0, rgb.1, rgb.2]));
    }

    let mut bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut bytes);

    let _ = img.write_to(&mut cursor, ImageFormat::Png);

    let base64_code = BASE64_STANDARD.encode(&bytes);

    base64_code
  }

  /// Метод автоматического решения капчи
  pub async fn solve_captcha(&self, index: u8, username: String, b64: String, options: CaptchaBypassOptions) {
    let task = tokio::spawn(async move {
      let api_key = match options.api_key {
        Some(key) => key,
        None => {
          emit_log(
            format!("Бот {} не смог решить капчу: API key not specified", username),
            "error",
          );
          return;
        }
      };

      match options.api_service {
        CaptchaApiService::TwoCaptcha => Self::solve_with_two_captcha(&index, username, b64, api_key).await,
        CaptchaApiService::TrueCaptcha => {
          let user_id = match options.user_id {
            Some(key) => key,
            None => {
              emit_log(
                format!("Бот {} не смог решить капчу: user ID not specified", username),
                "error",
              );
              return;
            }
          };

          Self::solve_with_true_captcha(&index, username, b64, user_id, api_key).await;
        }
      }
    });

    self.solver_tasks.write().await.insert(index, task);
  }

  /// Метод решения капчи при помощи сервиса `2captcha.com`
  async fn solve_with_two_captcha(index: &u8, username: String, b64: String, api_key: String) {
    let client = reqwest::Client::new();
    let create_url = "https://api.2captcha.com/createTask";
    let result_url = "https://api.2captcha.com/getTaskResult";

    let req_body = json!({
      "clientKey": api_key,
      "task": {
        "type": "ImageToTextTask",
        "body": b64,
        "phrase": false,
        "case": true,
        "numeric": 0,
        "math": false,
        "minLength": 0,
        "maxLength": 0,
      },
      "languagePool": "en"
    });

    let resp = match client.post(create_url).json(&req_body).send().await {
      Ok(r) => r,
      Err(e) => {
        emit_log(format!("Бот {} не смог решить капчу: {}", username, e), "error");
        return;
      }
    };

    let create_resp: serde_json::Value = match resp.json().await {
      Ok(v) => v,
      Err(e) => {
        emit_log(format!("Бот {} не смог решить капчу: {}", username, e), "error");
        return;
      }
    };

    if let Some(err_id) = create_resp["errorId"].as_i64() {
      if err_id != 0 {
        let err_desc = create_resp["errorDescription"].as_str().unwrap_or("unknown error");
        emit_log(format!("Бот {} не смог решить капчу: {}", username, err_desc), "error");
        return;
      }
    } else {
      emit_log(
        format!("Бот {} не смог решить капчу: incorrect response format", username),
        "error",
      );
      return;
    }

    let task_id = match create_resp["taskId"].as_str() {
      Some(id) => id.to_string(),
      None => {
        emit_log(
          format!("Бот {} не смог решить капчу: task ID is missing in response", username),
          "error",
        );
        return;
      }
    };

    emit_log(
      format!("Задача решения капчи бота {} создана, ожидание завершения...", username),
      "extended",
    );

    let mut attempts = 0;
    let max_attempts = 6;

    loop {
      if attempts >= max_attempts {
        emit_log(
          format!("Бот {} не смог решить капчу: timeout exceeded", username),
          "error",
        );
        return;
      }

      tokio::time::sleep(Duration::from_secs(5)).await;
      attempts += 1;

      let result_req_body = json!({
        "clientKey": api_key,
        "taskId": task_id
      });

      let res_resp = match client.post(result_url).json(&result_req_body).send().await {
        Ok(r) => r,
        Err(e) => {
          emit_log(format!("Бот {} не смог решить капчу: {}", username, e), "error");
          return;
        }
      };

      let res_data: serde_json::Value = match res_resp.json().await {
        Ok(v) => v,
        Err(e) => {
          emit_log(format!("Бот {} не смог решить капчу: {}", username, e), "error");
          return;
        }
      };

      if let Some(err_id) = res_data["errorId"].as_i64() {
        if err_id != 0 {
          let err_desc = res_data["errorDescription"].as_str().unwrap_or("unknown error");
          emit_log(format!("Бот {} не смог решить капчу: {}", username, err_desc), "error");
          return;
        }
      }

      if let Some(status) = res_data["status"].as_str() {
        if status == "ready" {
          if let Some(result) = res_data["solution"]["text"].as_str() {
            take_bot!(index, async |bot| {
              bot.chat(result);
              emit_log(format!("Бот {} решил капчу (текст: {})", username, result), "info");
            });
            return;
          } else {
            emit_log(
              format!("Бот {} не смог решить капчу: incorrect response format", username),
              "error",
            );
            return;
          }
        } else if status == "processing" {
          continue;
        } else {
          emit_log(
            format!("Бот {} не смог решить капчу: unknown status \"{}\"", username, status),
            "error",
          );
          return;
        }
      } else {
        emit_log(
          format!("Бот {} не смог решить капчу: status field is missing", username),
          "error",
        );
        return;
      }
    }
  }

  /// Метод решения капчи при помощи сервиса `truecaptcha.org`
  async fn solve_with_true_captcha(index: &u8, username: String, b64: String, user_id: String, api_key: String) {
    let client = reqwest::Client::new();
    let target_url = "https://api.apitruecaptcha.org/one/gettext";

    let req_body = json!({
      "userid": user_id,
      "apikey": api_key,
      "data": b64,
    });

    let resp = match client.post(target_url).json(&req_body).send().await {
      Ok(resp) => resp,
      Err(e) => {
        emit_log(format!("Бот {} не смог решить капчу: {}", username, e), "error");
        return;
      }
    };

    if !resp.status().is_success() {
      emit_log(
        format!("Бот {} не смог решить капчу: status code {}", username, resp.status()),
        "error",
      );
      return;
    }

    let json: Value = match resp.json().await {
      Ok(v) => v,
      Err(e) => {
        emit_log(
          format!("Бот {} не смог решить капчу: status code {}", username, e),
          "error",
        );
        return;
      }
    };

    let Some(result) = json["result"].as_str() else {
      emit_log(
        format!("Бот {} не смог решить капчу: result missing", username),
        "error",
      );
      return;
    };

    take_bot!(index, async |bot| {
      bot.chat(result);
      emit_log(format!("Бот {} решил капчу (текст: {})", username, result), "info");
    });
  }

  /// Метод обработки рамки
  pub async fn process_frame(&self, index: u8, packet: &ClientboundAddEntity) {
    if packet.entity_type == EntityKind::ItemFrame || packet.entity_type == EntityKind::GlowItemFrame {
      let pos = packet.position;
      self
        .frame_collector
        .push_frame(index, packet.id.0 as i32, pos.x, pos.y, pos.z)
        .await;
    }
  }

  /// Метод обработки данных карты из рамки
  pub async fn process_map_data(&self, bot: &Client, username: String, index: u8, packet: &ClientboundMapItemData) {
    let Some(options) = current_options().await else {
      return;
    };

    if !options.basic.use_anti_captcha || options.captcha_bypass.captcha_type != CaptchaType::Map {
      return;
    }

    let Some(map_patch) = &packet.color_patch.0 else {
      return;
    };

    let Some(profile) = PROFILE_SYSTEM.get(&index).await else {
      return;
    };

    if profile.captcha_caught {
      return;
    }

    if options.captcha_bypass.captcha_subtype == CaptchaSubtype::Frame {
      let Some(foot_pos) = bot.foot_pos() else {
        return;
      };

      let yaw = if let Some(look_dir) = bot.get_component::<LookDirection>() {
        look_dir.y_rot()
      } else {
        0.0
      };

      self
        .frame_collector
        .push_map_data(
          index,
          map_patch.width as u32,
          map_patch.height as u32,
          map_patch.map_colors.clone(),
          foot_pos.x,
          foot_pos.z,
          yaw,
        )
        .await;

      self.frame_collector.update_last_frame_time(index).await;

      if let Some(b64) = self.frame_collector.try_combine_all(&index, false).await {
        take_profile!(&index, async |profile| {
          profile.captcha_caught = true;
        });

        if options.basic.use_webhook && options.webhook.send_information {
          send_webhook(options.webhook.url, format!("Бот {} получил капчу с рамок", username));
        }

        emit_log(format!("Бот {} получил капчу с рамок", username), "info");

        if options.captcha_bypass.solve_mode == CaptchaSolveMode::Auto {
          self
            .solve_captcha(index, username.clone(), b64, options.captcha_bypass)
            .await;
        } else {
          TRANSFER.emit(TransferEvent::AntiMapCaptcha(AntiMapCaptchaPayload {
            b64,
            username: username.to_string(),
          }));
        }

        self.frame_collector.clear(&index).await;
      }
    } else {
      let b64 = self.create_png_image(map_patch.width as u32, map_patch.height as u32, &map_patch.map_colors);

      take_profile!(&index, async |profile| {
        profile.captcha_caught = true;
      });

      if options.basic.use_webhook && options.webhook.send_information {
        send_webhook(options.webhook.url, format!("Бот {} получил капчу с карты", username));
      }

      emit_log(format!("Бот {} получил капчу с карты", username), "info");

      if options.captcha_bypass.solve_mode == CaptchaSolveMode::Auto {
        self.solve_captcha(index, username, b64, options.captcha_bypass).await;
      } else {
        TRANSFER.emit(TransferEvent::AntiMapCaptcha(AntiMapCaptchaPayload {
          b64,
          username: username.to_string(),
        }));
      }
    }
  }

  /// Метод выключения и очистки утилиты
  pub async fn shutdown(&self) {
    let mut tasks_guard = self.solver_tasks.write().await;
    tasks_guard.iter().for_each(|(_, t)| t.abort());
    sleep!(500);
    tasks_guard.clear();
    drop(tasks_guard);

    self.frame_collector.clear_all().await;
  }
}

#[derive(Clone)]
struct MapData {
  pub width: u32,
  pub height: u32,
  pub colors: Vec<u8>,
  pub x: f64,
  pub z: f64,
  pub pos_x: f64,
  pub pos_z: f64,
  pub yaw: f32,
}

struct FrameCollector {
  maps: RwLock<HashMap<u8, Vec<MapData>>>,
  frame_positions: RwLock<HashMap<u8, Vec<(i32, f64, f64, f64)>>>,
  last_frame_time: RwLock<HashMap<u8, Instant>>,
  waiters: RwLock<HashMap<u8, ()>>,
}

impl FrameCollector {
  pub fn new() -> Self {
    Self {
      maps: RwLock::new(HashMap::new()),
      frame_positions: RwLock::new(HashMap::new()),
      last_frame_time: RwLock::new(HashMap::new()),
      waiters: RwLock::new(HashMap::new()),
    }
  }

  /// Метод добавления рамки
  pub async fn push_frame(&self, index: u8, entity_id: i32, x: f64, y: f64, z: f64) {
    let mut positions = self.frame_positions.write().await;
    let user_positions = positions.entry(index).or_insert_with(Vec::new);
    user_positions.push((entity_id, x, y, z));
  }

  /// Метод добавления данных карты из рамки
  pub async fn push_map_data(
    &self,
    index: u8,
    width: u32,
    height: u32,
    colors: Vec<u8>,
    pos_x: f64,
    pos_z: f64,
    yaw: f32,
  ) {
    let mut maps = self.maps.write().await;
    let user_maps = maps.entry(index).or_insert_with(Vec::new);

    user_maps.push(MapData {
      width,
      height,
      colors,
      x: 0.0,
      z: 0.0,
      pos_x,
      pos_z,
      yaw,
    });
  }

  /// Метод получения карт указанного бота
  pub async fn get_maps(&self, index: &u8) -> Option<Vec<MapData>> {
    let maps = self.maps.read().await;
    maps.get(index).cloned()
  }

  /// Метод обновления последнего времени прихода фрейма указанного бота
  pub async fn update_last_frame_time(&self, index: u8) {
    let mut last_time = self.last_frame_time.write().await;
    last_time.insert(index, Instant::now());
  }

  /// Метод получения последнего времени прихода фрейма указанного бота
  pub async fn get_last_captcha_time(&self, index: &u8) -> Option<Instant> {
    self.last_frame_time.write().await.get(index).cloned()
  }

  /// Метод попытки комбинации всех карт и созданий base64-изображения
  pub async fn try_combine_all(&self, index: &u8, force: bool) -> Option<String> {
    let mut map_data = self.get_maps(index).await?;

    if map_data.is_empty() {
      return None;
    }

    let Some(opts) = current_options().await else {
      return None;
    };

    let frame_count;

    if opts.captcha_bypass.captcha_size == CaptchaSize::Fixed {
      frame_count = (opts.captcha_bypass.number_of_columns * opts.captcha_bypass.number_of_rows) as usize;

      if !force && map_data.len() < frame_count {
        return None;
      }
    } else {
      let Some(last_frame_time) = self.get_last_captcha_time(index).await else {
        return None;
      };

      let current_time = Instant::now();
      let passed = current_time.duration_since(last_frame_time).as_millis();

      if !force && passed < opts.captcha_bypass.max_pause as u128 {
        if !self.waiters.write().await.contains_key(index) {
          let time_left = opts.captcha_bypass.max_pause as u64 - passed as u64;
          self.waiters.write().await.insert(*index, ());
          sleep!(time_left);
          self.waiters.write().await.remove(index);
          map_data = self.get_maps(index).await?;
        } else {
          return None;
        }
      }

      frame_count = map_data.len();
    }

    let first_map = &map_data[0];
    let yaw = first_map.yaw;
    let pos_x = first_map.pos_x;
    let pos_z = first_map.pos_z;

    let yaw_rad = (yaw as f64).to_radians();

    map_data.retain(|map| {
      let dx = map.x - pos_x;
      let dz = map.z - pos_z;
      let distance = (dx * dx + dz * dz).sqrt();

      let angle_to_frame = dz.atan2(dx);
      let angle_diff = (angle_to_frame - yaw_rad).abs();

      let normalized_angle = if angle_diff > PI {
        2.0 * PI - angle_diff
      } else {
        angle_diff
      };

      distance < 20.0 && normalized_angle < PI / 2.0
    });

    if map_data.len() < frame_count {
      let all_maps = self.get_maps(index).await?;
      map_data = all_maps;
    }

    map_data.truncate(frame_count);

    let positions = self.frame_positions.read().await;
    if let Some(user_positions) = positions.get(index) {
      if user_positions.len() >= frame_count {
        let mut sorted_positions = user_positions.clone();

        sorted_positions.sort_by(|a, b| match b.2.partial_cmp(&a.2).unwrap_or(Ordering::Equal) {
          Ordering::Equal => a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal),
          other => other,
        });

        let mut sorted_maps = vec![];

        for _ in sorted_positions.iter().take(frame_count) {
          if let Some(map) = map_data.pop() {
            sorted_maps.push(map);
          }
        }

        map_data = sorted_maps;
      }
    }

    let frames: Vec<_> = map_data.iter().take(frame_count).collect();

    let frame_width = frames[0].width;
    let frame_height = frames[0].height;

    let cols = opts.captcha_bypass.number_of_columns;
    let rows = opts.captcha_bypass.number_of_rows;

    let total_width = frame_width * cols;
    let total_height = frame_height * rows;

    let mut combined_img = ImageBuffer::new(total_width, total_height);

    for (idx, frame) in frames.iter().enumerate() {
      let col = (idx as u32) % cols;
      let row = (idx as u32) / cols;

      let x_offset = col * frame_width;
      let y_offset = row * frame_height;

      for (i, &id) in frame.colors.iter().enumerate() {
        let rgb = MAP_CAPTCHA_BYPASS.convert_id_to_rgb_color(id);
        let local_x = (i as u32) % frame.width;
        let local_y = (i as u32) / frame.width;

        let final_x = x_offset + local_x;
        let final_y = y_offset + local_y;

        if final_x < total_width && final_y < total_height {
          combined_img.put_pixel(final_x, final_y, Rgb([rgb.0, rgb.1, rgb.2]));
        }
      }
    }

    let mut bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut bytes);

    let _ = combined_img.write_to(&mut cursor, ImageFormat::Png);

    Some(BASE64_STANDARD.encode(&bytes))
  }

  /// Метод очистки данных указанного бота
  pub async fn clear(&self, index: &u8) {
    self.maps.write().await.remove(index);
    self.frame_positions.write().await.remove(index);
    self.last_frame_time.write().await.remove(index);
    self.waiters.write().await.remove(index);
  }

  /// Метод очистки всего хранилища
  pub async fn clear_all(&self) {
    self.maps.write().await.clear();
    self.frame_positions.write().await.clear();
    self.last_frame_time.write().await.clear();
    self.waiters.write().await.clear();
  }
}
