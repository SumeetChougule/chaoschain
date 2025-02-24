use anyhow::Result;
use async_trait::async_trait;

/// A common trait for a communication channel (e.g. Telegram, Slack, etc.)
#[async_trait]
pub trait CommunicationChannel: Send + Sync {
    /// Returns the name of the channel.
    fn channel_name(&self) -> &str;
    /// Send a message to the designated group or channel.
    async fn send_message(&self, message: String) -> Result<()>;
}


pub mod telegram;