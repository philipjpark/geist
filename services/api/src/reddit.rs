use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize)]
pub struct RedditMention {
    pub ticker: String,
    pub mentions: u32,
    pub subreddit: String,
    pub sample_title: String,
}

#[derive(Debug, Deserialize)]
struct RedditListing {
    data: RedditListingData,
}

#[derive(Debug, Deserialize)]
struct RedditListingData {
    children: Vec<RedditChild>,
}

#[derive(Debug, Deserialize)]
struct RedditChild {
    data: RedditPost,
}

#[derive(Debug, Deserialize)]
struct RedditPost {
    title: String,
    #[serde(default)]
    selftext: String,
    subreddit: String,
}

/// Subreddits scanned via Reddit's native `.json` feed (no API token).
const SUBREDDITS: &[&str] = &[
    "pennystocks",
    "RobinHoodPennyStocks",
    "otcstocks",
    "StockMarket",
    "stocks",
    "investing",
];

/// Each subreddit is fetched from `/hot.json` and `/rising.json`.
const FEEDS: &[&str] = &["hot", "rising"];

pub async fn fetch_reddit_mentions() -> (Vec<RedditMention>, &'static str) {
    if let Some(live) = fetch_live().await {
        if !live.is_empty() {
            return (live, "reddit_json");
        }
    }
    if let Some(live) = fetch_pullpush().await {
        if !live.is_empty() {
            return (live, "pullpush");
        }
    }
    (mock_reddit_mentions(), "mock_reddit")
}

async fn fetch_live() -> Option<Vec<RedditMention>> {
    let user_agent = std::env::var("REDDIT_USER_AGENT").unwrap_or_else(|_| {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".into()
    });

    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .ok()?;

    let ticker_re = Regex::new(r"\$([A-Z]{1,5})\b").ok()?;
    let mut counts: HashMap<String, (u32, String, String)> = HashMap::new();

    for sub in SUBREDDITS {
        for feed in FEEDS {
            let url = format!("https://www.reddit.com/r/{}/{feed}.json?limit=25", sub);
            let response = match client.get(&url).send().await {
                Ok(r) => r,
                Err(_) => continue,
            };
            if !response.status().is_success() {
                continue;
            }
            let listing: RedditListing = match response.json().await {
                Ok(l) => l,
                Err(_) => continue,
            };

            for child in listing.data.children {
                let post = child.data;
                let text = format!("{} {}", post.title, post.selftext);
                for cap in ticker_re.captures_iter(&text) {
                    let ticker = cap.get(1)?.as_str().to_string();
                    if is_noise_ticker(&ticker) {
                        continue;
                    }
                    let entry = counts
                        .entry(ticker)
                        .or_insert((0, post.subreddit.clone(), post.title.clone()));
                    entry.0 += 1;
                }
            }
        }
    }

    if counts.is_empty() {
        return None;
    }

    let mut mentions: Vec<RedditMention> = counts
        .into_iter()
        .map(|(ticker, (mentions, subreddit, sample_title))| RedditMention {
            ticker,
            mentions,
            subreddit,
            sample_title,
        })
        .collect();
    mentions.sort_by(|a, b| b.mentions.cmp(&a.mentions));
    mentions.truncate(12);
    Some(mentions)
}

async fn fetch_pullpush() -> Option<Vec<RedditMention>> {
    let client = reqwest::Client::builder()
        .user_agent("geist-discovery/1.0")
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .ok()?;

    let ticker_re = Regex::new(r"\$([A-Z]{1,5})\b").ok()?;
    let mut counts: HashMap<String, (u32, String, String)> = HashMap::new();

    for sub in SUBREDDITS {
        let url = format!(
            "https://api.pullpush.io/reddit/search/submission/?subreddit={sub}&size=25&sort=desc&sort_type=created_utc"
        );
        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(_) => continue,
        };
        if !response.status().is_success() {
            continue;
        }

        #[derive(Deserialize)]
        struct PullPushResponse {
            data: Vec<PullPushPost>,
        }

        #[derive(Deserialize)]
        struct PullPushPost {
            title: String,
            #[serde(default)]
            selftext: String,
            subreddit: String,
        }

        let payload: PullPushResponse = match response.json().await {
            Ok(p) => p,
            Err(_) => continue,
        };

        for post in payload.data {
            let text = format!("{} {}", post.title, post.selftext);
            for cap in ticker_re.captures_iter(&text) {
                let ticker = cap.get(1)?.as_str().to_string();
                if is_noise_ticker(&ticker) {
                    continue;
                }
                let entry = counts
                    .entry(ticker)
                    .or_insert((0, post.subreddit.clone(), post.title.clone()));
                entry.0 += 1;
            }
        }
    }

    if counts.is_empty() {
        return None;
    }

    let mut mentions: Vec<RedditMention> = counts
        .into_iter()
        .map(|(ticker, (mentions, subreddit, sample_title))| RedditMention {
            ticker,
            mentions,
            subreddit,
            sample_title,
        })
        .collect();
    mentions.sort_by(|a, b| b.mentions.cmp(&a.mentions));
    mentions.truncate(12);
    Some(mentions)
}

fn is_noise_ticker(ticker: &str) -> bool {
    matches!(
        ticker,
        "USD" | "CEO" | "IPO" | "ETF" | "FDA" | "SEC" | "OTC" | "ATH" | "DD" | "AI" | "USA" | "GDP" | "WSB" | "YOLO"
    )
}

pub fn mock_reddit_mentions() -> Vec<RedditMention> {
    vec![
        RedditMention {
            ticker: "CYDY".into(),
            mentions: 18,
            subreddit: "pennystocks".into(),
            sample_title: "CYDY biotech momentum thread".into(),
        },
        RedditMention {
            ticker: "OZSC".into(),
            mentions: 11,
            subreddit: "pennystocks".into(),
            sample_title: "OZSC volume spike discussion".into(),
        },
        RedditMention {
            ticker: "SHMP".into(),
            mentions: 7,
            subreddit: "RobinHoodPennyStocks".into(),
            sample_title: "SHMP liquidity concerns".into(),
        },
    ]
}
