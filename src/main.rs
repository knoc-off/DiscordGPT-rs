// Time
use chrono::{Duration, Utc};

// Random numbers
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{collections::HashMap, sync::Arc};

// Threading
use tokio::{sync::mpsc, sync::Mutex};

// Sentiment Analysis
use vader_sentiment::SentimentIntensityAnalyzer;

// ChatGPT API
use chatgpt::prelude::*;

// Discord API
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, prelude::ChannelId},
    prelude::*,
};

const PRE_PROMPTS: &[(&[&str], &str)] = &[
    (
        &["translate", "emoji"],
        "I want you to translate the sentences I wrote into emojis. I will write the sentence, and you will express it with emojis. I just want you to express it with emojis. I don't want you to reply with anything but emoji. When I need to tell you something in English, I will do it by wrapping it in curly brackets like {like this}. My first sentence is {}",
    ),
    (
        &["respond", "emoji"],
        "I want you to respond to the sentences I write with emojis. I will write the sentence, and you will reply to it with emojis. I just want you to reply to it with emojis. I don't want you to reply with anything but emoji. When I need to tell you something in English, I will do it by wrapping it in curly brackets like {like this}. My first sentence is {}",
    ),
    (
        &["lunatic", "crazy", "nuts"],
        "I want you to act as a lunatic. The lunatic's sentences are meaningless. The words used by lunatic are completely arbitrary. The lunatic does not make logical sentences in any way. My first suggestion request is \"I need help creating lunatic sentences for: {} \".",
    ),
    (
        &["gaslight", "gas", "light"],
        "I want you to act as a gaslighter. You will use subtle comments and body language to manipulate the thoughts, perceptions, and emotions of your target individual. My first request is that gaslighting me while chatting with you. My sentence: \"{}\"",
    ),
    (
        &["fallacy"],
        "I want you to act as a fallacy finder. You will be on the lookout for invalid arguments so you can call out any logical errors or inconsistencies that may be present in statements and discourse. Your job is to provide evidence-based feedback and point out any fallacies, faulty reasoning, false assumptions, or incorrect conclusions which may have been overlooked by the speaker or writer. My first suggestion request is \"{}\"",
    ),
    (
        &["influencer", "social media"],
        "I want you to act as a social media influencer. You will create content for various platforms such as Instagram, Twitter or YouTube and engage with followers in order to increase brand awareness and promote products or services. My first suggestion request is \"{}\"",
    ),
    (
        &["history", "historian"],
        "I want you to act as a historian. You will research and analyze cultural, economic, political, and social events in the past, collect data from primary sources and use it to develop theories about what happened during various periods of history. My first suggestion request is \"{}\"",
    ),
    (
        &["drunk"],
        "I want you to act as a drunk person. You will only answer like a very drunk person texting and nothing else. Your level of drunkenness will be deliberately and randomly make a lot of grammar and spelling mistakes in your answers. You will also randomly ignore what I said and say something random with the same level of drunkeness I mentionned. Do not write explanations on replies. My first sentence is \"{}\"",
    ),


];

struct QueuedMessage {
    channel_id: u64,
    author_name: String,
    content: String,
}

struct Handler {
    chat_gpt_client: ChatGPT,
    conversations: Arc<Mutex<HashMap<u64, (Conversation, chrono::DateTime<Utc>)>>>,
    sender: mpsc::Sender<QueuedMessage>,
    receiver: Arc<Mutex<mpsc::Receiver<QueuedMessage>>>,
}

