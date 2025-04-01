use actix_web::{
    post,
    web::{Data, Form},
    Responder, Result,
};

use futures::StreamExt;
use rig::{
    agent::Agent,
    audio_generation::AudioGenerationModel,
    completion::Message,
    providers::openai::{self, CompletionModel},
    streaming::StreamingCompletion,
    tool::Tool,
};
use serde::{Deserialize, Serialize};
use std::{
    sync::Mutex,
    time::{Instant, SystemTime},
};
use tracing::debug;

use crate::{state::prompt::PromptState, tools::GenerateImage};

#[derive(Serialize, Deserialize)]
struct PromptParams {
    prompt: String,
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

async fn stream_audio(state_mutex: Data<Mutex<PromptState>>, text: String) {
    debug!("Generating audio for {text}");

    let openai = openai::Client::from_env();
    let tts = openai.audio_generation_model(openai::TTS_1);

    let generation_time = SystemTime::now();

    // Tell the client to expect an audio segment
    let _ = state_mutex
        .lock()
        .unwrap()
        .queue_audio(generation_time)
        .await;

    // Generate the audio segment
    let data = tts
        .audio_generation_request()
        .voice("alloy")
        .text(&text)
        .send()
        .await
        .expect("Failed to generate audio");

    debug!(
        "Took {}s to generate",
        SystemTime::now()
            .duration_since(generation_time)
            .unwrap()
            .as_secs_f32()
    );

    // Send the audio segment to the client.
    let _ = state_mutex
        .lock()
        .unwrap()
        .send_audio(generation_time, &data.audio)
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
        .stream_completion(prompt.clone(), state.messages.clone())
        .await
        .expect("Failed to create request builder")
        .stream()
        .await
        .expect("Failed to create stream");

    // Make sure to drop our lock on the Mutex so audio generation has access
    // to the state transmitter
    drop(state);

    let mut token_buffer = vec![];

    // Minimum number of sentences before creating an audio generation request
    let mut min_sentence_count = 1;
    let mut sentence_count = 0;

    // Tokens of sentence
    let mut sentence = vec![];

    // The last time a token was sent to the client
    let mut last_send = Instant::now();

    // Whether or not a new message should be created before updating the contents
    let mut create_new_message = true;

    // The message to send an update too
    let mut message_index = 0;

    while let Some(chunk) = response.next().await {
        if chunk.is_err() {
            continue;
        }

        let choice = chunk.unwrap();

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
        sentence.push(piece.clone());

        if piece.ends_with(".") || piece.ends_with("!") || piece.ends_with("?") {
            sentence_count += 1;

            // No need to send every sentence individually;
            // the client needs time to read so we can incrementally
            // increase the batch size we send to TTS
            if sentence_count >= min_sentence_count {
                sentence_count = 0;
                min_sentence_count += 1;

                let text = sentence.join("");
                sentence.clear();

                // Create an audio generation request without pausing the stream
                actix_web::rt::spawn(stream_audio(state_mutex.clone(), text));
            }
        }

        let now = Instant::now();

        // Ensure that tokens are sent with a delay so SSE doesn't miss events
        if token_buffer.len() >= 1 && now.duration_since(last_send).as_millis() > 5 {
            let _ = send_buffer(state_mutex.clone(), message_index, &mut token_buffer).await;
            last_send = now;
        }
    }

    // Clean up remaining tokens in both buffers
    if sentence.len() >= 1 {
        let text = sentence.join("");
        actix_web::rt::spawn(stream_audio(state_mutex.clone(), text));
    }

    if token_buffer.len() >= 1 {
        let _ = send_buffer(state_mutex.clone(), message_index, &mut token_buffer).await;
    }

    let state = state_mutex.lock().unwrap();
    let _ = state.replace_chat_history().await;
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
