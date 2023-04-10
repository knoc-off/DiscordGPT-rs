use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use chrono::{Utc, Duration};

use chatgpt::prelude::*;

use std::sync::Arc;
use tokio::sync::Mutex;

static MY_STRING: &str = "
";

// Define a handler struct
struct Handler {
    conversation: Arc<Mutex<Conversation>>,
}

impl Handler {
    async fn new_chatbot(client: ChatGPT) -> Self {
        let conversation = client.new_conversation();
        Handler {
            conversation: Arc::new(Mutex::new(conversation)),
        }
    }

    async fn chatbot(&self, input_str: &String) -> Result<String> {
        let mut conversation = self.conversation.lock().await;
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
                .say(&ctx.http, self.chatbot(&msg.content).await.unwrap())
                .await;
        }
    }
}

// async fn chatbot(input_str: &String, context: &String) -> Result<String> {
//     let key = std::env::var("OPENAI_API_KEY").expect("Expected a token in the environment");
//     let client = ChatGPT::new(key)?;
//     /// Sending a message and getting the completion
//     let response = client.send_message(context.to_owned() + input_str).await?;
//     Ok(response.message().content.to_string())
// }



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
