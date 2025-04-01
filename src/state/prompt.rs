use std::time::SystemTime;

use actix_web_lab::sse;
use rig::completion::Message;
use tokio::sync::mpsc::Sender;


#[derive(Default)]
pub struct PromptState {
    pub messages: Vec<Message>,
    pub tx: Option<Sender<sse::Event>>,
}

impl PromptState {
    pub async fn replace_chat_history(&self) {
        todo!()
    }

    pub async fn send_message(&mut self, message: Message) -> usize {
        todo!()
    }

    pub async fn update_message(&mut self, index: usize, chunk: String) {
        todo!()
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
