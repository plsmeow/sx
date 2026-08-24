use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use hashbrown::HashSet;
use serde::Deserialize;
use serde_json::Value;
use socks5_impl::protocol::{Address, UserKey};
use tokio::net::TcpStream;
use tokio::sync::RwLock;

const MAX_PROXY_COUNT: usize = 50000;

const TRUSTED_SERVICES: [(&'static str, &'static str); 11] = [
  ("json:proxyscrape", "https://api.proxyscrape.com/v4/free-proxy-list/get?request=display_proxies&proxy_format=protocolipport&format=json"),
  ("json:proxifly", "https://cdn.jsdelivr.net/gh/proxifly/free-proxy-list@main/proxies/all/data.json"),
  ("json:jetkai-proxy-list", "https://raw.githubusercontent.com/jetkai/proxy-list/main/online-proxies/json/proxies-advanced.json"),
  ("json:monosans-proxy-list", "https://raw.githubusercontent.com/monosans/proxy-list/main/proxies_pretty.json"),
  ("json:vakhov-proxy-list", "https://raw.githubusercontent.com/vakhov/fresh-proxy-list/refs/heads/master/proxylist.json"),
  ("json:geonode", "https://proxylist.geonode.com/api/proxy-list?limit=500&page=1&sort_by=lastChecked&sort_type=desc"),
  ("json:geonode", "https://proxylist.geonode.com/api/proxy-list?limit=500&page=2&sort_by=lastChecked&sort_type=desc"),
  ("json:geonode", "https://proxylist.geonode.com/api/proxy-list?limit=500&page=3&sort_by=lastChecked&sort_type=desc"),
  ("json:proxyfreeonly", "https://proxyfreeonly.com/api/free-proxy-list?limit=500&page=1&sortBy=lastChecked&sortType=desc"),
  ("json:proxyfreeonly", "https://proxyfreeonly.com/api/free-proxy-list?limit=500&page=2&sortBy=lastChecked&sortType=desc"),
  ("json:proxyfreeonly", "https://proxyfreeonly.com/api/free-proxy-list?limit=500&page=3&sortBy=lastChecked&sortType=desc"),
];

const UNTRUSTED_SERVICES: [(&'static str, &'static str); 7] = [
  (
    "text:gfpcom-proxy-list",
    "https://raw.githubusercontent.com/wiki/gfpcom/free-proxy-list/lists/http.txt",
  ),
  (
    "text:gfpcom-proxy-list",
    "https://raw.githubusercontent.com/wiki/gfpcom/free-proxy-list/lists/socks4.txt",
  ),
  (
    "text:gfpcom-proxy-list",
    "https://raw.githubusercontent.com/wiki/gfpcom/free-proxy-list/lists/socks5.txt",
  ),
  (
    "text:fyvri-proxy-list",
    "https://raw.githubusercontent.com/fyvri/fresh-proxy-list/archive/storage/classic/socks5.txt",
  ),
  (
    "text:r00tee-proxy-list",
    "https://raw.githubusercontent.com/r00tee/Proxy-List/main/Socks5.txt",
  ),
  (
    "text:vmheaven-proxy-list",
    "https://raw.githubusercontent.com/vmheaven/VMHeaven-Free-Proxy-Updated/refs/heads/main/socks5_anonymous.txt",
  ),
  (
    "text:iplocate-proxy-list",
    "https://raw.githubusercontent.com/iplocate/free-proxy-list/refs/heads/main/protocols/socks5.txt",
  ),
];

const MIXED_SERVICES: [(&'static str, &'static str); 18] = {
  let mut arr = [("", ""); 18];

  let mut i = 0;
  while i < TRUSTED_SERVICES.len() {
    arr[i] = TRUSTED_SERVICES[i];
    i += 1;
  }

  let mut j = 0;
  while j < UNTRUSTED_SERVICES.len() {
    arr[i + j] = UNTRUSTED_SERVICES[j];
    j += 1;
  }

  arr
};

#[derive(Deserialize, Clone, Debug)]
pub struct ProxyCollectorOpts {
  pub algorithm: String,
  pub country: String,
  pub port: String,
  pub count: String,
}

