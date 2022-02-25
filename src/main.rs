use std::time::Duration;
use std::{env, fmt};

use anyhow::bail;
use chrono::{DateTime, Utc};
use human_duration::human_duration;
use ordered_float::OrderedFloat;
use serde::Deserialize;

const GRAVITY: f32 = 1.8;
const MESSAGES_LIMIT: usize = 100;

enum Sort {
    ByScore,
    ByReactions,
    ByReactionsThroughput,
}

fn main() -> anyhow::Result<()> {
    let authorization = env::var("DISCORD_AUTHORIZATION").unwrap();
    let channelid = "937158195007348786";

    let sort = match env::args().nth(1).as_ref().map(AsRef::as_ref) {
        Some("--sort-by-score") => Sort::ByScore,
        Some("--sort-by-reactions") => Sort::ByReactions,
        Some("--sort-by-reactions-throughput") => Sort::ByReactionsThroughput,
        Some(_) => bail!(
            "invalid argument, please use \
            `--sort-by-score`, `--sort-by-reactions` or `--sort-by-reactions-throughput`"
        ),
        None => Sort::ByReactions,
    };

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
    match sort {
        Sort::ByScore => all_messages.sort_unstable_by_key(|m| OrderedFloat(m.score(&now))),
        Sort::ByReactions => all_messages.sort_unstable_by_key(Message::reactions_count),
        Sort::ByReactionsThroughput => all_messages.sort_unstable_by_key(|m| {
            let elapsed = now - m.timestamp;
            let num_hours = elapsed.num_hours() as f32;
            let num_mins = elapsed.num_minutes() as f32 / 60.;
            let throughput = m.reactions_count() as f32 / (num_hours + num_mins);
            OrderedFloat(throughput)
        }),
    }

    for (i, message) in all_messages.into_iter().rev().enumerate() {
        let elapsed = now - message.timestamp;
        let throughput = message.reactions_count() as f32 / elapsed.num_hours() as f32;
        let human_readable_duration = human_readable_duration(&elapsed.to_std().unwrap()) + "ago,";
        let elapsed = format!("({:<12} {:.02?}👍/h)", human_readable_duration, throughput);
        println!("#{:<3} {:25} {}", i + 1, elapsed, message);
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
    #[serde(rename = "channel_id")]
    _channel_id: String,
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
    #[serde(rename = "me")]
    _me: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct Emoji {
    name: String,
}
