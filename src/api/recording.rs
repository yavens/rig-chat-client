use actix_web::{get, post, web::Form, Responder, Result};
use askama::Template;
use serde::{Deserialize, Serialize};

#[get("/api/recording")]
pub async fn get() -> Result<impl Responder> {
    Ok(Recording {})
}

#[derive(Deserialize, Serialize)]
struct PostParams {
    data: String,
}

#[post("/api/recording")]
pub async fn post(body: Form<PostParams>) -> Result<impl Responder> {
    Ok(FilledPrompt { prompt: "".to_string() })
}

#[derive(Template)]
#[template(path = "recording.html")]
struct Recording {}

#[derive(Template)]
#[template(path = "prompt.html")]
struct FilledPrompt {
    prompt: String,
}
