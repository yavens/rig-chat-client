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
    streaming::{StreamingChat, StreamingCompletion},
    tool::Tool,
};

use serde::{Deserialize, Serialize};
use std::{sync::Mutex, time::Instant};
use tracing::debug;

use crate::{state::prompt::PromptState, tools::GenerateImage};

#[derive(Serialize, Deserialize)]
struct PromptParams {
    prompt: String,
}

async fn handle_tool_call(
    agent: &Data<Agent<CompletionModel>>,
    state: Data<Mutex<PromptState>>,
    name: String,
    args: serde_json::Value,
) {
    if name == GenerateImage::NAME {
        let agent = agent.clone();

        // Handle image generation non-blocking
        actix_web::rt::spawn(async move {
            let mut state = state.lock().unwrap();
            let image = state.send_message(Message::assistant("")).await;

            let response = agent.tools.call(&name, args.to_string()).await;

            if let Ok(data_uri) = response {
                let _ = state
                    .update_message(
                        image,
                        format!(
                            r#"<img src="{}"/>"#,
                            serde_json::from_str::<String>(&data_uri)
                                .expect("Failed to convert data uri")
                        ),
                    )
                    .await;
            }
        });
    }
}

async fn send_buffer(state: Data<Mutex<PromptState>>, index: usize, buffer: &mut Vec<String>) {
    let new_message = buffer.join("");
    buffer.clear();

    let _ = state
        .lock()
        .unwrap()
        .update_message(index, new_message)
        .await;
}

async fn stream_response(
    prompt: String,
    agent: Data<Agent<CompletionModel>>,
    state_mutex: Data<Mutex<PromptState>>,
) {
    let mut state = state_mutex.lock().unwrap();

    let _ = state.send_message(Message::user(prompt.clone())).await;

    let mut response = agent
        .stream_chat(&prompt, state.messages.clone())
        .await
        .expect("Failed to create stream");

    // Make sure to drop our lock on the Mutex so audio generation has access
    // to the state transmitter
    drop(state);

    let mut token_buffer = vec![];

    // The last time a token was sent to the client
    let mut last_send = Instant::now();

    // Whether or not a new message should be created before updating the contents
    let mut create_new_message = true;

    // The message to send an update too
    let mut message_index = 0;

    while let Some(chunk) = response.next().await {
        let Ok(choice) = chunk else {
            continue;
        };

        let piece = match choice {
            rig::streaming::StreamingChoice::Message(message) => message,
            rig::streaming::StreamingChoice::ToolCall(name, _description, args) => {
                // Process tool calls and move to the next chunk
                handle_tool_call(&agent, state_mutex.clone(), name, args).await;
                continue;
            }
        };

        // Create an empty message to start adding data to
        if create_new_message {
            let mut state = state_mutex.lock().unwrap();
            message_index = state.send_message(Message::assistant("")).await;
            create_new_message = false;
            last_send = Instant::now();
        }

        token_buffer.push(piece.clone());

        let now = Instant::now();

        // Ensure that tokens are sent with a delay so SSE doesn't miss events
        if now.duration_since(last_send).as_millis() > 5 {
            let _ = send_buffer(state_mutex.clone(), message_index, &mut token_buffer).await;
            last_send = now;
        }
    }

    if token_buffer.len() >= 1 {
        let _ = send_buffer(state_mutex.clone(), message_index, &mut token_buffer).await;
    }

    let state = state_mutex.lock().unwrap();
    let _ = state.replace_chat_history().await;
}

#[post("/api/prompt")]
pub async fn post(
    body: Form<PromptParams>,
    agent: Data<Agent<CompletionModel>>,
    state_mutex: Data<Mutex<PromptState>>,
) -> Result<impl Responder> {
    let prompt = body.prompt.clone();

    debug!("Responding to \"{}\"", prompt);

    actix_web::rt::spawn(stream_response(prompt, agent, state_mutex));

    Ok("")
}
