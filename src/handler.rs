use chatgpt::prelude::*;
use chrono::{Duration, Utc};
use serenity::prelude::*;
use std::{collections::HashMap, sync::Arc};
use tokio::{sync::mpsc, sync::Mutex};


use serenity::model::prelude::ChannelId;

use crate::sentiment_analysis::get_preset_based_on_sentiment;

use crate::preset_selection::{get_pre_prompt, get_sentiment_appropriate_response};

pub struct QueuedMessage {
    pub channel_id: u64,
    pub author_name: String,
    pub content: String,
}

pub struct Handler {
    pub chat_gpt_client: ChatGPT,
    pub conversations: Arc<Mutex<HashMap<u64, (Conversation, chrono::DateTime<Utc>)>>>,
    pub sender: mpsc::Sender<QueuedMessage>,
    pub receiver: Arc<Mutex<mpsc::Receiver<QueuedMessage>>>,
}

impl Clone for Handler {
    fn clone(&self) -> Self {
        Self {
            chat_gpt_client: self.chat_gpt_client.clone(),
            conversations: self.conversations.clone(),
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}

impl Handler {
    pub async fn new_chatbot(client: ChatGPT) -> Self {
        let (sender, receiver) = mpsc::channel(100);
        Handler {
            chat_gpt_client: client,
            conversations: Arc::new(Mutex::new(HashMap::new())),
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub async fn queue_handler(self: Arc<Self>, ctx: Context) {
        loop {
            let queued_message = {
                let mut receiver = self.receiver.lock().await;
                receiver.recv().await
            };

            if let Some(queued_message) = queued_message {
                //let msg = &queued_message.message;
                let response = self
                    .chatbot(
                        queued_message.channel_id,
                        &(queued_message.author_name + ": " + &queued_message.content),
                    )
                    .await
                    .unwrap(); // This Errors when the token length is exceeded

                println!("Response: {}", response);

                let _ = ChannelId(queued_message.channel_id)
                    .say(&ctx.http, response)
                    .await;
            }

            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }

    pub async fn chatbot(&self, channel_id: u64, input_str: &String) -> Result<String> {
        // Lock the conversations HashMap
        let mut conversations = self.conversations.lock().await;

        // Get the current time
        let now = Utc::now();

        // Attempt to find an existing conversation for the given channel_id
        // If it doesn't exist, create a new conversation with the chosen preset and store the current timestamp as the last message time
        let conversation_entry = conversations.entry(channel_id).or_insert_with(|| {
            let preset = get_preset_based_on_sentiment(input_str);
            println!(
                "Generating a new conversation for channel: {}, with preset: {}",
                channel_id, preset
            );
            (
                self.chat_gpt_client.new_conversation_directed(preset),
                Utc::now(),
            )
        });

        // Check if the conversation's last message time is older than 10 minutes
        // If it is, recreate the conversation with the chosen preset and update the last message time to the current time
        if now.signed_duration_since(conversation_entry.1) > Duration::minutes(5)
            || input_str.contains("!reset!")
        {
            let preset = get_preset_based_on_sentiment(input_str);
            println!(
                "Refreshing the conversation for channel: {}, with preset: {}",
                channel_id, preset
            );
            *conversation_entry = (
                self.chat_gpt_client.new_conversation_directed(preset),
                Utc::now(),
            );
        }

        // Send the user's message to the conversation and receive a response
        let response = conversation_entry.0.send_message(input_str).await?;

        // Update the conversation's last message time to the current time
        conversation_entry.1 = Utc::now();

        // Return the response content as a String
        Ok(response.message().content.to_string())
    }
}
