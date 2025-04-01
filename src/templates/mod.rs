/// This module defines the various HTML templates used around the demo.

use askama::Template;
use askama_markdown_cmark::filters;
use rig::message::{AssistantContent, Message, UserContent};

/// The full conversation history
#[derive(Template)] 
#[template(path = "chat_history.html")]
pub struct ChatHistoryTemplate {
    /// The name of the struct can be anything
    pub messages: Vec<Message>,
}

/// A message shown in chat history
#[derive(Template)]
#[template(path = "message.html")]
pub struct MessageTemplate {
    /// The index of the message, used for the SSE message name
    pub index: usize,
    /// The message itself
    pub message: Message,
}

/// The recording prompt
#[derive(Template)]
#[template(path = "recording.html")]
pub struct RecordingTemplate {}

/// The normal prompt but with a default value
#[derive(Template)]
#[template(path = "prompt.html")]
pub struct FilledPromptTemplate {
    /// The default prompt value
    pub prompt: String,
}

/// The whole site index
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    /// The chat history at the time of request
    pub chat_history: ChatHistoryTemplate,
}