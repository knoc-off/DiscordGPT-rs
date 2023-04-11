#ChatGPT-Discord-Bot
A Discord bot powered by OpenAI's GPT-4 based ChatGPT that allows users to have an engaging conversation with the bot. The bot automatically adapts its responses based on user sentiment and conversation context. It maintains separate conversations for each channel, providing a more cohesive experience in multi-channel servers.

##Features

- Context-aware conversations
- Sentiment-based response presets
- Persistent conversations per channel
- Time-based conversation reset
- Quick response to recent messages

##Setup and Run

1. Install Rust: https://www.rust-lang.org/tools/install
2. Clone this repository: git clone https://github.com/yourusername/ChatGPT-Discord-Bot.git
3. Change to the project directory: cd ChatGPT-Discord-Bot
4. Set up the following environment variables:
  - DISCORD_TOKEN: Your Discord bot token
  - OPENAI_API_KEY: Your OpenAI API key
5. Run the project: cargo run

##Highlights

- Sentiment-based response presets: The bot analyzes user sentiment and selects a response preset accordingly. This allows for a more engaging and natural conversation with the bot.
- Context-aware conversations: The bot maintains separate conversations for each channel, ensuring a cohesive experience in multi-channel servers.
- Time-based conversation reset: Conversations that are older than 10 minutes will be automatically reset, allowing the bot to start fresh and avoid responding to outdated context.
- Quick response to recent messages: If a user replies quickly to the bot (within 1 minute), the bot will respond regardless of whether its name is mentioned. This feature is channel-specific and time-based.

##How It Works

The bot is implemented using the Serenity crate for Discord API interaction and the ChatGPT crate for OpenAI API interaction. The EventHandler trait is used to handle various Discord events such as bot ready and message events. The main logic resides in the chatbot function, where the bot manages conversations based on channel IDs and user sentiment.
For a more detailed overview of the code, refer to the source code comments and inline documentation.
Feel free to contribute, report issues, or suggest improvements!
