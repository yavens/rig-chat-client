use std::{env, sync::Mutex};

use actix_files::Files;
use actix_web::{
    middleware,
    web::{Data, FormConfig, PayloadConfig},
    App, HttpServer,
};
use dotenv::dotenv;
use rig::providers::openai;
use state::prompt::PromptState;
use tools::GenerateImage;

mod api;
mod site;
pub mod state;
pub mod templates;
pub mod tools;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let port = if let Ok(port) = env::var("PORT") {
        port.parse::<u16>().unwrap_or(8080)
    } else {
        8080
    };

    let history: Data<Mutex<PromptState>> = Data::new(Mutex::new(PromptState::default()));

    let server = HttpServer::new(move || {
        let agent = openai::Client::from_env()
            .agent(openai::GPT_4O) // Any text completion model can be used here thanks to Rig’s structure
            .tool(GenerateImage {})
            .build();

        let agent = Data::new(agent);

        App::new()
            .wrap(middleware::Logger::default())
            .app_data(agent)
            .app_data(Data::clone(&history))
            .app_data(PayloadConfig::new(1000000 * 250))
            .app_data(FormConfig::default().limit(1000000 * 250))
            .service(Files::new("/static", "./static"))
            // Site Routes
            .service(site::index::get)
            // Api Routes
            .service(api::connect::get)
            .service(api::prompt::post)
    });

    server.bind(("0.0.0.0", port))?.run().await
}