enum ServiceResponse {
  Json(Value),
  Text(String),
}

/// Функция получения сервисов, склонных к указанному алгоритму
fn services_by_algorithm(algorithm: &str) -> Vec<(&'static str, &'static str)> {
  match algorithm {
    "trusted" => TRUSTED_SERVICES.to_vec(),
    "untrusted" => UNTRUSTED_SERVICES.to_vec(),
    _ => MIXED_SERVICES.to_vec(),
  }
}

/// Функция отправки запроса
async fn send_request(url: &str, content_class: &str) -> Option<ServiceResponse> {
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(6))
    .build()
    .ok()?;

  let resp = client.get(url).send().await.ok()?;

  if !resp.status().is_success() {
    return None;
  }

  match content_class {
    "json" => Some(ServiceResponse::Json(resp.json::<Value>().await.ok()?)),
    "text" => Some(ServiceResponse::Text(resp.text().await.ok()?)),
    _ => None,
  }
}

/// Вспомогательная функция запуска задачи скрапинга
fn spawn_scraper_task(
  tag: &'static str,
  url: &'static str,
  opts: ProxyCollectorOpts,
) -> tokio::task::JoinHandle<Vec<String>> {
  tokio::spawn(async move {
    let tag_split: Vec<&str> = tag.split(':').collect();
    if tag_split.len() < 2 {
      return Vec::new();
    }

    let content_class = tag_split[0];
    let name = tag_split[1];

    let Some(resp) = send_request(url, content_class).await else {
      return Vec::new();
    };

    process_response(resp, name, &opts)
  })
}

/// Функция обработки ответа и извлечения прокси
fn process_response(resp: ServiceResponse, name: &str, opts: &ProxyCollectorOpts) -> Vec<String> {
  let mut proxies = Vec::new();

  match resp {
    ServiceResponse::Json(json) => extract_proxies_from_json(name, opts, json, &mut proxies),
    ServiceResponse::Text(text) => extract_proxies_from_text(name, text, &mut proxies),
  }

  proxies
}

