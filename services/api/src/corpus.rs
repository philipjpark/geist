use regex::Regex;
use std::collections::HashMap;

use crate::news::NewsItem;
use crate::reddit::RedditMention;
use crate::trends::TrendTopic;
use crate::x_corpus::XMention;

const BASE_SYSTEM: &str = r#"You are Geist — a semantic execution compiler for fragmented and OTC markets.
Structure before reasoning: ingest multi-feed corpus, propagate tickers across Reddit, Trends, News RSS, and X, then rank execution relevance.
Intelligence only — never recommend executing trades or sizing positions."#;

const NOISE: &[&str] = &[
    "SEC", "OTC", "CEO", "FDA", "IPO", "ETF", "LLC", "INC", "THE", "FOR", "AND", "RSS", "API",
    "USD", "NYSE", "NASDAQ", "CNBC", "NEWS", "GROUP", "FLAGS", "EASED", "FRAUD",
];

pub fn is_noise_ticker(ticker: &str) -> bool {
    let t = ticker.to_uppercase();
    NOISE.iter().any(|n| *n == t)
}

#[derive(Debug, Clone)]
pub struct PropagatedTicker {
    pub ticker: String,
    pub score: u8,
    pub sources: String,
    pub reason: String,
    pub leg_type: String,
}

pub fn extract_tickers_from_text(text: &str) -> Vec<String> {
    let re = Regex::new(r"(?i)\$?([A-Z]{2,5})\b").unwrap();
    let upper = text.to_uppercase();
    let mut seen = Vec::new();
    for cap in re.captures_iter(&upper) {
        let t = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        if t.len() < 2 || NOISE.iter().any(|n| *n == t) {
            continue;
        }
        if !seen.contains(&t) {
            seen.push(t);
        }
    }
    seen
}

fn seed_line(seed: &str) -> String {
    if seed.trim().is_empty() {
        "auto-discover".into()
    } else {
        seed.trim().to_string()
    }
}

fn pipeline_line(active: &[String]) -> String {
    let order = ["reddit", "google_trends", "news", "x"];
    let mut parts: Vec<&str> = Vec::new();
    for id in order {
        if active.iter().any(|s| s == id) {
            parts.push(match id {
                "reddit" => "Reddit",
                "google_trends" => "Google Trends",
                "news" => "News RSS",
                "x" => "X",
                _ => id,
            });
        }
    }
    if parts.is_empty() {
        "semantic rank".into()
    } else {
        format!("{} -> semantic rank -> OpenAI correlate", parts.join(" -> "))
    }
}

