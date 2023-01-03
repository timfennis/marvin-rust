use serde::{Deserialize, Serialize};

pub mod client;

pub enum TelegramError {
    HttpError { msg: String },
    JsonError { msg: String },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum TelegramRequest {
    Me,
    Updates {
        offset: Option<i64>,
        limit: Option<i64>,
        timeout: u32,
    },
    SendMessage {
        chat_id: u64,
        text: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseContainer<T> {
    pub ok: bool,
    pub result: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Update {
    pub update_id: i64,
    pub message: Message,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub text: String,
    pub chat: Chat,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chat {
    pub id: u64,
    pub first_name: String,
    pub last_name: String,
    #[serde(alias = "type")]
    pub chat_type: String,
}

type GetUpdatesResponse = ResponseContainer<Vec<Update>>;
type SendMessageResponse = ResponseContainer<Message>;
