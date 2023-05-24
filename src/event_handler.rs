use chrono::Duration;
use chrono::Utc;
use serenity::async_trait;
use serenity::client::Context;
use serenity::client::EventHandler;
use serenity::model::prelude::*;
use std::sync::Arc;

use crate::handler::QueuedMessage;

// Implement EventHandler trait for the Handler struct
#[async_trait]
impl EventHandler for crate::handler::Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let handler_clone = Arc::new(self.clone());
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            handler_clone.queue_handler(ctx_clone).await;
        });
    }

    // This function will be called when a message is received
    async fn message(&self, ctx: Context, msg: Message) {
        let bot_user = ctx.http.get_current_user().await.expect(" failed to get user event_handler.rs");

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
                    || Utc::now().signed_duration_since(*last_message) <= Duration::seconds(30)
            } else {
                msg.content
                    .to_lowercase()
                    .contains(&bot_user.name.to_lowercase())
            }
        };

        // Check if the message contains the bot's name or was sent within 1 minute of the last conversation message in the channel
        if should_respond {
            let queued_message = QueuedMessage {
                channel_id: msg.channel_id.0,
                author_name: msg.author.name.clone(),
                content: msg.content.clone(),
            };

            if let Err(e) = self.sender.send(queued_message).await {
                eprintln!("Failed to send message to the queue: {}", e);
            }
        }
    }
}
