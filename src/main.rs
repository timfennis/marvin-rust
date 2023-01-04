use calendar::fetch_calendar_info;
use chrono::{Local, NaiveTime};
use std::collections::HashSet;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::{debug, info, warn, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use telegram::client::Client;
use telegram::{Message, TelegramError};

mod calendar;
mod telegram;

#[tokio::main]
async fn main() {
    // Set up tracing/logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let default_panic = std::panic::take_hook();

    // if a single task panics during execution just abort the entire program
    std::panic::set_hook(Box::new(move |info| {
        default_panic(info);
        std::process::exit(1);
    }));

    // Let's get some calendar information
    let calendar_url =
        env::var("GARBAGE_URL").expect("You need to specify a GARBAGE_URL environment variable");

    let events = fetch_calendar_info(&calendar_url)
        .await
        .unwrap_or_else(|_| panic!("failed to fetch calendar information"));

    // Setting up telegram tasks
    let db = Arc::new(Mutex::new(HashSet::new()));

    let telegram_token = env::var("TELEGRAM_TOKEN")
        .expect("You need to specify a TELEGRAM_TOKEN environment variable");

    let client = Arc::new(Client {
        token: telegram_token,
    });

    let (inbox_tx, mut inbox_rx) = broadcast::channel(16);
    let (outbox_tx, mut outbox_rx) = broadcast::channel(16);

    // Address book listener
    {
        let mut inbox_rx = inbox_tx.subscribe();
        let db = db.clone();
        tokio::spawn(async move {
            address_book(&mut inbox_rx, db).await.unwrap_or_else(|_| {
                warn!("unable to read from inbox_rx");
                panic!();
            });
        });
    }

    // Schedule notification events
    for event in events {
        let local_notification_time = event
            .date
            .and_time(NaiveTime::from_hms_opt(8, 0, 0).expect("invalid time"))
            .and_local_timezone(chrono_tz::Europe::Amsterdam)
            .unwrap();

        {
            let outbox_tx = outbox_tx.clone();
            let db = db.clone();
            tokio::spawn(async move {
                let seconds_until_notification =
                    local_notification_time.timestamp() - Local::now().timestamp();
                while seconds_until_notification > 0 {
                    let delay = (seconds_until_notification as u64) / 2;
                    info!(
                        delay = delay,
                        "sleeping for half the duration until the notification"
                    );
                    tokio::time::sleep(Duration::from_secs(delay)).await;

                    for contact in db.lock().unwrap().iter() {
                        outbox_tx
                            .send((*contact, "notification !!".to_owned()))
                            .expect("error sending notification");
                    }
                }
            });
        }
    }

    // Inbox receiver/broadcaster
    {
        let client = client.clone();
        tokio::spawn(async move {
            telegram_message_receiver(client, inbox_tx)
                .await
                .unwrap_or_else(|_| {
                    warn!("unable to read from the telegram API");
                    panic!();
                });
        });
    }

    // Outbox sender
    {
        let client = client.clone();
        tokio::spawn(async move {
            telegram_message_sender(client, &mut outbox_rx).await;
        });
    }

    // Phone book print service
    {
        let db = db.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;
                let contacts = db.lock().unwrap();
                debug!(len = contacts.len(), "current contacts");
            }
        });
    }

    // Echo service
    loop {
        let message = inbox_rx.recv().await.unwrap_or_else(|_| {
            warn!("RecvError while reading from inbox_rx in echo service");
            panic!();
        });

        match outbox_tx.send((message.chat.id, message.text)) {
            Ok(_) => {}
            Err(_) => {
                warn!("error sending message to outbox_tx");
            }
        }
    }
}

async fn address_book(
    inbox_rx: &mut Receiver<Message>,
    address_book: Arc<Mutex<HashSet<u64>>>,
) -> Result<(), RecvError> {
    loop {
        let message = inbox_rx.recv().await?;
        address_book.lock().unwrap().insert(message.chat.id);
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
                }
            }
        }
    }
}

async fn telegram_message_receiver(
    client: Arc<Client>,
    tx: Sender<Message>,
) -> Result<(), TelegramError> {
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
            Err(err) => {
                return Err(err);
            }
        };
    }
}
