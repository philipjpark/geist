use serde::{Deserialize, Serialize};

use crate::corpus::{build_corpus_system_prompt, is_noise_ticker, propagate_assets, PropagatedTicker};
use crate::news::NewsItem;
use crate::reddit::RedditMention;
use crate::trends::TrendTopic;
use crate::x_corpus::XMention;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedTicker {
    pub ticker: String,
    pub score: u8,
    pub reason: String,
    #[serde(rename = "legType", default = "default_leg_type")]
    pub leg_type: String,
}

fn default_leg_type() -> String {
    "otc".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub step: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoParallel {
    pub symbol: String,
    pub rationale: String,
}

#[derive(Debug, Clone)]
pub struct AgentResult {
    pub summary: String,
    pub correlated: Vec<CorrelatedTicker>,
    pub steps: Vec<AgentStep>,
    pub thesis_insight: String,
    pub crypto_parallel: CryptoParallel,
    pub source: String,
    pub corpus_prompt: String,
}

pub async fn run_discovery_agent(
    seed: &str,
    active_sources: &[String],
    reddit: &[RedditMention],
    trends: &[TrendTopic],
    news: &[NewsItem],
    x_mentions: &[XMention],
) -> AgentResult {
    let corpus_prompt =
        build_corpus_system_prompt(seed, active_sources, reddit, trends, news, x_mentions);

    if should_use_openai() {
        if let Some(live) = call_openai(seed, &corpus_prompt).await {
            return AgentResult { corpus_prompt, ..live };
        }
    }

    rank_from_corpus(seed, reddit, trends, news, x_mentions, corpus_prompt)
}

fn should_use_openai() -> bool {
    if std::env::var("DISCOVERY_USE_LLM")
        .map(|v| v == "0" || v.eq_ignore_ascii_case("false"))
        .unwrap_or(false)
    {
        return false;
    }
    std::env::var("OPENAI_API_KEY")
        .map(|k| !k.is_empty() && k != "your_openai_api_key_here")
        .unwrap_or(false)
}

async fn call_openai(seed: &str, corpus_prompt: &str) -> Option<AgentResult> {
    let api_key = std::env::var("OPENAI_API_KEY").ok()?;
    if api_key.is_empty() || api_key == "your_openai_api_key_here" {
        return None;
    }

    let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".into());

    let user = format!(
        "{corpus_prompt}\n\n\
Task: Using the full corpus above (Reddit, Google Trends, News RSS, X, TICKER_UNION), \
propagate and rank up to 10 tickers with highest cross-feed conviction. \
Weight news risk tags, social velocity, and trend interest. \
Return JSON: summary, thesisInsight, cryptoParallel, steps (OBSERVE/REASON/PLAN/COMMIT), \
correlated array (up to 10 items with ticker, score 0-100, reason, legType otc|stock). \
Seed: {}",
        if seed.is_empty() { "auto-discover" } else { seed }
    );

    #[derive(Serialize)]
    struct ChatMessage {
        role: &'static str,
        content: String,
    }

    #[derive(Serialize)]
    struct ChatRequest {
        model: String,
        messages: Vec<ChatMessage>,
        temperature: f32,
        response_format: ResponseFormat,
    }

    #[derive(Serialize)]
    struct ResponseFormat {
        #[serde(rename = "type")]
        format_type: &'static str,
    }

    let body = ChatRequest {
        model,
        messages: vec![
            ChatMessage {
                role: "system",
                content: corpus_prompt.to_string(),
            },
            ChatMessage {
                role: "user",
                content: user,
            },
        ],
        temperature: 0.25,
        response_format: ResponseFormat {
            format_type: "json_object",
        },
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(28))
        .build()
        .ok()?;

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let payload: OpenAiResponse = response.json().await.ok()?;
    let content = payload.choices.first()?.message.content.clone();
    parse_agent_json(&content, "openai", corpus_prompt.to_string())
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Deserialize)]
struct OpenAiMessage {
    content: String,
}

