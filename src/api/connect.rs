use std::{sync::Mutex, time::Duration};

use actix_web::{get, web::Data, Responder};
use actix_web_lab::sse;

use crate::state::prompt::PromptState;


#[get("/api/connect")]
pub async fn get(state_mutex: Data<Mutex<PromptState>>) -> impl Responder {
    let (tx, rx) = tokio::sync::mpsc::channel(10);

    let mut state = state_mutex.lock().expect("Could not acquire lock");

    (*state).tx = Some(tx);

    sse::Sse::from_infallible_receiver(rx).with_retry_duration(Duration::from_secs(10))
}
