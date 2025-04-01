use actix_web::{
    error, post,
    web::{Data, Form},
    Responder, Result,
};

use futures::StreamExt;
use rig::{
    agent::Agent,
    completion::{Chat, Completion, Prompt},
    message::Message,
    providers::openai::CompletionModel,
};

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tracing::debug;

use crate::{state::prompt::PromptState, templates::MessageTemplate};

#[derive(Serialize, Deserialize)]
struct PromptParams {
    prompt: String,
}

#[post("/api/prompt")]
pub async fn post(
    body: Form<PromptParams>,
    agent: Data<Agent<CompletionModel>>,
    state_mutex: Data<Mutex<PromptState>>,
) -> Result<impl Responder> {
    let mut state = state_mutex.lock().expect("Failed to acquire lock");
    let prompt = body.prompt.clone();

    debug!("Responding to \"{}\"", prompt);

    let response = agent.chat(prompt.clone(), state.messages.clone()).await;

    let Ok(response) = response else {
        return Err(error::ErrorInternalServerError("Failed to get response"));
    };

    (*state).messages.push(Message::user(&prompt));
    (*state).messages.push(Message::assistant(&response));

    Ok(format!(
        "{}{}",
        MessageTemplate {
            index: 0,
            message: Message::user(&prompt)
        },
        MessageTemplate {
            index: 0,
            message: Message::assistant(response),
        },
    ))
}
