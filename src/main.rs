use std::env;
use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender, Receiver};
use tracing::{debug, info, warn, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use telegram::client::Client;
use telegram::{Message, TelegramError};

mod telegram;

#[tokio::main]
async fn main() {
    // Set up tracing/logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let telegram_token = env::var("TELEGRAM_TOKEN")
        .expect("You need to specify a TELEGRAM_TOKEN environment variable");

    let client = Arc::new(Client {
        token: telegram_token,
    });

    let client2 = client.clone();
    let client3 = client.clone();

    let (inbox_tx, mut inbox_rx) = broadcast::channel(16);
    let (outbox_tx, mut outbox_rx) = broadcast::channel(16);

    tokio::spawn(async move {
        telegram_message_receiver(client2, inbox_tx).await;
    });

    tokio::spawn(async move {
        telegram_message_sender(client3, &mut outbox_rx).await;
    });

    // Echo service
    loop {
        let message = inbox_rx.recv().await.unwrap();

        match outbox_tx.send((message.chat.id, message.text)) {
            Ok(_) => {},
            Err(_) => {
                warn!("error sending message to outbox_tx");
            },
        }
    }
}

async fn telegram_message_sender(client: Arc<Client>, rx: &mut Receiver<(u64, String)>) {
    loop {
        if let Ok((chat_id, text)) = rx.recv().await {
            debug!(chat_id, text, "sending message to client");
            match client.send_messages(chat_id, &text).await {
                Ok(_) => {
                    debug!("message sent to telegram");
                }
                Err(_) => {
                    warn!("error sending message to telegram");
                },
            }
        }
    }
}

async fn telegram_message_receiver(client: Arc<Client>, tx: Sender<Message>) {
    let mut highest_update_id = -1i64;

    loop {
        debug!(highest_update_id, "using highest update id");
        match client
            .get_messages(Some(highest_update_id).filter(|x| x.is_positive()), None)
            .await
        {
            Ok(r) => {
                for result in r.result {
                    debug!(update_id = result.update_id, "got an update id");
                    highest_update_id = std::cmp::max(highest_update_id, result.update_id + 1);
                    info!(message = result.message.text, "we got a message");
                    tx.send(result.message).unwrap();
                }
            }
            Err(err) => match err {
                TelegramError::JsonError { msg } => warn!("Telegram JSON Error: {msg}"),
                TelegramError::HttpError { msg } => warn!("Telegram HTTP Error: {msg}"),
            },
        };
    }
}
