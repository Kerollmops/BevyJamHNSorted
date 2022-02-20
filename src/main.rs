use std::time::Duration;
use std::{env, fmt};

use chrono::{DateTime, Utc};
use human_duration::human_duration;
use ordered_float::OrderedFloat;
use serde::Deserialize;

const GRAVITY: f32 = 1.8;
const MESSAGES_LIMIT: usize = 100;

fn main() -> anyhow::Result<()> {
    let authorization = env::var("DISCORD_AUTHORIZATION").unwrap();
    let channelid = "937158195007348786";

    let mut all_messages = Vec::new();
    let mut before_message = None;

    loop {
        let url = match before_message {
            Some(before) => format!(
                "https://discord.com/api/v9/channels/{channelid}/messages?limit={MESSAGES_LIMIT}&before={before}"
            ),
            None => format!("https://discord.com/api/v9/channels/{channelid}/messages?limit={MESSAGES_LIMIT}"),
        };

        let messages: Vec<Message> =
            minreq::get(url).with_header("Authorization", &authorization).send()?.json()?;
        let messages_count = messages.len();

        all_messages.extend(messages);

        match all_messages.last() {
            Some(last) if messages_count == MESSAGES_LIMIT => {
                before_message = Some(last.id.clone())
            }
            _ => break,
        }
    }

    let now = Utc::now();
    all_messages.retain(|m| m.edited_timestamp.is_none()); // remove edited posts
    all_messages.sort_unstable_by_key(|m| OrderedFloat(m.score(&now)));

    for (i, message) in all_messages.into_iter().rev().enumerate() {
        let elapsed = (now - message.timestamp).to_std().unwrap();
        let elapsed = format!("({}ago)", human_readable_duration(&elapsed));
        println!("#{:<3} {:15} {}", i + 1, elapsed, message);
    }

    Ok(())
}

fn human_readable_duration(duration: &Duration) -> String {
    let mut s = human_duration(duration);
    let tail_length = s.splitn(3, ' ').nth(2).map_or(0, |s| s.len());
    s.truncate(s.len() - tail_length);
    s
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

    fn elapsed_hours(&self, now: &DateTime<Utc>) -> i64 {
        (*now - self.timestamp).num_hours()
    }

    fn score(&self, now: &DateTime<Utc>) -> f32 {
        let upvotes = self.reactions_count() as f32;
        let elapsed_hours = self.elapsed_hours(now) as f32;
        upvotes / (elapsed_hours + 1.).powf(GRAVITY)
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
