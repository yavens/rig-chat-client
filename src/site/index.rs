use std::sync::Mutex;

use actix_web::{get, web::Data, Responder, Result};

use crate::{state::prompt::PromptState, templates::{ChatHistoryTemplate, IndexTemplate}};


#[get("/")]
pub async fn get(state: Data<Mutex<PromptState>>) -> Result<impl Responder> {
    let state = state.get_ref().lock().unwrap();

    Ok(IndexTemplate {
        chat_history: ChatHistoryTemplate {
            messages: state.messages.clone(),
        },
    })
}
