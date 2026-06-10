use std::sync::Arc;
use crescent_protocol::{ProtocolClient, ProtocolVersion};

#[cfg_attr(feature = "native-binding", derive(uniffi::Error))]
#[derive(Debug)]
pub enum BotError {
  ConnectionError { message: String },
}

impl std::fmt::Display for BotError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      BotError::ConnectionError { message } => write!(f, "Connection error: {message}"),
    }
  }
}
impl std::error::Error for BotError {}

#[cfg(feature = "node-binding")]
impl From<BotError> for napi::Error {
  fn from(err: BotError) -> Self {
    napi::Error::from_reason(err.to_string())
  }
}

#[cfg_attr(feature = "native-binding", derive(uniffi::Record))]
#[cfg_attr(feature = "node-binding", napi_derive::napi(object))]
#[derive(Debug, Clone)]
pub struct BotEvent {
  pub kind: String,
  pub payload: String,
}

#[cfg_attr(feature = "native-binding", uniffi::export(with_foreign))]
pub trait BotEventListener: Send + Sync {
  fn on_event(&self, event: BotEvent);
}

#[cfg(feature = "node-binding")]
impl BotEventListener for napi::threadsafe_function::ThreadsafeFunction<BotEvent> {
  fn on_event(&self, event: BotEvent) {
    let _ = self.call(
      Ok(event),
      napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
    );
  }
}

#[cfg_attr(feature = "node-binding", napi_derive::napi)]
#[cfg_attr(feature = "native-binding", derive(uniffi::Object))]
pub struct MinecraftBot {
  client: tokio::sync::Mutex<Option<ProtocolClient>>,
}

#[cfg_attr(feature = "node-binding", napi_derive::napi)]
#[cfg_attr(feature = "native-binding", uniffi::export)]
impl MinecraftBot {
  #[cfg_attr(feature = "node-binding", napi(constructor))]
  #[cfg_attr(feature = "native-binding", uniffi::constructor)]
  pub fn new() -> Self {
    Self {
      client: tokio::sync::Mutex::new(None),
    }
  }
}

#[cfg(feature = "node-binding")]
#[napi_derive::napi]
impl MinecraftBot {
  #[napi]
  pub async fn connect(&self, account: &crate::accounts::MinecraftAccount, address: String) -> Result<(), BotError> {
    self.connect_core(account.account.clone(), address).await
  }

  #[napi]
  pub async fn listen(&self, callback: napi::threadsafe_function::ThreadsafeFunction<BotEvent>) -> Result<(), BotError> {
    self.listen_core(Arc::new(callback)).await
  }

  #[napi]
  pub async fn chat(&self, message: String) -> Result<(), BotError> {
    self.chat_core(message).await
  }
}

#[cfg(feature = "native-binding")]
#[uniffi::export]
impl MinecraftBot {
  pub async fn connect(&self, account: Arc<crate::accounts::MinecraftAccount>, address: String) -> Result<(), BotError> {
    self.connect_core(account.account.clone(), address).await
  }

  pub async fn listen(&self, listener: Arc<dyn BotEventListener>) -> Result<(), BotError> {
    self.listen_core(listener).await
  }

  pub async fn chat(&self, message: String) -> Result<(), BotError> {
    self.chat_core(message).await
  }
}

impl MinecraftBot {
  async fn connect_core(&self, account: crescent_protocol::ClientAccount, address: String) -> Result<(), BotError> {
    // Parse host and port from address (e.g. "localhost:25565" or "localhost")
    let parts: Vec<&str> = address.split(':').collect();
    let host = parts[0].to_string();
    let port = if parts.len() > 1 {
      parts[1].parse::<u16>().unwrap_or(25565)
    } else {
      25565
    };

    // Protocol version is 774 (1.21.11) as requested by user
    let client = ProtocolClient::connect(&host, port, account, ProtocolVersion::V1_21_11).await
      .map_err(|e| BotError::ConnectionError { message: e.to_string() })?;

    *self.client.lock().await = Some(client);
    Ok(())
  }

  async fn listen_core(&self, listener: Arc<dyn BotEventListener>) -> Result<(), BotError> {
    let mut rx = {
      let guard = self.client.lock().await;
      if let Some(ref client) = *guard {
        client.subscribe()
      } else {
        return Err(BotError::ConnectionError { message: "Bot not connected".to_string() });
      }
    };

    while let Ok(event) = rx.recv().await {
        match event {
          crescent_protocol::BotEvent::Spawned => {
            listener.on_event(BotEvent {
              kind: "init".to_string(),
              payload: String::new(),
            });
          }
          crescent_protocol::BotEvent::Chat { sender: _, message } => {
            listener.on_event(BotEvent {
              kind: "chat".to_string(),
              payload: message,
            });
          }
          crescent_protocol::BotEvent::Disconnect { reason } => {
            listener.on_event(BotEvent {
              kind: "disconnect".to_string(),
              payload: reason,
            });
            break;
          }
        }
      }
      Ok(())
  }

  async fn chat_core(&self, message: String) -> Result<(), BotError> {
    let guard = self.client.lock().await;
    if let Some(ref client) = *guard {
      client.send_chat(&message).await
        .map_err(|e| BotError::ConnectionError { message: e.to_string() })?;
      Ok(())
    } else {
      Err(BotError::ConnectionError { message: "Bot not connected".to_string() })
    }
  }
}
