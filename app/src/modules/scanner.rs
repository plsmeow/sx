use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bytes::BytesMut;
use once_cell::sync::Lazy;
use salarixi_extensions::buffer::BufferExt;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::emit::EMIT_MANAGER;
use crate::result::CommandResult;
use crate::{failed, success};

static SCANNER_ROOT_TASK: Lazy<RwLock<Option<JoinHandle<()>>>> = Lazy::new(|| RwLock::new(None));
static SCANNER_EXTRA_TASKS: Lazy<RwLock<Vec<JoinHandle<()>>>> = Lazy::new(|| RwLock::new(Vec::new()));

/// Функция парсинга диапазона IP-адресов
fn parse_ip_range(range: &str) -> CommandResult<Vec<String>> {
  let splitted = range.split('/').collect::<Vec<&str>>();

  let Ok(ip) = splitted[0].parse::<Ipv4Addr>() else {
    failed!("null address parsing error");
  };

  let Ok(designation) = splitted[1].parse::<u32>() else {
    failed!("designation parsing error");
  };

  if designation < 20 {
    failed!("number after '/' cannot be less than 20 or greater than 32");
  }

  let mask = if designation == 0 {
    0
  } else {
    !0u32 << (32 - designation)
  };

  let ip_num: u32 = ip.into();
  let null = ip_num & mask;
  let broadcast = null | !mask;

  let null_addr = Ipv4Addr::from(null);
  let null_octets = null_addr.as_octets();
  let broadcast_addr = Ipv4Addr::from(broadcast);
  let broadcast_octets = broadcast_addr.as_octets();

  let mut hosts = Vec::new();

  for a in 0..=255 {
    if a < null_octets[0] || a > broadcast_octets[0] {
      continue;
    }

    for b in 0..=255 {
      if b < null_octets[1] || b > broadcast_octets[1] {
        continue;
      }

      for c in 0..=255 {
        if c < null_octets[2] || c > broadcast_octets[2] {
          continue;
        }

        for d in 0..=255 {
          if d < null_octets[3] || d > broadcast_octets[3] {
            continue;
          }

          let octets = [a, b, c, d];
          if octets == *null_octets || octets == *broadcast_octets {
            continue;
          }

          let host = Ipv4Addr::from(octets).to_string();
          hosts.push(host);
        }
      }
    }
  }

  success!(hosts);
}

fn set_scanner_status(status: u8) {
  let mut buf = BytesMut::new();
  status.write(&mut buf);

  EMIT_MANAGER.emit("scanner:status", buf.into());
}

fn push_scanned_server(address: String, icon: Option<String>, online_players: i32, max_players: i32, version: String) {
  let mut buf = BytesMut::new();
  address.write(&mut buf);
  icon.write(&mut buf);
  online_players.write(&mut buf);
  max_players.write(&mut buf);
  version.write(&mut buf);

  EMIT_MANAGER.emit("scanner:push-server", buf.into());
}

fn set_hosts_scanned(scanned: u16, total: u16) {
  let mut buf = BytesMut::new();
  scanned.write(&mut buf);
  total.write(&mut buf);

  EMIT_MANAGER.emit("scanner:hosts-scanned", buf.into());
}

fn set_servers_found(count: u16) {
  let mut buf = BytesMut::new();
  count.write(&mut buf);

  EMIT_MANAGER.emit("scanner:servers-found", buf.into());
}

/// Функция сканирования сети на наличие Minecraft серверов
async fn start_scanning(range: String, mut task_count: usize, target_port: u32, timeout: u64) -> CommandResult<()> {
  let mut extra_tasks = SCANNER_EXTRA_TASKS.write().await;
  extra_tasks.iter().for_each(|t| t.abort());
  extra_tasks.clear();
  drop(extra_tasks);

  if let Some(task) = SCANNER_ROOT_TASK.read().await.as_ref() {
    task.abort();
  }

  let result = parse_ip_range(&range);
  let Some(data) = result.data else {
    failed!("{}", result.error.unwrap_or("unknown error".to_string()));
  };

  let total_hosts = data.len();

  set_scanner_status(1);
  set_hosts_scanned(0, total_hosts as u16);
  set_servers_found(0);

  if task_count > total_hosts {
    task_count = total_hosts;
  }

  let scanner_task = tokio::spawn(async move {
    let hosts = Arc::new(RwLock::new(data));
    let servers = Arc::new(AtomicU16::new(0));
    let status_reset = Arc::new(AtomicBool::new(false));

    for _ in 0..task_count {
      let hosts_clone = hosts.clone();
      let servers_clone = servers.clone();
      let status_reset_clone = status_reset.clone();
      let task = tokio::spawn(async move {
        loop {
          let mut hosts_guard = hosts_clone.write().await;
          if hosts_guard.len() < 1 {
            if !status_reset_clone.load(Ordering::SeqCst) {
              status_reset_clone.store(true, Ordering::SeqCst);
              stop_scanning().await;
            }

            break;
          }

          let Some(target_host) = hosts_guard.pop() else {
            continue;
          };

          drop(hosts_guard);

          let target_addr = format!("{}:{}", target_host, target_port);
          let result = tokio::time::timeout(
            Duration::from_millis(timeout),
            azalea::ping::ping_server(target_addr.clone()),
          )
          .await;

          if let Ok(Ok(data)) = result {
            push_scanned_server(
              target_addr,
              data.favicon,
              data.players.online,
              data.players.max,
              data.version.name,
            );

            let previous = servers_clone.fetch_add(1, Ordering::SeqCst);
            set_servers_found(previous + 1);
          }

          set_hosts_scanned(
            (total_hosts - hosts_clone.read().await.len()) as u16,
            total_hosts as u16,
          );
        }
      });

      SCANNER_EXTRA_TASKS.write().await.push(task);
    }
  });

  *SCANNER_ROOT_TASK.write().await = Some(scanner_task);

  success!(());
}

/// Функция остановки сканирования серверов
async fn stop_scanning() {
  let mut extra_tasks = SCANNER_EXTRA_TASKS.write().await;
  extra_tasks.iter().for_each(|t| t.abort());
  extra_tasks.clear();
  drop(extra_tasks);

  if let Some(task) = SCANNER_ROOT_TASK.read().await.as_ref() {
    task.abort();
  }

  *SCANNER_ROOT_TASK.write().await = None;

  set_scanner_status(0);
}

#[tauri::command]
pub async fn start_network_scanning(
  range: String,
  task_count: usize,
  target_port: u32,
  timeout: u64,
) -> CommandResult<()> {
  start_scanning(range, task_count, target_port, timeout).await
}

#[tauri::command]
pub async fn stop_network_scanning() {
  stop_scanning().await;
}
