use std::sync::Mutex;

use actix_web::{get, web::Data, Responder, Result};
use askama_actix::Template;

use crate::{state::prompt::PromptState, templates::ChatHistoryTemplate};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    chat_history: ChatHistoryTemplate,
}

#[get("/")]
pub async fn get(state: Data<Mutex<PromptState>>) -> Result<impl Responder> {
    let state = state.get_ref().lock().unwrap();

    Ok(IndexTemplate {
        chat_history: ChatHistoryTemplate {
            messages: state.messages.clone(),
        },
    })
}
