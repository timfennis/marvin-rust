use std::env;
use tokio::time::{sleep, Duration};
use tracing::Level;
use tracing_subscriber::{FmtSubscriber,EnvFilter};

mod telegram;

use telegram::TelegramError;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // Set up tracing/logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let telegram_token = env::var("TELEGRAM_TOKEN")
        .expect("You need to specify a TELEGRAM_TOKEN environment variable");

    println!("Starting the marvin server");

    loop {
        match telegram::telegram_get_messages(&telegram_token, None).await {
            Ok(r) => println!("update id: {}", r.result.first().unwrap().update_id),
            Err(err) => match err {
                TelegramError::JsonError { msg } => println!("Telegram JSON Error: {msg}"),
                TelegramError::HttpError { msg } => println!("Telegram HTTP Error: {msg}"),
            },
        };

        sleep(Duration::from_secs(5)).await;
    }
}
