use std::env;
use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn, Level};
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

    let mut highest_update_id = -1i64;

    loop {
        debug!(highest_update_id, "using highest update id");
        match telegram_client
            .get_messages(Some(highest_update_id).filter(|x| x.is_positive()), None)
            .await
        {
            Ok(r) => {
                for result in r.result {
                    debug!(update_id = result.update_id, "got an update id");
                    highest_update_id = std::cmp::max(highest_update_id, result.update_id + 1);
                    info!(message = result.message.text, "we got a message");
                    let echo_result = telegram_client.send_messages(result.message.chat.id, &result.message.text).await;
                    match echo_result {
                        Ok(_) => info!("we echo'd yeeey"),
                        Err(_) => warn!("echo failed nooooo!"),
                    }
                }
            }
            Err(err) => match err {
                TelegramError::JsonError { msg } => warn!("Telegram JSON Error: {msg}"),
                TelegramError::HttpError { msg } => warn!("Telegram HTTP Error: {msg}"),
            },
        };

        // sleep(Duration::from_secs(5)).await;
    }
}
