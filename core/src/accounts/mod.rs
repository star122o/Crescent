use crate::minecraft::BotError;
use crescent_protocol::ClientAccount;
use crescent_auth::microsoft::{begin_device_flow, poll_for_token, DEFAULT_CLIENT_ID};
use crescent_auth::authenticate;

#[cfg_attr(feature = "node-binding", napi_derive::napi)]
#[cfg_attr(feature = "native-binding", derive(uniffi::Object))]
pub struct MinecraftAccount {
  pub(crate) account: ClientAccount,
}

#[cfg_attr(feature = "node-binding", napi_derive::napi)]
#[cfg_attr(feature = "native-binding", uniffi::export)]
impl MinecraftAccount {
  #[cfg_attr(feature = "node-binding", napi(factory))]
  #[cfg_attr(feature = "native-binding", uniffi::constructor)]
  pub fn offline(username: String) -> Self {
    Self {
      account: ClientAccount::Offline { username },
    }
  }

  #[cfg_attr(feature = "node-binding", napi(factory))]
  #[cfg_attr(feature = "native-binding", uniffi::constructor)]
  pub async fn microsoft(_email: String) -> Result<Self, BotError> {
    Err(BotError::ConnectionError {
      message: "Direct Microsoft login is deprecated. Please use MicrosoftAuthFlow instead.".to_string(),
    })
  }
}

#[cfg_attr(feature = "node-binding", napi_derive::napi(object))]
#[cfg_attr(feature = "native-binding", derive(uniffi::Record))]
#[derive(Clone, Debug)]
pub struct MicrosoftAuthInfo {
  pub user_code: String,
  pub verification_uri: String,
  pub device_code: String,
  pub expires_in: u32,
  pub interval: u32,
}

#[cfg_attr(feature = "node-binding", napi_derive::napi)]
#[cfg_attr(feature = "native-binding", derive(uniffi::Object))]
pub struct MicrosoftAuthFlow {
  client: reqwest::Client,
}

#[cfg_attr(feature = "node-binding", napi_derive::napi)]
#[cfg_attr(feature = "native-binding", uniffi::export)]
impl MicrosoftAuthFlow {
  #[cfg_attr(feature = "node-binding", napi(constructor))]
  #[cfg_attr(feature = "native-binding", uniffi::constructor)]
  pub fn new() -> Self {
    Self {
      client: reqwest::Client::new(),
    }
  }

  pub async fn initiate(&self) -> Result<MicrosoftAuthInfo, BotError> {
    let response = begin_device_flow(&self.client, DEFAULT_CLIENT_ID).await
      .map_err(|e| BotError::ConnectionError { message: e.to_string() })?;

    Ok(MicrosoftAuthInfo {
      user_code: response.user_code,
      verification_uri: response.verification_uri,
      device_code: response.device_code,
      expires_in: response.expires_in as u32,
      interval: response.interval as u32,
    })
  }

  pub async fn poll(&self, info: MicrosoftAuthInfo) -> Result<MinecraftAccount, BotError> {
    let dev_resp = crescent_auth::microsoft::DeviceCodeResponse {
      device_code: info.device_code,
      user_code: info.user_code,
      verification_uri: info.verification_uri,
      expires_in: info.expires_in as u64,
      interval: info.interval as u64,
    };

    let msa = poll_for_token(&self.client, DEFAULT_CLIENT_ID, &dev_resp).await
      .map_err(|e| BotError::ConnectionError { message: e.to_string() })?;

    let (mc_token, profile) = authenticate(&self.client, &msa.access_token).await
      .map_err(|e| BotError::ConnectionError { message: e.to_string() })?;

    Ok(MinecraftAccount {
      account: ClientAccount::Online {
        username: profile.name,
        uuid: profile.id,
        access_token: mc_token.access_token,
      },
    })
  }
}

impl Default for MicrosoftAuthFlow {
  fn default() -> Self {
    Self::new()
  }
}
