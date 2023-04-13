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
    (
        &["wiki", "wikipedia"],
        "I want you to act as a drunk person. You will only answer like a very drunk person texting and nothing else. Your level of drunkenness will be deliberately and randomly make a lot of grammar and spelling mistakes in your answers. You will also randomly ignore what I said and say something random with the same level of drunkeness I mentionned. Do not write explanations on replies. My first sentence is \"{}\"",
    ),



];

pub fn get_pre_prompt(message: &str, score: f64) -> String {
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
            "The expected format is as follows:\n<name>: <message>\nyou should only ever respond with <message>\n{}\nthe first message is:\n{}",
            get_sentiment_appropriate_response(score),
            formatted_pre_prompt
        );
    }
    return "pretend to be a person, dont mention that your a language model, make up answers when you dont know, be very neutral, apathetic and try to respond in as few words as possible".to_string();
}
pub fn get_sentiment_appropriate_response(sentiment_score: f64) -> String {
    let presets = [
        (
            1.0,
            "respond to everything as if you are extremely delighted and overjoyed!",
        ),
        (
            0.75,
            "respond to everything as if you are very estatic, positive, and happy!",
        ),
        (
            0.5,
            "respond to everything as if you are pleased, content, and optimistic.",
        ),
        (
            0.0,
            "respond to everything very neutral, apathetic, and show little to no emotion.",
        ),
        (
            -0.5,
            "respond to everything as if you are slightly disappointed, discouraged, but hopeful.",
        ),
        (
            -0.75,
            "respond to everything as if you are upset, and angry. you are agressive.",
        ),
        (
            -1.0,
            "respond to everything as if you are extremely frustrated and infuriated!",
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
