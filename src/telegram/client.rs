use crate::telegram::TelegramRequest;

use tracing::debug;

use super::{GetUpdatesResponse, SendMessageResponse, TelegramError};

pub struct Client {
    pub token: String,
}

impl Client {
    pub async fn send_messages(
        &self,
        chat_id: u64,
        text: &str,
    ) -> Result<SendMessageResponse, TelegramError> {
        let resp = send_telegram_request(
            &self.token,
            &TelegramRequest::SendMessage {
                chat_id: chat_id,
                text: text.to_owned(),
            },
        )
        .await;

        let resp = match resp {
            Ok(val) => Ok(val),
            Err(e) => Err(TelegramError::HttpError { msg: e.to_string() }),
        }?;

        let resp = serde_json::from_str(&resp);

        match resp {
            Ok(val) => Ok(val),
            Err(e) => Err(TelegramError::JsonError { msg: e.to_string() }),
        }
    }

    pub async fn get_messages(
        &self,
        offset: Option<i64>,
        timeout: Option<u32>,
    ) -> Result<GetUpdatesResponse, TelegramError> {
        // let token: &str = &self.token;
        let resp = send_telegram_request(
            &self.token,
            &TelegramRequest::Updates {
                offset: offset,
                limit: Some(1),
                timeout: timeout.unwrap_or(60),
            },
        )
        .await;
        let resp = match resp {
            Ok(val) => Ok(val),
            Err(e) => Err(TelegramError::HttpError { msg: e.to_string() }),
        }?;

        let resp = serde_json::from_str(&resp);

        match resp {
            Ok(val) => Ok(val),
            Err(e) => Err(TelegramError::JsonError { msg: e.to_string() }),
        }
    }
}

/// Basic method for sending requests to the telegram API, acts as a building block for other functions
async fn send_telegram_request(
    token: &str,
    request: &TelegramRequest,
) -> Result<String, reqwest::Error> {
    let (method, body) = match request {
        TelegramRequest::Me => ("getMe", None),
        TelegramRequest::Updates {
            offset: _,
            limit: _,
            timeout: _,
        } => ("getUpdates", Some(serde_json::to_string(request).unwrap())),
        TelegramRequest::SendMessage {
            chat_id: _,
            text: _,
        } => ("sendMessage", Some(serde_json::to_string(request).unwrap())),
    };

    let url = format!("https://api.telegram.org/bot{token}/{method}");

    debug!(body, "sending request to telegram");

    let client = reqwest::Client::new();
    let builder = client.post(url);
    let builder = match body {
        Some(content) => builder
            .header("Content-Type", "Application/json")
            .body(content),
        None => builder,
    };

    let resp = builder.send().await?.text().await?;

    debug!(response = resp, "got response from telegram");

    Ok(resp)
}
