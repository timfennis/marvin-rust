use super::{telegram_get_messages, TelegramError, TelegramResponse};

pub struct Client {
    pub token: String,
}

impl Client {
    pub async fn get_messages(
        &self,
        offset: Option<i64>,
    ) -> Result<TelegramResponse, TelegramError> {
        telegram_get_messages(&self.token, offset).await
    }
}
