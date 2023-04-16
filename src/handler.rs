use chatgpt::prelude::*;

use chrono::{Duration, Utc};
use serenity::prelude::*;
use std::{collections::HashMap, sync::Arc};
use tokio::{sync::mpsc, sync::Mutex};

use serenity::model::prelude::ChannelId;

use crate::sentiment_analysis::get_preset_based_on_sentiment;

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
            if let Some(queued_message) = self.receive_message().await {
                let response_result = self.chatbot_response(&queued_message).await;

                match response_result {
                    Ok(response) => {
                        self.send_response(ctx.http.clone(), &queued_message, response)
                            .await
                    }
                    Err(e) => self.handle_error(e, &queued_message).await,
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }

    async fn receive_message(&self) -> Option<QueuedMessage> {
        let mut receiver = self.receiver.lock().await;
        receiver.recv().await
    }

    async fn chatbot_response(&self, queued_message: &QueuedMessage) -> Result<String> {
        let message_text = queued_message.author_name.clone() + ": " + &queued_message.content;
        self.chatbot(queued_message.channel_id, &message_text).await
    }

    async fn send_response(
        &self,
        http: Arc<serenity::http::Http>,
        queued_message: &QueuedMessage,
        response: String,
    ) {
        println!("Response: {}", response);

        let _ = ChannelId(queued_message.channel_id)
            .send_message(&http, |m| {
                m.content(response);
                m.tts(true)
            })
            .await;
    }

    async fn handle_error(&self, error: chatgpt::err::Error, queued_message: &QueuedMessage) {
        eprintln!("Error: {}", error);

        let mut conversations = self.conversations.lock().await;

        println!("Attempting to Reset");

        if let Some(conversation_entry) = conversations.get_mut(&queued_message.channel_id) {
            self.handle_reset(conversation_entry, 10)
        }
    }

    pub async fn chatbot(&self, channel_id: u64, input_str: &String) -> Result<String> {
        // Lock the conversations HashMap
        let mut conversations = self.conversations.lock().await;

        let conversation_entry = self
            .get_or_create_conversation(&mut conversations, channel_id, input_str)
            .await;

        self.handle_reset(conversation_entry, 10);

        // Send the user's message to the conversation and receive a response
        let response = conversation_entry.0.send_message(input_str).await?;

        // Update the conversation's last message time to the current time
        conversation_entry.1 = Utc::now();

        // Return the response content as a String
        Ok(response.message().content.to_string())
    }

    async fn get_or_create_conversation<'a>(
        &'a self,
        conversations: &'a mut HashMap<u64, (Conversation, chrono::DateTime<Utc>)>,
        channel_id: u64,
        input_str: &String,
    ) -> &mut (Conversation, chrono::DateTime<Utc>) {
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
        if now.signed_duration_since(conversation_entry.1) > Duration::minutes(5) {
            self.refresh_conversation(conversation_entry, &input_str);
        }

        conversation_entry
    }

    fn refresh_conversation(
        &self,
        conversation_entry: &mut (Conversation, chrono::DateTime<Utc>),
        input_str: &String,
    ) {
        let preset = get_preset_based_on_sentiment(input_str);
        println!("Refreshing the conversation with preset: {}", preset);
        *conversation_entry = (
            self.chat_gpt_client.new_conversation_directed(preset),
            Utc::now(),
        );
    }

    fn handle_reset(
        &self,
        conversation_entry: &mut (Conversation, chrono::DateTime<Utc>),
        memory: usize,
    ) {
        // If the conversation history contains more than 20 messages, recreate the conversation with the last 10 messages and pre-prompt message

        if conversation_entry.0.history.len() > 20 {
            // should consider splitting the pre_prompt into mutliple bits?
            let pre_prompt_message = conversation_entry.0.history.get(0).cloned().unwrap();

            let mut message_memory = conversation_entry
                .0
                .history
                .iter()
                .rev()
                .take(memory)
                .cloned()
                .collect::<Vec<_>>();

            // load the initial message, gives more consistent responses.
            message_memory.insert(0, pre_prompt_message);

            println!("Recreating the conversation with the following messages:");
            for message in &message_memory {
                println!("{message:#?}")
            }

            *conversation_entry = (
                Conversation::new_with_history(self.chat_gpt_client.clone(), message_memory),
                Utc::now(),
            );
        }
    }
}
