use std::fs::OpenOptions;
use std::io::Write;

use azalea::player::GameProfileComponent;
use chrono::prelude::*;
use once_cell::sync::Lazy;

use crate::bot::extensions::{go_to, BotDefaultExt};
use crate::bot::systems::profile::PROFILE_SYSTEM;
use crate::take_bot;

pub static RADAR: Lazy<Radar> = Lazy::new(|| Radar::new());

pub struct RadarInfo {
  pub uuid: String,
  pub tx: f64,
  pub ty: f64,
  pub tz: f64,
  pub ox: f64,
  pub oz: f64,
}

pub struct Radar;

impl Radar {
  pub fn new() -> Self {
    Self
  }

  pub async fn find_target(&self, target: String) -> Option<RadarInfo> {
    let mut info = None;

    for index in PROFILE_SYSTEM.get_all_connected().await.keys() {
      take_bot!(index, async |bot| {
        let Some(tab_list) = bot.get_players() else {
          return;
        };

        for uuid in tab_list.keys() {
          let Some(entity) = bot.entity_by_uuid(*uuid) else {
            continue;
          };

          let Some(profile) = bot.get_entity_component::<GameProfileComponent>(entity) else {
            continue;
          };

          if profile.0.name != target {
            continue;
          }

          let player_pos = bot.get_entity_position(entity);
          let Some(client_pos) = bot.foot_pos() else {
            return;
          };

          info = Some(RadarInfo {
            uuid: uuid.to_string(),
            tx: player_pos.x,
            ty: player_pos.y,
            tz: player_pos.z,
            ox: client_pos.x,
            oz: client_pos.z,
          });
        }
      });
    }

    info
  }

  pub fn save_data(&self, target: String, mut path: String, filename: String, x: f64, y: f64, z: f64) {
    let date = Local::now().format("%H:%M:%S").to_string();
    let content = format!("[ {} ] {} ~ X: {}, Y: {}, Z: {}", date, target, x, y, z);

    path.push_str(&filename.replace("#t", target.as_str()));
    path.push_str(".txt");

    let mut file = OpenOptions::new().create(true).append(true).open(&path).unwrap();

    writeln!(&mut file, "{}", content).unwrap();
  }

  pub async fn follow(&self, x: i32, z: i32) {
    for (index, _) in PROFILE_SYSTEM.get_all_connected().await.iter() {
      go_to(*index, x, z);
    }
  }
}
