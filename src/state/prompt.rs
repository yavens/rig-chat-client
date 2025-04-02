use std::time::SystemTime;

use actix_web_lab::sse;
use askama::Template;
use rig::completion::Message;
use tokio::sync::mpsc::Sender;

use crate::templates::{ChatHistoryTemplate, MessageTemplate};

#[derive(Default)]
pub struct PromptState {
    pub messages: Vec<Message>,
    pub tx: Option<Sender<sse::Event>>,
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
        todo!()
    }

    pub async fn send_audio(&self, generation_time: SystemTime, data: &[u8]) {
        todo!()
    }

    async fn send(&self, event: sse::Event) {
        if self.tx.is_none() {
            return;
        }

        let _ = self.tx.as_ref().unwrap().send(event).await;
    }
}