fn parse_agent_json(content: &str, source: &str, corpus_prompt: String) -> Option<AgentResult> {
    let trimmed = content.trim();
    let json_str = if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            &trimmed[start..=end]
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    #[derive(Deserialize)]
    struct AgentJson {
        summary: String,
        #[serde(rename = "thesisInsight", default)]
        thesis_insight: String,
        #[serde(rename = "cryptoParallel", default)]
        crypto_parallel: Option<CryptoParallel>,
        #[serde(default)]
        steps: Vec<AgentStep>,
        correlated: Vec<CorrelatedTicker>,
    }

    let parsed: AgentJson = serde_json::from_str(json_str).ok()?;
    let mut correlated: Vec<CorrelatedTicker> = parsed
        .correlated
        .into_iter()
        .filter(|c| !is_noise_ticker(&c.ticker))
        .take(10)
        .collect();

    Some(AgentResult {
        summary: parsed.summary,
        correlated,
        steps: if parsed.steps.is_empty() {
            default_steps()
        } else {
            parsed.steps
        },
        thesis_insight: parsed.thesis_insight,
        crypto_parallel: parsed.crypto_parallel.unwrap_or(CryptoParallel {
            symbol: "ETH".into(),
            rationale: "Onchain beta leg parallel to OTC narrative risk.".into(),
        }),
        source: source.into(),
        corpus_prompt,
    })
}

fn default_steps() -> Vec<AgentStep> {
    vec![
        AgentStep {
            step: "OBSERVE".into(),
            status: "done".into(),
            detail: "Reddit, Trends, News RSS, X ingested".into(),
        },
        AgentStep {
            step: "REASON".into(),
            status: "done".into(),
            detail: "Cross-feed ticker propagation".into(),
        },
        AgentStep {
            step: "PLAN".into(),
            status: "done".into(),
            detail: "Up to 10 assets ranked".into(),
        },
        AgentStep {
            step: "COMMIT".into(),
            status: "ready".into(),
            detail: "Select ticker to compile".into(),
        },
    ]
}

fn propagated_to_correlated(propagated: &[PropagatedTicker]) -> Vec<CorrelatedTicker> {
    propagated
        .iter()
        .filter(|p| !is_noise_ticker(&p.ticker))
        .take(10)
        .map(|p| CorrelatedTicker {
            ticker: p.ticker.clone(),
            score: p.score,
            reason: format!("{} · {}", p.sources, p.reason),
            leg_type: p.leg_type.clone(),
        })
        .collect()
}

fn rank_from_corpus(
    _seed: &str,
    reddit: &[RedditMention],
    trends: &[TrendTopic],
    news: &[NewsItem],
    x_mentions: &[XMention],
    corpus_prompt: String,
) -> AgentResult {
    let propagated = propagate_assets(reddit, trends, news, x_mentions);
    let correlated = propagated_to_correlated(&propagated);

    let top = correlated
        .first()
        .map(|c| c.ticker.clone())
        .or_else(|| reddit.first().map(|r| r.ticker.clone()))
        .or_else(|| x_mentions.first().map(|x| x.ticker.clone()))
        .unwrap_or_else(|| "CYDY".into());

    let trend = trends
        .first()
        .map(|t| t.topic.clone())
        .unwrap_or_else(|| "OTC penny stocks".into());

    let news_note = if news.is_empty() {
        String::new()
    } else {
        format!(" {} news headlines scanned.", news.len())
    };

    let correlated = if correlated.is_empty() {
        vec![CorrelatedTicker {
            ticker: top.clone(),
            score: 70,
            reason: "corpus default rank".into(),
            leg_type: "otc".into(),
        }]
    } else {
        correlated
    };

    AgentResult {
        summary: format!(
            "${} leads cross-feed scan ({} assets propagated). Trend: \"{}\".{news_note} Ready to compile.",
            top,
            correlated.len(),
            trend
        ),
        correlated,
        steps: default_steps(),
        thesis_insight: "Multi-feed corpus propagation surfaces OTC tickers where social velocity, news risk tags, and trend interest overlap.".into(),
        crypto_parallel: CryptoParallel {
            symbol: "ETH".into(),
            rationale: "24/7 onchain coordination parallels broker-hours OTC fragmentation.".into(),
        },
        source: "corpus_rank".into(),
        corpus_prompt,
    }
}
