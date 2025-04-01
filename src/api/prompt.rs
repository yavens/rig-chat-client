use actix_web::{
    post,
    web::{Data, Form},
    Responder, Result,
};

use futures::StreamExt;
use rig::{
    agent::Agent,
    providers::openai::CompletionModel,
};

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tracing::debug;

use crate::state::prompt::PromptState;

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
    let prompt = body.prompt.clone();

    debug!("Responding to \"{}\"", prompt);

    Ok("")
}
