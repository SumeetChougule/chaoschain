use ice9_core::Config;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TelegramConfig {
    pub api_key: String,
    pub group_id: i64,
}

impl Config for TelegramConfig {
    const NAMESPACE: &str = "telegram";

    fn template() -> Self {
        Self {
            api_key: "API KEY HERE".into(),
            group_id: -1,
        }
    }
}
