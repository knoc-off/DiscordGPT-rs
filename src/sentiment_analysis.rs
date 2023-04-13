use vader_sentiment::SentimentIntensityAnalyzer;

use crate::preset_selection::get_pre_prompt;

pub fn analyze_sentiment(message: &str) -> f64 {
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

pub fn get_preset_based_on_sentiment(message: &str) -> String {
    let score = analyze_sentiment(&message);
    // this is a hack but it should work...
    // if (score.abs() - 0.0).abs() < 0.25 {
    // code to run if score is close to 0
    //         return get_pre_prompt(message);
    //     }

    return get_pre_prompt(message, score);
    //return get_sentiment_appropriate_response(score);
}
