use serde::{Deserialize, Serialize};
use std::env;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let telegram_token = env::var("TELEGRAM_TOKEN")
        .expect("You need to specify a TELEGRAM_TOKEN environment variable");

    println!("Starting the marvin server");

    loop {
        match telegram_get_messages(&telegram_token, None).await {
            Ok(r) => println!("update id: {}", r.result.first().unwrap().update_id),
            Err(err) => match err {
                TelegramError::JsonError { msg } => println!("Telegram JSON Error: {msg}"),
                TelegramError::HttpError { msg }  => println!("Telegram HTTP Error: {msg}"),
            } 
        };

        sleep(Duration::from_secs(5)).await;
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum TelegramRequest {
    Me,
    Updates {
        offset: Option<i64>,
        limit: Option<i64>,
        timeout: i64,
    },
}

#[derive(Serialize, Deserialize)]
struct TelegramResponse {
    ok: bool,
    result: Vec<TelegramResult>,
}

#[derive(Serialize, Deserialize)]
struct TelegramResult {
    update_id: i64,
}

async fn telegram(token: &str, request: &TelegramRequest) -> Result<String, reqwest::Error> {
    let (method, body) = match request {
        TelegramRequest::Me => ("getMe", None),
        TelegramRequest::Updates {
            offset: _,
            limit: _,
            timeout: _,
        } => ("getUpdates", Some(serde_json::to_string(request).unwrap())),
    };

    let url = format!("https://api.telegram.org/bot{token}/{method}");

    println!("-- sending request with body {:?}", body);

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

enum TelegramError {
    HttpError { msg: String },
    JsonError { msg: String },
}

async fn telegram_get_messages(
    token: &str,
    offset: Option<i64>,
) -> Result<TelegramResponse, TelegramError> {
    let resp = telegram(
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
    
    println!("Got JSON response {resp}");

    let tresp = serde_json::from_str(&resp);

    match tresp {
        Ok(val) => Ok(val),
        Err(e) => Err(TelegramError::JsonError { msg: e.to_string() }),
    }
}
