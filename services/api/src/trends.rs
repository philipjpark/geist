use serde::Deserialize;
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct TrendTopic {
    pub topic: String,
    pub interest: u8,
    pub source: String,
}

pub async fn fetch_trend_topics(seed: &str) -> (Vec<TrendTopic>, &'static str) {
    if let Some(live) = fetch_pytrends(seed).await {
        if !live.is_empty() {
            return (live, "pytrends");
        }
    }
    (mock_trend_topics(seed), "mock_trends")
}

async fn fetch_pytrends(seed: &str) -> Option<Vec<TrendTopic>> {
    let script = resolve_pytrends_script()?;
    let python = std::env::var("PYTHON_BIN").unwrap_or_else(|_| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });

    let queries: Vec<String> = if seed.is_empty() {
        vec!["penny stocks".into(), "OTC stocks".into()]
    } else {
        vec![format!("{seed} penny stock"), "penny stocks".into()]
    };

    let mut topics = Vec::new();
    for query in queries {
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(18),
            Command::new(&python).arg(&script).arg(&query).output(),
        )
        .await
        .ok()?
        .ok()?;

        if !output.status.success() {
            continue;
        }

        let stdout = String::from_utf8(output.stdout).ok()?;
        if let Ok(parsed) = serde_json::from_str::<Vec<TrendTopic>>(stdout.trim()) {
            if !parsed.is_empty() {
                topics = parsed;
                break;
            }
        }
    }

    if topics.is_empty() {
        return None;
    }

    let mut topics = topics;
    for topic in &mut topics {
        topic.interest = topic.interest.min(100);
        if topic.source.is_empty() {
            topic.source = "pytrends".into();
        }
    }
    if topics.is_empty() {
        None
    } else {
        Some(topics)
    }
}

fn resolve_pytrends_script() -> Option<PathBuf> {
    let mut candidates = vec![
        PathBuf::from("../../scripts/pytrends_fetch.py"),
        PathBuf::from("scripts/pytrends_fetch.py"),
        PathBuf::from("../scripts/pytrends_fetch.py"),
    ];
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        candidates.push(PathBuf::from(manifest).join("../../scripts/pytrends_fetch.py"));
    }
    candidates
        .into_iter()
        .find(|path| path.exists())
        .map(|path| path.canonicalize().unwrap_or(path))
}

pub fn mock_trend_topics(seed: &str) -> Vec<TrendTopic> {
    let base = if seed.is_empty() { "OTC biotech" } else { seed };
    vec![
        TrendTopic {
            topic: format!("{base} penny stock"),
            interest: 82,
            source: "mock_trends".into(),
        },
        TrendTopic {
            topic: "OTC liquidity".into(),
            interest: 67,
            source: "mock_trends".into(),
        },
        TrendTopic {
            topic: "fragmented markets".into(),
            interest: 54,
            source: "mock_trends".into(),
        },
    ]
}