pub fn build_corpus_system_prompt(
    seed: &str,
    active_sources: &[String],
    reddit: &[RedditMention],
    trends: &[TrendTopic],
    news: &[NewsItem],
    x_mentions: &[XMention],
) -> String {
    let reddit_block = if !active_sources.iter().any(|s| s == "reddit") {
        "(reddit skipped)".into()
    } else if reddit.is_empty() {
        "(no reddit cashtags this scan)".into()
    } else {
        reddit
            .iter()
            .map(|r| {
                format!(
                    "ticker=${} mentions={} subreddit=r/{} sample=\"{}\"",
                    r.ticker, r.mentions, r.subreddit, r.sample_title
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let trends_block = if !active_sources.iter().any(|s| s == "google_trends") {
        "(google trends skipped)".into()
    } else if trends.is_empty() {
        "(no trend topics this scan)".into()
    } else {
        trends
            .iter()
            .map(|t| {
                let extracted = extract_tickers_from_text(&t.topic);
                let tickers = if extracted.is_empty() {
                    String::new()
                } else {
                    format!(" tickers={}", extracted.iter().map(|t| format!("${t}")).collect::<Vec<_>>().join(","))
                };
                format!("topic=\"{}\" interest={}{}", t.topic, t.interest, tickers)
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let news_block = if !active_sources.iter().any(|s| s == "news") {
        "(news rss skipped)".into()
    } else if news.is_empty() {
        "(no news headlines this scan)".into()
    } else {
        news
            .iter()
            .take(12)
            .map(|n| {
                let tickers = extract_tickers_from_text(&n.title);
                let ticker_str = if tickers.is_empty() {
                    String::new()
                } else {
                    format!(" tickers={}", tickers.iter().map(|t| format!("${t}")).collect::<Vec<_>>().join(","))
                };
                format!(
                    "title=\"{}\" publisher={} risk={} tags={}{}",
                    n.title,
                    n.publisher,
                    n.risk_score,
                    n.risk_tags.join(","),
                    ticker_str
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let x_block = if !active_sources.iter().any(|s| s == "x") {
        "(x skipped)".into()
    } else if x_mentions.is_empty() {
        "(no x cashtags this scan)".into()
    } else {
        x_mentions
            .iter()
            .map(|x| {
                format!(
                    "ticker=${} mentions={} author={} sample=\"{}\"",
                    x.ticker, x.mentions, x.author, x.sample_text
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let propagated = propagate_assets(reddit, trends, news, x_mentions);
    let union_block = if propagated.is_empty() {
        "(no tickers propagated — await feed ingest)".into()
    } else {
        propagated
            .iter()
            .take(15)
            .map(|p| {
                format!(
                    "${} score={} sources=[{}] detail=\"{}\"",
                    p.ticker, p.score, p.sources, p.reason
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let openai_line = if openai_configured() {
        "enabled (OPENAI_API_KEY present)"
    } else {
        "unavailable (set OPENAI_API_KEY for LLM correlate)"
    };

    let x_line = if x_configured() {
        "enabled (X_BEARER_TOKEN present)"
    } else {
        "unavailable (set X_BEARER_TOKEN for live X corpus)"
    };

    format!(
        "{BASE_SYSTEM}\n\n\
--- CORPUS INGEST ---\n\
seed: {}\n\
pipeline: {}\n\
feeds_active: {}\n\
openai: {}\n\
x_api: {}\n\n\
[REDDIT]\n{reddit_block}\n\n\
[GOOGLE_TRENDS]\n{trends_block}\n\n\
[NEWS_RSS]\n{news_block}\n\n\
[X]\n{x_block}\n\n\
[TICKER_UNION]\n\
cross_feed_propagation (reddit+trends+news+x overlap):\n{union_block}\n\n\
[AGENTIC_TASK]\n\
1. OBSERVE — ingest all active feeds above\n\
2. PROPAGATE — extract and union tickers across Reddit, Trends headlines, News RSS titles, X cashtags\n\
3. REASON — score overlap, narrative risk (news tags), social velocity (reddit/x mentions), trend interest\n\
4. RANK — emit up to 10 correlated tickers (OTC preferred; include cross-asset when news/trends warrant)\n\
5. COMMIT — hand ranked tickers to semantic IR compile on user selection\n\
--- END CORPUS ---",
        seed_line(seed),
        pipeline_line(active_sources),
        if active_sources.is_empty() {
            "none".into()
        } else {
            active_sources.join(",")
        },
        openai_line,
        x_line,
    )
}

fn openai_configured() -> bool {
    std::env::var("OPENAI_API_KEY")
        .map(|k| !k.is_empty() && k != "your_openai_api_key_here")
        .unwrap_or(false)
}

fn x_configured() -> bool {
    for key in ["X_BEARER_TOKEN", "TWITTER_BEARER_TOKEN"] {
        if let Ok(v) = std::env::var(key) {
            let t = v.trim();
            if !t.is_empty()
                && t != "your_x_bearer_token_here"
                && t != "your_twitter_bearer_token_here"
            {
                return true;
            }
        }
    }
    false
}

pub fn propagate_assets(
    reddit: &[RedditMention],
    trends: &[TrendTopic],
    news: &[NewsItem],
    x_mentions: &[XMention],
) -> Vec<PropagatedTicker> {
    #[derive(Default)]
    struct Acc {
        reddit: u32,
        x: u32,
        news: u32,
        news_risk: u32,
        trend: u32,
        flags: Vec<String>,
    }

    let mut map: HashMap<String, Acc> = HashMap::new();

    for r in reddit {
        let key = r.ticker.to_uppercase();
        if key.len() < 2 || key.len() > 5 || NOISE.iter().any(|n| *n == key) {
            continue;
        }
        let a = map.entry(key).or_default();
        a.reddit += r.mentions;
        if !a.flags.contains(&"reddit".to_string()) {
            a.flags.push("reddit".into());
        }
    }

    for x in x_mentions {
        let key = x.ticker.to_uppercase();
        if key.len() < 2 || key.len() > 5 || NOISE.iter().any(|n| *n == key) {
            continue;
        }
        let a = map.entry(key).or_default();
        a.x += x.mentions;
        if !a.flags.contains(&"x".to_string()) {
            a.flags.push("x".into());
        }
    }

    for n in news {
        for t in extract_tickers_from_text(&n.title) {
            let key = t.to_uppercase();
            if key.len() < 2 || key.len() > 5 || NOISE.iter().any(|n| *n == key) {
                continue;
            }
            let a = map.entry(key).or_default();
            a.news += 1;
            a.news_risk = a.news_risk.saturating_add(n.risk_score as u32);
            if !a.flags.contains(&"news".to_string()) {
                a.flags.push("news".into());
            }
        }
    }

    for tr in trends {
        let boost = (tr.interest / 10) as u32;
        for t in extract_tickers_from_text(&tr.topic) {
            let key = t.to_uppercase();
            if key.len() < 2 || key.len() > 5 || NOISE.iter().any(|n| *n == key) {
                continue;
            }
            let a = map.entry(key).or_default();
            a.trend = a.trend.saturating_add(boost);
            if !a.flags.contains(&"trends".to_string()) {
                a.flags.push("trends".into());
            }
        }
        if boost > 0 {
            for r in reddit.iter().take(3) {
                let key = r.ticker.to_uppercase();
                if key.len() < 2 || key.len() > 5 || NOISE.iter().any(|n| *n == key) {
                    continue;
                }
                let a = map.entry(key).or_default();
                a.trend = a.trend.saturating_add(boost / 2);
                if !a.flags.contains(&"trends".to_string()) {
                    a.flags.push("trends".into());
                }
            }
        }
    }

    let mut out: Vec<PropagatedTicker> = map
        .into_iter()
        .map(|(ticker, a)| {
            let source_count = a.flags.len() as u32;
            let cross_boost = if source_count >= 3 {
                25
            } else if source_count == 2 {
                12
            } else {
                0
            };
            let score = (a.reddit * 3
                + a.x * 3
                + a.news * 8
                + a.trend
                + a.news_risk / 15
                + cross_boost)
                .min(100) as u8;

            let sources = a.flags.join("+");
            let reason = format!(
                "reddit={} x={} news={} trend={}",
                a.reddit, a.x, a.news, a.trend
            );

            PropagatedTicker {
                ticker,
                score: score.max(20),
                sources,
                reason,
                leg_type: "otc".into(),
            }
        })
        .collect();

    out.sort_by(|a, b| b.score.cmp(&a.score));
    out.truncate(15);
    out
}