/// Функция извлечения прокси из JSON контента
fn extract_proxies_from_json(name: &str, opts: &ProxyCollectorOpts, json: Value, proxies: &mut Vec<String>) {
  let country_upper = opts.country.to_uppercase();

  match name {
    "proxyscrape" => {
      if let Some(arr) = json.get("proxies").and_then(|v| v.as_array()) {
        for proxy in arr {
          let p_country = proxy
            .get("ip_data")
            .and_then(|d| d.get("countryCode"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

          let p_protocol = proxy
            .get("protocol")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();

          if (opts.country == "any" || p_country == country_upper) && p_protocol == "socks5" {
            if let Some(p_str) = proxy.get("proxy").and_then(|v| v.as_str()) {
              proxies.push(p_str.to_string());
            }
          }
        }
      }
    }
    "proxifly" => {
      if let Some(arr) = json.as_array() {
        for proxy in arr {
          let p_country = proxy
            .get("geolocation")
            .and_then(|g| g.get("country"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

          let p_protocol = proxy
            .get("protocol")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();

          if (opts.country == "any" || p_country == country_upper) && p_protocol == "socks5" {
            if let Some(p_str) = proxy.get("proxy").and_then(|v| v.as_str()) {
              proxies.push(p_str.to_string());
            }
          }
        }
      }
    }
    "jetkai-proxy-list" => {
      if let Some(arr) = json.as_array() {
        for proxy in arr {
          let p_country = proxy
            .get("location")
            .and_then(|l| l.get("isocode"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

          let p_type = proxy
            .get("protocols")
            .and_then(|p| p.as_array())
            .and_then(|a| a.first())
            .and_then(|p| p.get("type"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();

          if (opts.country == "any" || p_country == country_upper) && p_type == "socks5" {
            let ip = proxy.get("ip").and_then(|v| v.as_str()).unwrap_or("");
            let port = proxy.get("port").and_then(|v| v.as_str()).unwrap_or("");
            if !ip.is_empty() && !port.is_empty() {
              proxies.push(format!("{}://{}:{}", p_type, ip, port));
            }
          }
        }
      }
    }
    "monosans-proxy-list" => {
      if let Some(arr) = json.as_array() {
        for proxy in arr {
          if proxy.get("username").is_some() || proxy.get("password").is_some() {
            continue;
          }

          let p_country = proxy
            .get("geolocation")
            .and_then(|g| g.get("country"))
            .and_then(|c| c.get("iso_code"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

          let p_protocol = proxy
            .get("protocol")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();

          if (opts.country == "any" || p_country == country_upper) && p_protocol == "socks5" {
            let host = proxy.get("host").and_then(|v| v.as_str()).unwrap_or("");
            let port = proxy.get("port").and_then(|v| v.as_str()).unwrap_or("");
            if !host.is_empty() && !port.is_empty() {
              proxies.push(format!("{}://{}:{}", p_protocol, host, port));
            }
          }
        }
      }
    }
    "vakhov-proxy-list" => {
      if let Some(arr) = json.as_array() {
        for proxy in arr {
          let p_country = proxy.get("country_code").and_then(|v| v.as_str()).unwrap_or("");
          if opts.country != "any" && p_country != country_upper {
            continue;
          }

          let ip = proxy.get("ip").and_then(|v| v.as_str()).unwrap_or("");
          let port = proxy.get("port").and_then(|v| v.as_str()).unwrap_or("");
          if ip.is_empty() || port.is_empty() {
            continue;
          }

          if proxy.get("socks5").and_then(|v| v.as_i64()).unwrap_or(0) != 0 {
            proxies.push(format!("{}://{}:{}", "socks5", ip, port));
          }
        }
      }
    }
    "geonode" | "proxyfreeonly" => {
      let arr = json.get("data").and_then(|v| v.as_array()).or_else(|| json.as_array());

      if let Some(arr) = arr {
        for proxy in arr {
          let p_country = proxy.get("country").and_then(|v| v.as_str()).unwrap_or("");
          if opts.country != "any" && p_country != country_upper {
            continue;
          }

          let ip = proxy.get("ip").and_then(|v| v.as_str()).unwrap_or("");
          let port = proxy.get("port").and_then(|v| v.as_str()).unwrap_or("");
          if ip.is_empty() || port.is_empty() {
            continue;
          }

          if let Some(protocols) = proxy.get("protocols").and_then(|v| v.as_array()) {
            for p in protocols {
              if let Some(p_str) = p.as_str() {
                let protocol_lower = p_str.to_lowercase();
                if protocol_lower == "socks5" {
                  proxies.push(format!("{}://{}:{}", protocol_lower, ip, port));
                }
              }
            }
          }
        }
      }
    }
    _ => {}
  }
}

/// Функция извлечения прокси из TEXT контента
fn extract_proxies_from_text(name: &str, text: String, proxies: &mut Vec<String>) {
  let lines: Vec<&str> = text.lines().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

  match name {
    "gfpcom-proxy-list" => {
      for line in lines {
        let parts: Vec<&str> = line.split("://").collect();
        let line_protocol = if parts.len() > 1 {
          parts[0].to_lowercase()
        } else {
          "http".to_string()
        };

        if line_protocol == "socks5" {
          proxies.push(line.to_string());
        }
      }
    }
    "fyvri-proxy-list" | "r00tee-proxy-list" | "vmheaven-proxy-list" | "iplocate-proxy-list" => {
      // Тут все только SOCKS5
      for line in lines {
        proxies.push(format!("socks5://{}", line));
      }
    }
    _ => {}
  }
}

/// Функция фильтровки прокси по порту
fn filter_by_port(proxy_list: Vec<String>, port_pattern: String) -> Vec<String> {
  let mut filtered = Vec::new();

  if port_pattern != "any" {
    for proxy in proxy_list {
      if proxy.contains(&port_pattern) {
        filtered.push(proxy);
      }
    }
  } else {
    filtered = proxy_list;
  }

  filtered
}

/// Функция скрапинга прокси
async fn scrape_proxies(options: ProxyCollectorOpts) -> Vec<String> {
  let mut tasks = Vec::new();

  let target_services = services_by_algorithm(&options.algorithm);

  for (tag, url) in target_services {
    tasks.push(spawn_scraper_task(tag, url, options.clone()));
  }

  let mut results = Vec::new();

  for task in tasks {
    results.push(task.await);
  }

  let mut proxy_set: HashSet<String> = HashSet::new();
  for res in results {
    if let Ok(proxies) = res {
      for p in proxies {
        proxy_set.insert(p);
      }
    }
  }

  let mut proxy_list: Vec<String> = proxy_set.into_iter().collect();

  let limit = if options.count == "max" {
    MAX_PROXY_COUNT
  } else {
    options.count.parse::<usize>().unwrap_or(MAX_PROXY_COUNT)
  };

  if proxy_list.len() > limit {
    proxy_list.truncate(limit);
  }

  filter_by_port(proxy_list, options.port)
}

/// Функция проверки одного прокси
async fn check_proxy(proxy: String) -> bool {
  let without_protocol = proxy.replace("socks5://", "");
  let splited = without_protocol.split('@').collect::<Vec<&str>>();

  let (auth_str, addr) = match splited.get(1) {
    Some(addr) => (Some(splited[0]), addr.to_string()),
    None => (None, splited[0].to_string()),
  };

  let auth = if let Some(a) = auth_str {
    let split = a.split(':').collect::<Vec<&str>>();
    let mut auth = None;

    if let Some(username) = split.get(0) {
      if let Some(password) = split.get(1) {
        auth = Some(UserKey::new(*username, *password));
      }
    }

    auth
  } else {
    None
  };

  let services = vec![
    ("cloudflare.com", 80),
    ("facebook.com", 80),
    ("yandex.ru", 80),
    ("youtube.com", 80),
    ("github.com", 80),
    ("reddit.com", 80),
  ];

  let success_requests = Arc::new(AtomicU8::new(0));
  let mut tasks = Vec::new();

  for (hostname, port) in services {
    let hostname_string = hostname.to_string();
    let auth_clone = auth.clone();
    let addr_clone = addr.clone();
    let success_requests_clone = success_requests.clone();
    let task = tokio::spawn(async move {
      let mut stream = match tokio::time::timeout(Duration::from_secs(8), TcpStream::connect(addr_clone)).await {
        Ok(res) => match res {
          Ok(s) => s,
          Err(_) => return,
        },
        Err(_) => return,
      };

      let target_addr = Address::DomainAddress(hostname_string.into(), port);

      match tokio::time::timeout(
        Duration::from_secs(8),
        socks5_impl::client::connect(&mut stream, target_addr, auth_clone),
      )
      .await
      {
        Ok(res) => match res {
          Ok(_) => {
            success_requests_clone.fetch_add(1, Ordering::SeqCst);
          }
          _ => {}
        },
        _ => {}
      }
    });

    tasks.push(task);
  }

  for task in tasks {
    let _ = task.await;
  }

  success_requests.load(Ordering::SeqCst) >= 2
}

/// Команда сбора прокси
#[tauri::command]
pub async fn collect_proxies(options: ProxyCollectorOpts) -> Vec<u8> {
  let proxies = scrape_proxies(options).await;
  proxies.join("\n").as_bytes().to_vec()
}

/// Команда проверки прокси
#[tauri::command]
pub async fn check_proxies(bytes: Vec<u8>) -> Vec<u8> {
  let proxies_str = String::from_utf8_lossy(&bytes);
  let proxies = proxies_str.split('\n').collect::<Vec<&str>>();
  let checked_proxies = Arc::new(RwLock::new(Vec::new()));
  let mut tasks = Vec::new();

  for proxy in proxies {
    let cpc = checked_proxies.clone();
    let proxy_string = proxy.to_string();
    let task = tokio::spawn(async move {
      if check_proxy(proxy_string.clone()).await {
        cpc.write().await.push(proxy_string);
      }
    });

    tasks.push(task);
  }

  for task in tasks {
    let _ = task.await;
  }

  let guard = checked_proxies.read().await;
  guard.join("\n").as_bytes().to_vec()
}
