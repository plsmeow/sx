use std::future::Future;
use std::pin::Pin;
use std::sync::{Mutex, MutexGuard};

use azalea::account::{Account, AccountTrait};
use azalea::auth::cache::ExpiringValue;
use azalea::auth::sessionserver::{ClientSessionServerError, SessionServerJoinOpts};
use azalea::auth::certs::Certificates;
use azalea::auth::{
  AccessTokenResponse, AuthError, get_ms_auth_token, get_ms_link_code, get_minecraft_token,
  get_profile, refresh_ms_auth_token,
};
use uuid::Uuid;

use crate::launch::options::{AccountOptions, AccountType};
use crate::server::transfer::emit_log;

type AuthFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
  mutex.lock().unwrap_or_else(|e| e.into_inner())
}

/// Аккаунт, авторизующийся при помощи готового токена сессии Minecraft
#[derive(Debug)]
pub struct SessionTokenAccount {
  username: String,
  uuid: Uuid,
  access_token: Mutex<String>,
  certs: Mutex<Option<Certificates>>,
}

impl SessionTokenAccount {
  pub async fn new(access_token: String) -> Result<Self, String> {
    let client = reqwest::Client::new();

    let profile = get_profile(&client, &access_token)
      .await
      .map_err(|e| format!("failed to fetch profile: {}", e))?;

    Ok(Self {
      username: profile.name,
      uuid: profile.id,
      access_token: Mutex::new(access_token),
      certs: Mutex::new(None),
    })
  }
}

impl AccountTrait for SessionTokenAccount {
  fn username(&self) -> &str {
    &self.username
  }

  fn uuid(&self) -> Uuid {
    self.uuid
  }

  fn access_token(&self) -> Option<String> {
    Some(lock(&self.access_token).to_owned())
  }

  fn certs(&self) -> Option<Certificates> {
    lock(&self.certs).as_ref().cloned()
  }

  fn set_certs(&self, certs: Certificates) {
    *lock(&self.certs) = Some(certs);
  }

  fn join<'a>(
    &'a self,
    public_key: &'a [u8],
    private_key: &'a [u8; 16],
    server_id: &'a str,
    proxy: Option<reqwest::Proxy>,
  ) -> AuthFuture<'a, Result<(), ClientSessionServerError>> {
    Box::pin(async move {
      let access_token = lock(&self.access_token).clone();

      azalea::auth::sessionserver::join(SessionServerJoinOpts {
        access_token: &access_token,
        public_key,
        private_key,
        uuid: &self.uuid(),
        server_id,
        proxy,
      })
      .await
    })
  }
}

/// Аккаунт Microsoft с авторизацией через устройство и обновлением токена
#[derive(Debug)]
pub struct MicrosoftDeviceAccount {
  email: String,

  username: String,
  uuid: Uuid,

  msa: Mutex<ExpiringValue<AccessTokenResponse>>,
  access_token: Mutex<String>,
  client: reqwest::Client,
  certs: Mutex<Option<Certificates>>,
}

impl MicrosoftDeviceAccount {
  pub async fn new(email: &str) -> Result<Self, String> {
    let client = reqwest::Client::new();

    let link_code = get_ms_link_code(&client, None, None)
      .await
      .map_err(|e| format!("failed to get link code: {}", e))?;

    emit_log(
      format!(
        "Вход в аккаунт {}: открой ссылку {} и введи код {}",
        email, link_code.verification_uri, link_code.user_code
      ),
      "warning",
    );

    let msa = get_ms_auth_token(&client, link_code, None)
      .await
      .map_err(|e| format!("failed to get microsoft token: {}", e))?;

    let minecraft = get_minecraft_token(&client, &msa.data.access_token)
      .await
      .map_err(|e| format!("failed to get minecraft token: {}", e))?;

    let profile = get_profile(&client, &minecraft.minecraft_access_token)
      .await
      .map_err(|e| format!("failed to fetch profile: {}", e))?;

    emit_log(
      format!("Аккаунт {} успешно авторизован как {}", email, profile.name),
      "info",
    );

    Ok(Self {
      email: email.to_owned(),
      username: profile.name,
      uuid: profile.id,
      msa: Mutex::new(msa),
      access_token: Mutex::new(minecraft.minecraft_access_token),
      client,
      certs: Mutex::new(None),
    })
  }
}

impl AccountTrait for MicrosoftDeviceAccount {
  fn username(&self) -> &str {
    &self.username
  }

  fn uuid(&self) -> Uuid {
    self.uuid
  }

  fn access_token(&self) -> Option<String> {
    Some(lock(&self.access_token).to_owned())
  }

  fn certs(&self) -> Option<Certificates> {
    lock(&self.certs).as_ref().cloned()
  }

  fn set_certs(&self, certs: Certificates) {
    *lock(&self.certs) = Some(certs);
  }

  fn refresh(&self) -> AuthFuture<'_, Result<(), AuthError>> {
    Box::pin(async {
      let mut msa = lock(&self.msa).clone();

      if msa.is_expired() {
        msa = refresh_ms_auth_token(&self.client, &msa.data.refresh_token, None, None).await?;
      }

      let minecraft = get_minecraft_token(&self.client, &msa.data.access_token).await?;

      *lock(&self.msa) = msa;
      *lock(&self.access_token) = minecraft.minecraft_access_token;

      Ok(())
    })
  }

  fn join<'a>(
    &'a self,
    public_key: &'a [u8],
    private_key: &'a [u8; 16],
    server_id: &'a str,
    proxy: Option<reqwest::Proxy>,
  ) -> AuthFuture<'a, Result<(), ClientSessionServerError>> {
    Box::pin(async move {
      let access_token = lock(&self.access_token).clone();

      azalea::auth::sessionserver::join(SessionServerJoinOpts {
        access_token: &access_token,
        public_key,
        private_key,
        uuid: &self.uuid(),
        server_id,
        proxy,
      })
      .await
    })
  }
}

/// Метод создания аккаунта нужного типа из пользовательских настроек
pub async fn build_account(username: &str, opts: &AccountOptions) -> Option<Account> {
  match opts.account_type {
    AccountType::Offline => Some(Account::offline(username)),
    AccountType::Microsoft => match MicrosoftDeviceAccount::new(username).await {
      Ok(account) => Some(account.into()),
      Err(e) => {
        emit_log(
          format!("Не удалось авторизовать аккаунт {}: {}", username, e),
          "error",
        );
        None
      }
    },
    AccountType::Session => {
      let token = opts
        .access_token
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty());

      let Some(token) = token else {
        emit_log(
          format!(
            "Аккаунт {} не будет запущен: access token не указан",
            username
          ),
          "error",
        );
        return None;
      };

      match SessionTokenAccount::new(token.to_string()).await {
        Ok(account) => Some(account.into()),
        Err(e) => {
          emit_log(
            format!("Аккаунт {} не будет запущен: {}", username, e),
            "error",
          );
          None
        }
      }
    }
  }
}
