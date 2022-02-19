use std::{env, fmt};

use chrono::{DateTime, Utc};
use ordered_float::OrderedFloat;
use serde::Deserialize;

const GRAVITY: f32 = 1.8;

fn main() -> anyhow::Result<()> {
    let authorization = env::var("DISCORD_AUTHORIZATION").unwrap();
    let channelid = "937158195007348786";
    let response = minreq::get(format!("https://discord.com/api/v9/channels/{channelid}/messages"))
        .with_header("Authorization", authorization)
        .send()?;

    let now = Utc::now();
    let mut messages: Vec<Message> = response.json()?;
    messages.retain(|m| m.edited_timestamp.is_none()); // remove edited posts
    messages.sort_unstable_by_key(|m| OrderedFloat(m.score(&now)));

    for message in messages {
        println!("{}", message);
    }

    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
struct Message {
    id: String,
    channel_id: String,
    content: String,
    #[serde(default)]
    reactions: Vec<Reaction>,
    timestamp: DateTime<Utc>,
    edited_timestamp: Option<DateTime<Utc>>,
}

impl Message {
    fn reactions_count(&self) -> usize {
        self.reactions.iter().map(|r| if r.emoji.name == "👍" { r.count } else { 0 }).sum()
    }

    fn score(&self, now: &DateTime<Utc>) -> f32 {
        let upvotes = self.reactions_count() as f32;
        let elapsed_hour = (*now - self.timestamp).num_hours() as f32;
        upvotes / (elapsed_hour + 2.0).powf(GRAVITY)
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:5} 👍 - {}", self.reactions_count(), self.content)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Reaction {
    count: usize,
    emoji: Emoji,
    me: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct Emoji {
    name: String,
}
