use anyhow::Result;
use teloxide::{prelude::*, types::ChatId};
use async_trait::async_trait;
use tracing::info;

use crate::CommunicationChannel;
use chaoschain_core::NetworkEvent;

/// TelegramChannel is responsible for sending messages to a Telegram group.
pub struct TelegramChannel {
    pub bot: Bot,
    pub group_id: ChatId,
}

impl TelegramChannel {
    /// Create a new TelegramChannel instance.
    pub fn new(bot_token: String, group_id: i64) -> Self {
        Self {
            bot: Bot::new(bot_token),
            group_id: ChatId(group_id),
        }
    }

    /// Listen for network events on a broadcast channel and forward them to Telegram.
    pub async fn run_broadcast(
        &self,
        mut rx: tokio::sync::broadcast::Receiver<NetworkEvent>,
    ) -> Result<()> {
        use tokio::sync::broadcast::error::RecvError;
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let msg_for_log = event.message.clone();
                    if let Err(err) = self.bot.send_message(self.group_id, event.message).await {
                        tracing::error!("Error: {}", msg_for_log);
                        tracing::error!("Error sending message to Telegram: {:?}", err );
                    }
                }
                Err(RecvError::Lagged(count)) => {
                    tracing::warn!("Lagged: missed {} messages", count);
                }
                Err(RecvError::Closed) => break,
            }
        }
        Ok(())
    }
}

#[async_trait]
impl CommunicationChannel for TelegramChannel {
    async fn send_message(&self, message: String) -> Result<()> {
        self.bot.send_message(self.group_id, message).await?;
        Ok(())
    }

    fn channel_name(&self) -> &str {
        "Telegram"
    }
}