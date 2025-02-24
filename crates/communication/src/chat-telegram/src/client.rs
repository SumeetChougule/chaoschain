use anyhow::Result;
use derive_more::{Deref, DerefMut};
use teloxide_core::{
    prelude::Requester,
    types::{ChatAction, ChatId, ParseMode},
    Bot,
};
use tracing::{info, error};

#[derive(Clone)]
pub struct Client {
    bot: Bot,
    group_id: ChatId,
}

impl Client {
    pub fn new(config: &TelegramConfig) -> Self {
        let bot = Bot::new(&config.api_key);
        Self { 
            bot,
            group_id: ChatId(config.group_id),
        }
    }

    pub async fn verify(&self) -> Result<()> {
        let me = self.bot.get_me().await?;
        info!("Telegram bot '{}' initialized", me.username());
        Ok(())
    }

    pub async fn send_message(&self, message: &str) -> Result<()> {
        self.bot
            .send_message(self.group_id, message)
            .parse_mode(ParseMode::Html)
            .await?;
        Ok(())
    }

    pub async fn send_message_with_typing(&self, message: &str) -> Result<()> {
        self.bot
            .send_chat_action(self.group_id, ChatAction::Typing)
            .await?;

        self.bot
            .send_message(self.group_id, message)
            .parse_mode(ParseMode::Html)
            .await?;

        Ok(())
    }
}
