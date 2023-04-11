use chatgpt::prelude::*;
use chrono::{Duration, Utc};
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use vader_sentiment::SentimentIntensityAnalyzer;

// Define a handler struct
struct Handler {
    chat_gpt_client: ChatGPT,
    conversations: Arc<Mutex<HashMap<u64, (Conversation, chrono::DateTime<Utc>)>>>,
}

impl Handler {
    async fn new_chatbot(client: ChatGPT) -> Self {
        Handler {
            chat_gpt_client: client,
            conversations: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn chatbot(&self, channel_id: u64, input_str: &String) -> Result<String> {
        // Lock the conversations HashMap
        let mut conversations = self.conversations.lock().await;

        // Get the current time
        let now = Utc::now();

        // Choose a preset from the static array of presets
        // let preset = PRESETS.choose(&mut rand::thread_rng()).unwrap();

        // Attempt to find an existing conversation for the given channel_id
        // If it doesn't exist, create a new conversation with the chosen preset and store the current timestamp as the last message time
        let conversation_entry = conversations.entry(channel_id).or_insert_with(|| {
            let preset = get_preset_based_on_sentiment(input_str);
            println!(
                "Generating a new conversation for channel {} with preset {}",
                channel_id, preset
            );
            (
                self.chat_gpt_client.new_conversation_directed(preset),
                Utc::now(),
            )
        });

        // Check if the conversation's last message time is older than 10 minutes
        // If it is, recreate the conversation with the chosen preset and update the last message time to the current time
        if now.signed_duration_since(conversation_entry.1) > Duration::minutes(2) {
            let preset = get_preset_based_on_sentiment(input_str);
            println!(
                "Refreshing the conversation for channel {} with preset {}",
                channel_id, preset
            );
            *conversation_entry = (
                self.chat_gpt_client.new_conversation_directed(preset),
                Utc::now(),
            );
        }
        //else {
        //    println!("Using an existing conversation for channel {}", channel_id);
        //}

        // Send the user's message to the conversation and receive a response
        let response = conversation_entry.0.send_message(input_str).await?;

        // Update the conversation's last message time to the current time
        conversation_entry.1 = Utc::now();

        // Return the response content as a String
        Ok(response.message().content.to_string())
    }
}

// Implement EventHandler trait for the Handler struct
#[async_trait]
impl EventHandler for Handler {
    // This function will be called when the bot is ready to start
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    // This function will be called when a message is received
    async fn message(&self, ctx: Context, msg: Message) {
        let bot_user = ctx.http.get_current_user().await.unwrap();

        if msg.author.id == bot_user.id {
            return;
        }

        println!("\nRecived A Message: {}", msg.content);

        let channel_id = msg.channel_id.0;
        let should_respond = {
            let conversations = self.conversations.lock().await;
            if let Some((_, last_message)) = conversations.get(&channel_id) {
                msg.content
                    .to_lowercase()
                    .contains(&bot_user.name.to_lowercase())
                    || Utc::now().signed_duration_since(*last_message) <= Duration::minutes(30)
            } else {
                msg.content
                    .to_lowercase()
                    .contains(&bot_user.name.to_lowercase())
            }
        };

        // Check if the message contains the bot's name or was sent within 1 minute of the last conversation message in the channel
        if should_respond {
            let response = self
                .chatbot(
                    msg.channel_id.0,
                    &(msg.author.name.to_owned() + ": " + &msg.content),
                )
                .await
                .unwrap();
            println!("Response: {}", response);
            // Reply to the message with a simple text
            let _ = msg.channel_id.say(&ctx.http, response).await;
        }
    }
}

fn get_preset_based_on_sentiment(message: &str) -> String {
    let analyzer = SentimentIntensityAnalyzer::new();
    let sentiment = analyzer.polarity_scores(message);

    let sentiment_score = sentiment.get("compound").unwrap_or(&0.0);
    println!("Sentiment score: {}", sentiment_score);

    let presets = [
        (
            0.75, // this is the sentiment score, this is tied to the message
            "you are a chatbot, be very positive, happy and try to respond in as few words as possible",
        ),
        (
            0.0,
            "you are a chatbot, be neutral, apethetic and try to respond in as few words as possible",
        ),
        (
            -0.75,
            "you are a chatbot, be very negative, angry and try to respond in as few words as possible",
        ),
    ];

    let closest_index = presets.iter().enumerate().fold(0, |acc, (index, &(sentiment, _prompt))| {
        let distance = (sentiment_score - sentiment).abs();
        let closest_distance = (sentiment_score - presets[acc].0).abs();

        if distance < closest_distance {
            index
        } else {
            acc
        }
    });

    let final_preset = format!(
        "The expected format is as follows:\n<name>: <message>\nyou should only ever respond with <message>\n{}",
        presets[closest_index].1
    );

    final_preset
}


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
