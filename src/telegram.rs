use serde::{Deserialize, Serialize};
use tracing::{debug, info};

pub mod client;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum TelegramRequest {
    Me,
    Updates {
        offset: Option<i64>,
        limit: Option<i64>,
        timeout: i64,
    },
}

#[derive(Serialize, Deserialize)]
pub struct TelegramResponse {
    pub ok: bool,
    pub result: Vec<TelegramResult>,
}

#[derive(Serialize, Deserialize)]
pub struct TelegramResult {
    pub update_id: i64,
}

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

    Ok(resp)
}

pub enum TelegramError {
    HttpError { msg: String },
    JsonError { msg: String },
}

pub fn telegram_get_messages(
    token: &str,
    offset: Option<i64>,
) -> Result<TelegramResponse, TelegramError> {
    let resp = send_telegram_request(
        token,
        &TelegramRequest::Updates {
            offset: offset,
            limit: Some(1),
            timeout: 5,
        },
    )
    .await;
    let resp = match resp {
        Ok(val) => Ok(val),
        Err(e) => Err(TelegramError::HttpError { msg: e.to_string() }),
    }?;

    info!(response = resp, "got JSON response from telegram");

    let tresp = serde_json::from_str(&resp);

    match tresp {
        Ok(val) => Ok(val),
        Err(e) => Err(TelegramError::JsonError { msg: e.to_string() }),
    }
}
