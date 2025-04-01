use askama::Template;
use askama_markdown_cmark::filters;
use rig::message::{AssistantContent, Message, UserContent};

#[derive(Template)] // this will generate the code...
#[template(path = "chat_history.html")] // using the template in this path, relative
                                        // to the `templates` dir in the crate root
pub struct ChatHistoryTemplate {
    // the name of the struct can be anything
    pub messages: Vec<Message>,
}

#[derive(Template)]
#[template(path = "message.html")]
pub struct MessageTemplate {
    pub index: usize,
    pub message: Message,
}
