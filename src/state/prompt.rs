use std::time::SystemTime;

use actix_web_lab::sse;
use askama::Template;
use base64::{prelude::BASE64_STANDARD, Engine};
use rig::completion::Message;
use serde::Serialize;
use tokio::sync::mpsc::Sender;

use crate::templates::{ChatHistoryTemplate, MessageTemplate};

#[derive(Default)]
pub struct PromptState {
    pub messages: Vec<Message>,
    pub tx: Option<Sender<sse::Event>>,
}

#[derive(Serialize)]
struct PlayAudioData {
    data_uri: String,
    generation_time: u128,
}

impl PromptState {
    pub async fn replace_chat_history(&self) {
        let _ = self
            .send(
                sse::Data::new(
                    ChatHistoryTemplate {
                        messages: self.messages.clone(),
                    }
                    .render()
                    .expect("Failed to render chat history"),
                )
                .event("chat_history")
                .into(),
            )
            .await;
    }

    pub async fn send_message(&mut self, message: Message) -> usize {
        let index = self.messages.len();
        self.messages.push(message.clone());

        let data = MessageTemplate { index, message }
            .render()
            .expect("Failed to render message");

        let _ = self
            .send(sse::Data::new(data).event("new_message").into())
            .await;

        index
    }

    pub async fn update_message(&mut self, index: usize, chunk: String) {
        let message = &self.messages[index];

        let new_message = match message {
            Message::User { content } => match content.first() {
                rig::message::UserContent::Text(text) => {
                    Message::user(format!("{}{}", text.text, chunk))
                }
                _ => panic!("Can't update non-text message"),
            },
            Message::Assistant { content } => match content.first() {
                rig::message::AssistantContent::Text(text) => {
                    Message::assistant(format!("{}{}", text.text, chunk))
                }
                _ => panic!("Can't update non-text message"),
            },
        };

        self.messages[index] = new_message;

        let _ = self
            .send(
                sse::Data::new(format!(r#"<span class="text">{chunk}</span>"#))
                    .event(format!("update_message#{}", index))
                    .into(),
            )
            .await;
    }

    pub async fn queue_audio(&self, generation_time: SystemTime) {
        let _ = self
            .send(
                sse::Data::new(
                    generation_time
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .expect("Failed to get duration since EPOCH")
                        .as_millis()
                        .to_string(),
                )
                .event("queue_audio")
                .into(),
            )
            .await;
    }

    pub async fn send_audio(&self, generation_time: SystemTime, data: &[u8]) {
        let base64 = BASE64_STANDARD.encode(data);
        let data_uri = format!("data:audio/mp3;base64,{}", base64);

        let data = PlayAudioData {
            data_uri,
            generation_time: generation_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Failed to get duration since EPOCH")
                .as_millis(),
        };

        let json = serde_json::to_string(&data).expect("Failed to serialize data");

        let _ = self
            .send(sse::Data::new(json).event("play_audio").into())
            .await;
    }

    async fn send(&self, event: sse::Event) {
        if self.tx.is_none() {
            return;
        }

        let _ = self.tx.as_ref().unwrap().send(event).await;
    }
}
