use chatgpt::prelude::*;

mod handler;
mod event_handler;
mod sentiment_analysis;
mod preset_selection;

use serenity::Client;
// use handler::Handler;
use event_handler::*;
use crate::handler::Handler;


#[tokio::main]
async fn main() {
    // Read the bot token from an environment variable
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let chat_gpt_key =
        std::env::var("OPENAI_API_KEY").expect("Expected a token in the environment");
    let client = ChatGPT::new(chat_gpt_key).unwrap();

    let handler = Handler::new_chatbot(client).await;

    // Create a client using the bot token and the Handler struct
    let mut client = Client::builder(token)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    // Start the client
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
