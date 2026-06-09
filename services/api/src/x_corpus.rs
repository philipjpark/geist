use base64::{engine::general_purpose::STANDARD as B64, Engine};
use hmac::{Hmac, Mac};
use rand::Rng;
use regex::Regex;
use serde::Deserialize;
use sha1::Sha1;
use std::collections::HashSet;

#[derive(Debug, Clone, serde::Serialize)]
pub struct XMention {
    pub ticker: String,
    pub mentions: u32,
    pub author: String,
    #[serde(rename = "sampleText")]
    pub sample_text: String,
}

/// Single API call — home timeline only, first N posts.
const MAX_X_POSTS: usize = 5;

#[derive(Debug, Deserialize)]
struct V1Tweet {
    text: String,
    user: V1User,
}

#[derive(Debug, Deserialize)]
struct V1User {
    screen_name: String,
}

struct OAuthCreds {
    api_key: String,
    api_secret: String,
    access_token: String,
    access_token_secret: String,
}

fn oauth_creds() -> Option<OAuthCreds> {
    let api_key = std::env::var("X_API_KEY").ok()?;
    let api_secret = std::env::var("X_API_SECRET").ok()?;
    let access_token = std::env::var("X_ACCESS_TOKEN").ok()?;
    let access_token_secret = std::env::var("X_ACCESS_TOKEN_SECRET").ok()?;

    let placeholders = [
        "your_x_api_key_here",
        "your_x_api_secret_here",
        "your_x_access_token_here",
        "your_x_access_token_secret_here",
    ];

    let vals = [&api_key, &api_secret, &access_token, &access_token_secret];
    if vals.iter().any(|v| {
        let t = v.trim();
        t.is_empty() || placeholders.iter().any(|p| *p == t)
    }) {
        return None;
    }

    Some(OAuthCreds {
        api_key: api_key.trim().to_string(),
        api_secret: api_secret.trim().to_string(),
        access_token: access_token.trim().to_string(),
        access_token_secret: access_token_secret.trim().to_string(),
    })
}

fn pct_encode(input: &str) -> String {
    let mut out = String::new();
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn oauth_authorization(creds: &OAuthCreds, method: &str, url: &str) -> String {
    let (base_url, query_pairs) = match url.split_once('?') {
        Some((base, qs)) => {
            let pairs: Vec<(String, String)> = qs
                .split('&')
                .filter_map(|p| p.split_once('='))
                .map(|(k, v)| (pct_encode(k), pct_encode(v)))
                .collect();
            (base, pairs)
        }
        None => (url, vec![]),
    };

    let nonce: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into());

    let mut params = vec![
        ("oauth_consumer_key".into(), pct_encode(&creds.api_key)),
        ("oauth_nonce".into(), pct_encode(&nonce)),
        ("oauth_signature_method".into(), "HMAC-SHA1".into()),
        ("oauth_timestamp".into(), pct_encode(&timestamp)),
        ("oauth_token".into(), pct_encode(&creds.access_token)),
        ("oauth_version".into(), "1.0".into()),
    ];
    params.extend(query_pairs);

    params.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    let param_string = params
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&");

    let base_string = format!(
        "{}&{}&{}",
        method.to_uppercase(),
        pct_encode(base_url),
        pct_encode(&param_string)
    );
    let signing_key = format!(
        "{}&{}",
        pct_encode(&creds.api_secret),
        pct_encode(&creds.access_token_secret)
    );

    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(signing_key.as_bytes()).expect("hmac key");
    mac.update(base_string.as_bytes());
    let signature = B64.encode(mac.finalize().into_bytes());

    format!(
        r#"OAuth oauth_consumer_key="{}", oauth_nonce="{}", oauth_signature="{}", oauth_signature_method="HMAC-SHA1", oauth_timestamp="{}", oauth_token="{}", oauth_version="1.0""#,
        creds.api_key,
        nonce,
        pct_encode(&signature),
        timestamp,
        creds.access_token,
    )
}

fn oauth_get(creds: &OAuthCreds, url: &str) -> Option<reqwest::RequestBuilder> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .ok()?;
    let auth = oauth_authorization(creds, "GET", url);
    Some(client.get(url).header("Authorization", auth))
}

fn default_x_mentions() -> Vec<XMention> {
    vec![
        XMention {
            ticker: "BNB".into(),
            mentions: 1,
            author: "@feed".into(),
            sample_text: "$BNB default corpus signal".into(),
        },
        XMention {
            ticker: "CAKE".into(),
            mentions: 1,
            author: "@feed".into(),
            sample_text: "$CAKE default corpus signal".into(),
        },
    ]
}

/// Cashtags and bare crypto symbols from the first `MAX_X_POSTS` posts.
fn mentions_from_posts(posts: &[(String, String)]) -> Vec<XMention> {
    let cashtag = Regex::new(r"(?i)\$([A-Za-z]{2,10})\b").unwrap();
    let bare = Regex::new(r"(?i)\b(BNB|CAKE|ETH|BTC|SOL)\b").unwrap();
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    for (text, author) in posts.iter().take(MAX_X_POSTS) {
        let mut found_in_post = Vec::new();

        for cap in cashtag.captures_iter(text) {
            found_in_post.push(cap[1].to_uppercase());
        }
        for cap in bare.captures_iter(text) {
            found_in_post.push(cap[1].to_uppercase());
        }

        for ticker in found_in_post {
            if seen.insert(ticker.clone()) {
                out.push(XMention {
                    ticker,
                    mentions: 1,
                    author: author.clone(),
                    sample_text: text.chars().take(140).collect(),
                });
            }
        }
    }

    out
}

async fn fetch_home_timeline(creds: &OAuthCreds) -> Option<Vec<XMention>> {
    let url = format!(
        "https://api.twitter.com/1.1/statuses/home_timeline.json?count={MAX_X_POSTS}&tweet_mode=extended"
    );
    let request = oauth_get(creds, &url)?;
    let response = request.send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }

    let tweets: Vec<V1Tweet> = response.json().await.ok()?;
    if tweets.is_empty() {
        return None;
    }

    let posts: Vec<(String, String)> = tweets
        .into_iter()
        .take(MAX_X_POSTS)
        .map(|t| (t.text, format!("@{}", t.user.screen_name)))
        .collect();

    let mentions = mentions_from_posts(&posts);
    if mentions.is_empty() {
        Some(default_x_mentions())
    } else {
        Some(mentions)
    }
}

pub async fn fetch_x_mentions(_seed: &str) -> (Vec<XMention>, String) {
    // One API call: home timeline, 5 posts. No bearer search fallback.
    if let Some(creds) = oauth_creds() {
        if let Some(live) = fetch_home_timeline(&creds).await {
            return (live, "x_home_timeline".into());
        }
    }

    (default_x_mentions(), "x_defaults".into())
}
