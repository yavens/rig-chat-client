use actix_web::{get, post, web::Form, Responder, Result};

use rig::{providers::openai, transcription::TranscriptionModel};
use serde::{Deserialize, Serialize};

use crate::templates::{FilledPromptTemplate, RecordingTemplate};

#[get("/api/recording")]
pub async fn get() -> Result<impl Responder> {
    Ok(RecordingTemplate {})
}

#[derive(Deserialize, Serialize)]
struct PostParams {
    data: String,
}

#[post("/api/recording")]
pub async fn post(body: Form<PostParams>) -> Result<impl Responder> {
    let transcription = openai::Client::from_env().transcription_model(openai::WHISPER_1);

    let Ok(data) = serde_json::from_str::<Vec<u8>>(&body.data) else {
        panic!("Failed to decode data!")
    };

    let response = transcription
        .transcription_request()
        .data(data)
        .filename(Some("audio.mp3".to_string()))
        .send()
        .await
        .expect("Failed to transcribe");

    let response = response.text;

    Ok(FilledPromptTemplate { prompt: response })
}
