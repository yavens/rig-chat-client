use std::{fs::{self, File}, io::Write, time::SystemTime};

use rig::{
    completion::ToolDefinition,
    image_generation::{ImageGenerationError, ImageGenerationModel},
    providers::openai,
    tool::Tool,
};
use serde::Deserialize;
use serde_json::json;

pub struct GenerateImage {}
#[derive(Deserialize)]
pub struct GenerateImageArgs {
    prompt: String,
}

impl Tool for GenerateImage {
    const NAME: &'static str = "generate_image";

    type Error = ImageGenerationError;

    type Args = GenerateImageArgs;

    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Generate an image for the user".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "prompt": {
                        "type": "string",
                        "description": "The description of the image you want to generate"
                    }
                }
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let openai = openai::Client::from_env().image_generation_model(openai::DALL_E_2);

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Couldn't get time")
            .as_millis();

        if !fs::exists("static/temp/images").expect("Can't confirm existence") {
            let _ = fs::create_dir_all("static/temp/images");
        }

        let path = format!("static/temp/images/{now}.png");

        let mut file = File::options()
            .write(true)
            .create(true)
            .open(&path)
            .expect("Couldn't open file");

        let response = openai
            .image_generation_request()
            .prompt(&args.prompt)
            .height(256)
            .width(256)
            .send()
            .await?;

        let _ = file.write(&response.image);

        Ok(format!("/{path}"))
    }
}
