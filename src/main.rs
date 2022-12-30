use std::env;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use telegram::client::Client;
use telegram::TelegramError;

mod telegram;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // Set up tracing/logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let telegram_token = env::var("TELEGRAM_TOKEN")
        .expect("You need to specify a TELEGRAM_TOKEN environment variable");

    info!("Starting the marvin server");

    let telegram_client = Client {
        token: telegram_token,
    };

    loop {
        match telegram_client.get_messages(None).await {
            Ok(r) => println!("update id: {}", r.result.first().unwrap().update_id),
            Err(err) => match err {
                TelegramError::JsonError { msg } => warn!("Telegram JSON Error: {msg}"),
                TelegramError::HttpError { msg } => warn!("Telegram HTTP Error: {msg}"),
            },
        };

        sleep(Duration::from_secs(5)).await;
    }
}