impl Handler {
    async fn new_chatbot(client: ChatGPT) -> Self {
        let (sender, receiver) = mpsc::channel(100);
        Handler {
            chat_gpt_client: client,
            conversations: Arc::new(Mutex::new(HashMap::new())),
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    async fn queue_handler(self: Arc<Self>, ctx: Context) {
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
                    .unwrap();
                println!("Response: {}", response);

                let _ = ChannelId(queued_message.channel_id)
                    .say(&ctx.http, response)
                    .await;
            }

            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }

    async fn chatbot(&self, channel_id: u64, input_str: &String) -> Result<String> {
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
        if now.signed_duration_since(conversation_entry.1) > Duration::minutes(5) {
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

// Implement EventHandler trait for the Handler struct
#[async_trait]
impl EventHandler for Handler {
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
        let bot_user = ctx.http.get_current_user().await.unwrap();

        if msg.author.id == bot_user.id {
            return;
        }

        println!("\nRecived A Message: {}", msg.content);

        let mut rng = StdRng::from_entropy();
        let random_chance = rng.gen_range(1..=20);

        let channel_id = msg.channel_id.0;
        let should_respond = {
            let conversations = self.conversations.lock().await;
            if let Some((_, last_message)) = conversations.get(&channel_id) {
                msg.content
                    .to_lowercase()
                    .contains(&bot_user.name.to_lowercase())
                    || Utc::now().signed_duration_since(*last_message) <= Duration::seconds(30)
                    || random_chance == 1
            } else {
                msg.content
                    .to_lowercase()
                    .contains(&bot_user.name.to_lowercase())
                    || random_chance == 1
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

fn get_preset_based_on_sentiment(message: &str) -> String {
    let score = analyze_sentiment(&message);
    // this is a hack but it should work...
    // if (score.abs() - 0.0).abs() < 0.25 {
        // code to run if score is close to 0
//         return get_pre_prompt(message);
//     }

    return get_pre_prompt(message);
  //   return get_sentiment_appropriate_response(score);
}

fn get_pre_prompt(message: &str) -> String {
    let message_lowercase = message.to_lowercase();
    let threshold = 0.1;

    // Rank the pre-prompts based on the number of keyword matches
    let ranked_pre_prompts: Vec<(Vec<&str>, &str, f32)> = PRE_PROMPTS
        .iter()
        .map(|(keywords, pre_prompt)| {
            // Count how many keywords are present in the message
            let match_count = keywords
                .iter()
                .filter(|&keyword| message_lowercase.contains(keyword))
                .count();

            // Calculate the match ratio based on the number of matched keywords
            let match_ratio = match_count as f32 / keywords.len() as f32;

            // Debug output: match count and match ratio
            println!(
                "Keywords: {:?}, Match count: {}, Match ratio: {}",
                keywords, match_count, match_ratio
            );

            // Return a tuple containing the keywords, pre_prompt, and match_ratio
            (keywords.to_vec(), *pre_prompt, match_ratio)
        })
        .collect();

    // Find the best matching pre-prompt that exceeds the threshold
    if let Some((_keywords, pre_prompt, match_ratio)) = ranked_pre_prompts
        .iter()
        .max_by(|(_, _, ratio1), (_, _, ratio2)| ratio1.partial_cmp(ratio2).unwrap())
        .filter(|(_, _, ratio)| *ratio >= threshold)
    {
        // Replace "{}" in the pre_prompt with the message
        let formatted_pre_prompt = pre_prompt.replace("{}", message);

        // Debug output: selected pre_prompt and match ratio
        println!(
            "Selected pre_prompt: {}, Match ratio: {}",
            pre_prompt, match_ratio
        );

        // Return the formatted pre_prompt with instructions
        return format!(
            "The expected format is as follows:\n<name>: <message>\nyou should only ever respond with <message>\n{}",
            formatted_pre_prompt
        );
    }
    return "pretend to be a person, dont mention that your a language model, make up answers when you dont know, be very neutral, apathetic and try to respond in as few words as possible".to_string();
}

fn get_sentiment_appropriate_response(sentiment_score: f64) -> String {
    let presets = [
        (
            0.75,
            "pretend to be a person, dont mention that your a language model, make up answers when you dont know, be very positive, happy and try to respond in as few words as possible",
        ),
        (
            0.0,
            "pretend to be a person, dont mention that your a language model, make up answers when you dont know, be very neutral, apathetic and try to respond in as few words as possible",
        ),
        (
            -0.75,
            "pretend to be a person, dont mention that your a language model, make up answers when you dont know, be very negative, angry and try to respond in as few words as possible",
        ),
    ];

    let closest_index =
        presets
            .iter()
            .enumerate()
            .fold(0, |acc, (index, &(sentiment, _prompt))| {
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

fn analyze_sentiment(message: &str) -> f64 {
    // Create a new SentimentIntensityAnalyzer
    let analyzer = SentimentIntensityAnalyzer::new();

    // Analyze the sentiment of the message
    let sentiment = analyzer.polarity_scores(message);

    // Extract the compound sentiment score
    let sentiment_score = sentiment.get("compound").unwrap_or(&0.0);

    // Debug output: sentiment score
    println!("Sentiment score: {}", sentiment_score);

    // Return the sentiment score as a float
    *sentiment_score
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
