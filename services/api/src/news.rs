use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct NewsItem {
    pub title: String,
    pub link: String,
    pub publisher: String,
    pub published: String,
    #[serde(rename = "riskTags")]
    pub risk_tags: Vec<String>,
    #[serde(rename = "riskScore")]
    pub risk_score: u8,
    pub source: String,
}

fn encode_query(q: &str) -> String {
    q.chars()
        .map(|c| match c {
            ' ' => '+'.to_string(),
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}

fn strip_tags(s: &str) -> String {
    let re = Regex::new(r"<[^>]+>").unwrap();
    re.replace_all(s, "").to_string()
}

fn extract_tag(block: &str, tag: &str) -> Option<String> {
    let pattern = format!(r"<{tag}[^>]*>([\s\S]*?)</{tag}>");
    let re = Regex::new(&pattern).ok()?;
    let cap = re.captures(block)?;
    let raw = cap.get(1)?.as_str();
    let text = strip_tags(raw)
        .replace("<![CDATA[", "")
        .replace("]]>", "")
        .trim()
        .to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn parse_rss_items(xml: &str, feed_source: &str) -> Vec<NewsItem> {
    let mut items = Vec::new();
    for block in xml.split("<item>").skip(1) {
        let end = block.find("</item>").unwrap_or(block.len());
        let chunk = &block[..end];

        let title = match extract_tag(chunk, "title") {
            Some(t) => t,
            None => continue,
        };
        let link = extract_tag(chunk, "link").unwrap_or_default();
        let published = extract_tag(chunk, "pubDate").unwrap_or_default();
        let publisher = extract_tag(chunk, "source")
            .or_else(|| {
                title
                    .rsplit(" - ")
                    .next()
                    .filter(|p| p.len() < 80)
                    .map(|p| p.to_string())
            })
            .unwrap_or_else(|| "unknown".into());

        let (risk_tags, risk_score) = classify_risk(&title);
        items.push(NewsItem {
            title,
            link,
            publisher,
            published,
            risk_tags,
            risk_score,
            source: feed_source.into(),
        });
    }
    items
}

fn classify_risk(title: &str) -> (Vec<String>, u8) {
    let lower = title.to_lowercase();
    let mut tags = Vec::new();
    let mut score: u16 = 10;

    let checks: &[(&str, &str, u16)] = &[
        ("fraud_warning", "fraud", 35),
        ("fraud_warning", "scam", 35),
        ("fraud_warning", "ponzi", 40),
        ("sec_action", "sec ", 30),
        ("sec_action", "subpoena", 35),
        ("sec_action", "investigation", 25),
        ("sec_action", "indictment", 40),
        ("sec_action", "charged", 30),
        ("market_halt", "halt", 30),
        ("market_halt", "suspended", 25),
        ("market_halt", "delist", 30),
        ("dilution", "offering", 20),
        ("dilution", "dilution", 25),
        ("dilution", "shelf registration", 25),
        ("dilution", "warrant", 15),
        ("disclosure", "10-k", 10),
        ("disclosure", "10-q", 10),
        ("disclosure", "8-k", 10),
        ("disclosure", "edgar", 10),
        ("disclosure", "filing", 8),
        ("pr_narrative", "press release", 5),
        ("pr_narrative", "announces", 5),
        ("pr_narrative", "partnership", 5),
        ("otc_risk", "otc", 12),
        ("otc_risk", "pink sheet", 20),
        ("otc_risk", "otc markets", 15),
    ];

    for (tag, needle, weight) in checks {
        if lower.contains(needle) && !tags.iter().any(|t| t == tag) {
            tags.push(tag.to_string());
            score += weight;
        }
    }

    if tags.is_empty() {
        tags.push("general".into());
    }

    (tags, score.min(100) as u8)
}

fn build_queries(seed: &str, tickers: &[String]) -> Vec<String> {
    let trimmed = seed.trim();
    let mut queries = Vec::new();

    if trimmed.is_empty() {
        queries.push("OTC penny stock SEC fraud warning".into());
    } else if trimmed.len() <= 5 && trimmed.chars().all(|c| c.is_ascii_alphanumeric()) {
        let t = trimmed.to_uppercase();
        queries.push(format!("{t} OTC stock"));
        queries.push(format!("{t} SEC investigation fraud"));
    } else {
        queries.push(format!("{trimmed} OTC penny stock"));
        queries.push(format!("{trimmed} fraud SEC warning"));
    }

    for ticker in tickers.iter().take(2) {
        let t = ticker.to_uppercase();
        let q = format!("{t} OTC press release");
        if !queries.iter().any(|existing| existing.contains(&t)) {
            queries.push(q);
        }
    }

    queries
}

async fn fetch_feed(url: &str, feed_source: &str) -> Vec<NewsItem> {
    let client = match reqwest::Client::builder()
        .user_agent("Geist/0.1 (OTC narrative risk; +https://github.com/geist)")
        .timeout(std::time::Duration::from_secs(12))
        .build()
    {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let body = match client.get(url).send().await {
        Ok(resp) => match resp.text().await {
            Ok(t) => t,
            Err(_) => return vec![],
        },
        Err(_) => return vec![],
    };
    parse_rss_items(&body, feed_source)
}

async fn fetch_google_news(query: &str) -> Vec<NewsItem> {
    let encoded = encode_query(query);
    let url = format!(
        "https://news.google.com/rss/search?q={encoded}&hl=en-US&gl=US&ceid=US:en"
    );
    fetch_feed(&url, "google_news_rss").await
}

async fn fetch_bing_news(query: &str) -> Vec<NewsItem> {
    let encoded = encode_query(query);
    let url = format!("https://www.bing.com/news/search?q={encoded}&format=rss");
    fetch_feed(&url, "bing_news_rss").await
}

fn dedupe_sort(mut items: Vec<NewsItem>) -> Vec<NewsItem> {
    let mut seen = std::collections::HashSet::new();
    items.retain(|item| {
        let key = item.title.to_lowercase();
        seen.insert(key)
    });
    items.sort_by(|a, b| b.risk_score.cmp(&a.risk_score));
    items.truncate(15);
    items
}

fn mock_news() -> Vec<NewsItem> {
    vec![
        NewsItem {
            title: "OTC Markets Group flags increased fraud warnings in pink sheet issuers".into(),
            link: "https://news.google.com".into(),
            publisher: "Mock Feed".into(),
            published: "Mon, 01 Jan 2026 12:00:00 GMT".into(),
            risk_tags: vec!["fraud_warning".into(), "otc_risk".into()],
            risk_score: 55,
            source: "mock_news".into(),
        },
        NewsItem {
            title: "Biotech microcap announces partnership — investors urged to review 8-K filing".into(),
            link: "https://news.google.com".into(),
            publisher: "Mock Feed".into(),
            published: "Sun, 31 Dec 2025 09:00:00 GMT".into(),
            risk_tags: vec!["pr_narrative".into(), "disclosure".into()],
            risk_score: 25,
            source: "mock_news".into(),
        },
    ]
}

pub async fn fetch_news(seed: &str, tickers: &[String]) -> (Vec<NewsItem>, String) {
    let queries = build_queries(seed, tickers);
    let mut items = Vec::new();
    let mut used_google = false;
    let mut used_bing = false;

    for query in &queries {
        let google = fetch_google_news(query).await;
        if !google.is_empty() {
            used_google = true;
            items.extend(google);
        }
        if items.len() >= 12 {
            break;
        }
    }

    if items.len() < 5 {
        let fallback_query = queries.first().cloned().unwrap_or_else(|| "OTC fraud SEC".into());
        let bing = fetch_bing_news(&fallback_query).await;
        if !bing.is_empty() {
            used_bing = true;
            items.extend(bing);
        }
    }

    items = dedupe_sort(items);

    if items.is_empty() {
        return (mock_news(), "mock_news".into());
    }

    let source = match (used_google, used_bing) {
        (true, true) => "google_news_rss+bing_news_rss",
        (true, false) => "google_news_rss",
        (false, true) => "bing_news_rss",
        (false, false) => "mock_news",
    };

    (items, source.into())
}
