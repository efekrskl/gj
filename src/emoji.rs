use std::collections::HashMap;

pub fn apply_emoji_prefix(sentence: &str) -> String {
    let emoji_map: HashMap<&str, &str> = HashMap::from([
        ("help", "🤝"),
        ("mentor", "🧑‍🏫"),
        ("review", "🔍"),
        ("implement", "💻"),
        ("write", "✍️"),
        ("document", "📄"),
        ("test", "🧪"),
        ("debug", "🐞"),
        ("learn", "📚"),
        ("present", "🗣️"),
        ("join", "👋"),
        ("meeting", "📅"),
        ("refactor", "🧼"),
        ("deploy", "🚀"),
        ("design", "🎨"),
        ("discuss", "💬"),
        ("plan", "🧠"),
        ("optimize", "⚡"),
        ("research", "🔬"),
        ("sync", "🔄"),
        ("fix", "🛠️"),
        ("pair", "👯"),
    ]);

    let sentence_lowercase = sentence.to_lowercase();
    let maybe_emoji = emoji_map
        .iter()
        .find(|(stem, _)| sentence_lowercase.contains(*stem))
        .map(|(_, emoji)| *emoji);

    if let Some(emoji) = maybe_emoji {
        format!("{} {}", emoji, sentence)
    } else {
        sentence.trim().to_string()
    }
}
