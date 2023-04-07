




use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use chrono::{Utc, Duration};

use chatgpt::prelude::*;

static MY_STRING: &str = "
";

// Define a handler struct
struct Handler;





async fn chatbot(input_str: &String, context: &String) -> Result<String> {
    let key = std::env::var("OPENAI_API_KEY").expect("Expected a token in the environment");
    let client = ChatGPT::new(key)?;
    /// Sending a message and getting the completion
    let response = client.send_message(context.to_owned() + input_str).await?;
    Ok(response.message().content.to_string())
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
            // Do not process the message if the author is the bot
            return;
        }

        // Check if the message contains the bot's name
        if msg
            .content
            .to_lowercase()
            .contains(&bot_user.name.to_lowercase())
        {
            let mut message_history = msg
                .channel_id
                .messages(&ctx.http, |builder| builder.before(msg.id).limit(5))
                .await
                .unwrap();

            // Reverse the order of the messages to maintain chronological order
            message_history.reverse();

            // Get the current time and set the time limit to 1 hour ago
            let now = Utc::now();
            let time_limit = now - Duration::hours(1);

            // Filter the messages based on the time limit and take at most 5 messages
            let messages: Vec<_> = message_history
                .into_iter()
                .filter(|message| message.timestamp > time_limit)
                .take(5)
                .collect();

            // Concatenate the messages to form the context string
            let mut context = MY_STRING.to_owned();
            for message in &messages {
                context.push_str(&format!("- {}: {}\n", message.author.name, message.content));
            }
            println!("Context: {}", context );

            // Reply to the message with a simple text
            let _ = msg
                .channel_id
                .say(&ctx.http, chatbot(&msg.content, &context).await.unwrap())
                .await;
        }
    }
}

#[tokio::main]
async fn main() {
    // Read the bot token from an environment variable
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Create a client using the bot token and the Handler struct
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // Start the client
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
