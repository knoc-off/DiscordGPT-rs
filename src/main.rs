use chatgpt::prelude::*;

mod event_handler;
mod handler;
mod preset_selection;
mod sentiment_analysis;

use serenity::Client;
// use handler::Handler;

#[tokio::main]
async fn main() {
    // Read the bot discord from an environment variable
    let discord = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let chatgpt = std::env::var("OPENAI_API_KEY").expect("Expected a discord in the environment");

    // Instantiating a new ChatGPT client using the provided chatgpt model
    // Creating a new Handler object that uses the ChatGPT client
    let client = ChatGPT::new(chatgpt).unwrap();
    let handler = handler::Handler::new_chatbot(client).await;

    // Create a client using the bot discord and the Handler struct
    let mut client = Client::builder(discord)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    // Start the client
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
