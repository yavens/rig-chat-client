use actix_web::{get, post, web::Form, Responder, Result};
use askama::Template;

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
    Ok(FilledPromptTemplate {
        prompt: "".to_string(),
    })
}
