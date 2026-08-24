use crate::bot::systems::index::INDEX_SYSTEM;
use crate::launch::options::{EmailType, NicknameType};
use crate::tools::*;

const LEGIT_NICKNAMES: &[&str] = &[
  "Apple",
  "Boss",
  "Crazy",
  "Dancer",
  "Eagle",
  "Flyer",
  "Game",
  "Hunter",
  "Insider",
  "Joker",
  "King",
  "Lancer",
  "Master",
  "Ninja",
  "Ocean",
  "Phoenix",
  "Queen",
  "Ranger",
  "Savage",
  "Tornado",
  "Ultra",
  "Viking",
  "Warrior",
  "Xenon",
  "Yoda",
  "Zodiac",
  "Ace",
  "Bold",
  "Clever",
  "Dreams",
  "Salarixi",
  "Epic",
  "Frost",
  "Gigant",
  "Hero",
  "Iceberg",
  "Jester",
  "Knight",
  "Lethal",
  "Meteor",
  "Nova",
  "Protector",
  "Orange",
  "Pioneer",
  "Quasar",
  "Rampage",
  "Solar",
  "Titan",
  "Unstoppable",
  "Vandal",
  "Wizard",
  "Xray",
  "Yellow",
  "Zeus",
  "Awesome",
  "Brilliant",
  "Courage",
  "Dynamite",
  "Elegant",
  "Fearless",
  "Glory",
  "Hysteria",
  "Alpha",
  "Bionic",
  "Cosmic",
  "Dominator",
  "Electric",
  "Fury",
  "Gigabyte",
  "Heroic",
  "Ice",
  "Jolt",
  "Killer",
  "Laser",
  "Meteorite",
  "Nemesis",
  "Oceanic",
  "Paradox",
  "Quake",
  "Rampart",
  "Specter",
  "Thunder",
  "Ultimate",
  "Vapor",
  "Wizardry",
  "Xenonix",
  "Yellowstone",
  "Zigzag",
  "Apex",
  "Blitz",
  "Catalyst",
  "Dynamo",
  "Eon",
  "Flux",
  "Ghost",
  "Hawk",
  "Infinity",
  "Jolted",
  "Kaleidoscope",
  "Lumina",
  "Maelstrom",
  "Nebel",
  "Onyx",
  "Pulsar",
  "Quixote",
  "Renegade",
  "Skybolt",
  "Tornado",
  "Umbra",
  "Vagabond",
  "Wildfire",
  "Xylophone",
];

const LEGIT_PASSWORDS: &[&str] = &[
  "password",
  "invisible",
  "ytrewq",
  "mypass",
  "drowssap",
  "asdfghjkl",
  "stealth",
  "unreadable",
  "possible",
];

/// Функция генерации юзернейма или пароля
pub fn generate_username_or_password(item: &str, ty: &str, template: &str) -> Option<String> {
  match ty {
    "legit" => {
      let mut value = randelem(if item == "username" {
        LEGIT_NICKNAMES
      } else {
        LEGIT_PASSWORDS
      })
      .to_string();

      value = value
        .chars()
        .map(|c| {
          if randchance(0.25) {
            match c.to_ascii_lowercase() {
              'o' => '0',
              'a' => '4',
              'z' => '3',
              'e' => '3',
              'i' => '1',
              'l' => '1',
              'p' => '5',
              'v' => '8',
              'b' => '6',
              _ => c,
            }
          } else {
            c
          }
        })
        .collect();

      value = value
        .chars()
        .map(|c| {
          if randchance(0.4) && c.is_ascii_alphabetic() {
            c.to_ascii_uppercase()
          } else {
            c
          }
        })
        .collect();

      if value.len() <= 13 {
        let suffix_len = if value.len() <= 6 {
          randnum(4, 6) as usize
        } else {
          randnum(2, 3) as usize
        };

        let suffix = randstr(CharClass::Multi, suffix_len as i32);
        let separator = if randchance(0.5) { "_" } else { "" };

        value = format!("{}{}{}", value, separator, suffix);
      }

      Some(value)
    }
    "random" => {
      let templates = vec![
        "#m#m#m#m", "#n#n#n#n", "#l#l#l#l", "#m#l#n#m", "#l#m#n#n", "#m#m", "#n#n", "#m#n#l",
      ];
      let chosen_template = randelem(&templates);

      Some(mutate_text(chosen_template.to_string()))
    }
    "custom" => Some(mutate_text(template.to_string())),
    _ => None,
  }
}

/// Функция генерации электронной почты
pub fn generate_email(ty: &EmailType) -> Option<String> {
  match ty {
    EmailType::Random => {
      let templates = vec![
        "#m#m#m#m",
        "#n#n#n#n",
        "#l#l#l#l",
        "#m#l#n#m",
        "#l#m#n#n",
        "#m#m",
        "#n#n",
        "#m#n#l",
        "#n#n#m#n",
        "#m#m#m#n#m",
        "#m#n#m#n#l",
      ];

      let services = vec!["gmail.com", "proton.me", "yandex.ru", "mail.ru"];

      let chosen_template = randelem(&templates);
      let chosen_service = randelem(&services);

      Some(format!(
        "{}@{}",
        mutate_text(chosen_template.to_string()),
        chosen_service
      ))
    }
    EmailType::Without => None,
  }
}

/// Функция генерации уникального юзернейма
pub async fn generate_unique_username(ty: &NicknameType, template: &str) -> Option<String> {
  let usernames = INDEX_SYSTEM.get_all_usernames().await;

  for _ in 0..5 {
    let Some(username) = generate_username_or_password("username", ty.to_str(), template) else {
      continue;
    };

    if !usernames.contains(&username) {
      return Some(username);
    }
  }

  None
}
