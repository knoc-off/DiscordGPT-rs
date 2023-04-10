use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use chrono::{Utc, Duration};

use chatgpt::prelude::*;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

static MY_STRING: &str = "
";

// Define a handler struct
struct Handler {
    chat_gpt_client: ChatGPT,
    conversations: Arc<Mutex<HashMap<u64, Conversation>>>,
    //conversation: Arc<Mutex<Conversation>>,
}


impl Handler {
    async fn new_chatbot(client: ChatGPT) -> Self {
        Handler {
            chat_gpt_client: client,
            conversations: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn chatbot(&self, channel_id: u64, input_str: &String) -> Result<String> {
        let mut conversations = self.conversations.lock().await;
        let conversation = conversations
            .entry(channel_id)
            .or_insert_with(|| self.chat_gpt_client.new_conversation());

        let response = conversation.send_message(input_str).await?;
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

        println!("Recived A Message: {}", msg.content);

        if msg.author.id == bot_user.id {
            return;
        }

        // Check if the message contains the bot's name
        if msg
            .content
            .to_lowercase()
            .contains(&bot_user.name.to_lowercase())
        {

            // Concatenate the messages to form the context string
            // let mut context = MY_STRING.to_owned();
            // println!("Context: {}", context );

            // Reply to the message with a simple text
            let _ = msg
                .channel_id
                .say(&ctx.http, self.chatbot(msg.channel_id.0, &msg.content).await.unwrap())
                .await;
        }
    }
}



#[tokio::main]
async fn main() {
    // Read the bot token from an environment variable
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let chat_gpt_key = std::env::var("OPENAI_API_KEY").expect("Expected a token in the environment");
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
