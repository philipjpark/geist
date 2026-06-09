use serde::{Deserialize, Serialize};

use crate::agent::{run_discovery_agent, AgentResult, AgentStep, CorrelatedTicker, CryptoParallel};
use crate::news::{fetch_news, NewsItem};
use crate::reddit::{fetch_reddit_mentions, RedditMention};
use crate::trends::{fetch_trend_topics, TrendTopic};
use crate::x_corpus::{fetch_x_mentions, XMention};

#[derive(Debug, Deserialize)]
pub struct DiscoverRequest {
    #[serde(default)]
    pub seed: String,
    #[serde(default = "default_sources")]
    pub sources: Vec<String>,
}

fn default_sources() -> Vec<String> {
    vec![
        "reddit".into(),
        "google_trends".into(),
        "news".into(),
        "x".into(),
    ]
}

fn has_source(sources: &[String], id: &str) -> bool {
    sources.iter().any(|s| s == id)
}

#[derive(Debug, Serialize)]
pub struct DiscoverSources {
    pub reddit: String,
    #[serde(rename = "google_trends")]
    pub google_trends: String,
    pub stocktwits: String,
    pub x: String,
    pub whatsapp: String,
    pub telegram: String,
    pub tiktok: String,
    pub news: String,
    pub agent: String,
}

#[derive(Debug, Serialize)]
pub struct DiscoverResponse {
    pub seed: String,
    pub sources: DiscoverSources,
    #[serde(rename = "redditMentions")]
    pub reddit_mentions: Vec<RedditMention>,
    #[serde(rename = "trendTopics")]
    pub trend_topics: Vec<TrendTopic>,
    #[serde(rename = "correlatedTickers")]
    pub correlated_tickers: Vec<CorrelatedTicker>,
    #[serde(rename = "agentSummary")]
    pub agent_summary: String,
    #[serde(rename = "agentSteps")]
    pub agent_steps: Vec<AgentStep>,
    #[serde(rename = "thesisInsight")]
    pub thesis_insight: String,
    #[serde(rename = "cryptoParallel")]
    pub crypto_parallel: CryptoParallel,
    #[serde(rename = "suggestedQuery")]
    pub suggested_query: String,
    #[serde(rename = "newsItems")]
    pub news_items: Vec<NewsItem>,
    #[serde(rename = "xMentions")]
    pub x_mentions: Vec<XMention>,
    #[serde(rename = "corpusPrompt")]
    pub corpus_prompt: String,
}

fn source_label(enabled: bool, live: Option<&str>) -> String {
    if !enabled {
        return "skipped".into();
    }
    live.unwrap_or("live").into()
}

fn merge_seed_tickers(reddit: &[RedditMention], x: &[XMention]) -> Vec<String> {
    let mut out = Vec::new();
    for r in reddit {
        let t = r.ticker.to_uppercase();
        if !out.contains(&t) {
            out.push(t);
        }
    }
    for x in x {
        let t = x.ticker.to_uppercase();
        if !out.contains(&t) {
            out.push(t);
        }
    }
    out
}

pub async fn run_discovery(seed: &str, sources: &[String]) -> DiscoverResponse {
    let reddit_on = has_source(sources, "reddit");
    let trends_on = has_source(sources, "google_trends");
    let news_on = has_source(sources, "news");
    let x_on = has_source(sources, "x");

    let reddit_fut = async {
        if reddit_on {
            fetch_reddit_mentions().await
        } else {
            (vec![], "skipped".into())
        }
    };
    let trends_fut = async {
        if trends_on {
            fetch_trend_topics(seed).await
        } else {
            (vec![], "skipped".into())
        }
    };
    let x_fut = async {
        if x_on {
            fetch_x_mentions(seed).await
        } else {
            (vec![], "skipped".into())
        }
    };

    let ((reddit, reddit_source), (trend_topics, trends_source), (x_mentions, x_source)) =
        tokio::join!(reddit_fut, trends_fut, x_fut);

    let seed_tickers = merge_seed_tickers(&reddit, &x_mentions);
    let (news_items, news_source) = if news_on {
        fetch_news(seed, &seed_tickers).await
    } else {
        (vec![], "skipped".into())
    };

    let agent: AgentResult = run_discovery_agent(
        seed,
        sources,
        &reddit,
        &trend_topics,
        &news_items,
        &x_mentions,
    )
    .await;

    let suggested_query = agent
        .correlated
        .first()
        .map(|c| c.ticker.clone())
        .or_else(|| reddit.first().map(|r| r.ticker.clone()))
        .or_else(|| x_mentions.first().map(|x| x.ticker.clone()))
        .unwrap_or_else(|| "CYDY".into());

    DiscoverResponse {
        seed: seed.to_string(),
        sources: DiscoverSources {
            reddit: source_label(reddit_on, Some(&reddit_source)),
            google_trends: source_label(trends_on, Some(&trends_source)),
            stocktwits: "skipped".into(),
            x: source_label(x_on, Some(&x_source)),
            whatsapp: "skipped".into(),
            telegram: "skipped".into(),
            tiktok: "skipped".into(),
            news: source_label(news_on, Some(&news_source)),
            agent: agent.source.clone(),
        },
        reddit_mentions: reddit,
        trend_topics,
        correlated_tickers: agent.correlated,
        agent_summary: agent.summary,
        agent_steps: agent.steps,
        thesis_insight: agent.thesis_insight,
        crypto_parallel: agent.crypto_parallel,
        suggested_query,
        news_items,
        x_mentions,
        corpus_prompt: agent.corpus_prompt,
    }
}
