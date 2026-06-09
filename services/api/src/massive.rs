use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct OtcMarketData {
    pub symbol: String,
    pub name: String,
    pub bid: f64,
    pub ask: f64,
    pub spread_percent: f64,
    pub volume: f64,
    pub volatility: f64,
    pub liquidity_depth: f64,
    pub disclosure_quality: f64,
    pub source: String,
}

#[derive(Debug, Deserialize)]
struct MassiveSnapshotResponse {
    ticker: Option<MassiveTickerSnapshot>,
}

#[derive(Debug, Deserialize)]
struct MassiveTickerSnapshot {
    #[serde(default)]
    ticker: Option<String>,
    #[serde(default, rename = "lastQuote")]
    last_quote: Option<MassiveQuote>,
    #[serde(default, rename = "lastTrade")]
    last_trade: Option<MassiveTrade>,
    #[serde(default)]
    day: Option<MassiveDay>,
}

/// Massive/Polygon snapshot quote fields: `p` = bid, `P` = ask.
#[derive(Debug, Deserialize)]
struct MassiveQuote {
    #[serde(default)]
    p: Option<f64>,
    #[serde(default)]
    P: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct MassiveTrade {
    #[serde(default)]
    p: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct MassiveDay {
    #[serde(default)]
    v: Option<f64>,
    #[serde(default)]
    h: Option<f64>,
    #[serde(default)]
    l: Option<f64>,
    #[serde(default)]
    c: Option<f64>,
}

pub async fn fetch_otc_data(ticker: &str) -> Option<OtcMarketData> {
    let api_key = std::env::var("MASSIVE_API_KEY").ok()?;
    if api_key.is_empty() || api_key == "your_massive_api_key_here" {
        return None;
    }

    let base = std::env::var("MASSIVE_API_BASE_URL")
        .unwrap_or_else(|_| "https://api.massive.com".to_string());
    let url = format!(
        "{}/v2/snapshot/locale/us/markets/stocks/tickers/{}?apiKey={}",
        base.trim_end_matches('/'),
        ticker.to_uppercase(),
        api_key
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .ok()?;
    let response = client.get(&url).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }

    let payload: MassiveSnapshotResponse = response.json().await.ok()?;
    let snap = payload.ticker?;

    let mut bid = snap.last_quote.as_ref().and_then(|q| q.p).unwrap_or(0.0);
    let mut ask = snap.last_quote.as_ref().and_then(|q| q.P).unwrap_or(0.0);
    let last_trade = snap.last_trade.as_ref().and_then(|t| t.p).unwrap_or(0.0);
    let day = snap.day.as_ref();

    if bid <= 0.0 && ask <= 0.0 {
        if last_trade > 0.0 {
            let day_spread = day
                .and_then(|d| {
                    let h = d.h?;
                    let l = d.l?;
                    let c = d.c?;
                    if c > 0.0 { Some(((h - l) / c) * 100.0) } else { None }
                })
                .unwrap_or(4.0)
                .clamp(1.5, 15.0);
            let half = (last_trade * day_spread / 100.0) / 2.0;
            bid = (last_trade - half).max(0.001);
            ask = last_trade + half;
        } else if let Some(d) = day {
            let close = d.c.unwrap_or(0.0);
            if close > 0.0 {
                let range = (d.h.unwrap_or(close) - d.l.unwrap_or(close)).max(close * 0.03);
                bid = (close - range / 2.0).max(0.001);
                ask = close + range / 2.0;
            }
        }
    }

    if bid <= 0.0 && ask <= 0.0 {
        return None;
    }
    if ask <= 0.0 {
        ask = bid;
    }
    if bid <= 0.0 {
        bid = ask;
    }

    let mid = (bid + ask) / 2.0;
    let spread_percent = if mid > 0.0 {
        ((ask - bid).abs() / mid) * 100.0
    } else {
        5.0
    };
    let volume = day.and_then(|d| d.v).unwrap_or(0.0);
    let symbol = snap.ticker.unwrap_or_else(|| ticker.to_uppercase());

    Some(OtcMarketData {
        symbol: symbol.clone(),
        name: format!("{} OTC", symbol),
        bid,
        ask,
        spread_percent,
        volume,
        volatility: estimate_volatility(spread_percent, volume),
        liquidity_depth: estimate_liquidity_depth(volume, spread_percent),
        disclosure_quality: 55.0,
        source: "massive_api".into(),
    })
}

pub fn mock_otc_data(ticker: &str) -> OtcMarketData {
    let upper = ticker.to_uppercase();
    let (name, bid, ask, volume) = match upper.as_str() {
        "CYDY" => ("CytoDyn Inc.", 0.28, 0.31, 1_250_000.0),
        "SHMP" => ("NaturalShrimp Inc.", 0.04, 0.06, 420_000.0),
        "OZSC" => ("Ozop Energy Solutions", 0.015, 0.022, 2_100_000.0),
        _ => ("OTC Equity", 1.20, 1.35, 85_000.0),
    };
    let mid = (bid + ask) / 2.0;
    let spread_percent = ((ask - bid) / mid) * 100.0;

    OtcMarketData {
        symbol: upper.clone(),
        name: name.into(),
        bid,
        ask,
        spread_percent,
        volume,
        volatility: estimate_volatility(spread_percent, volume),
        liquidity_depth: estimate_liquidity_depth(volume, spread_percent),
        disclosure_quality: if upper == "CYDY" { 48.0 } else { 62.0 },
        source: "mock_otc".into(),
    }
}

fn estimate_volatility(spread_percent: f64, volume: f64) -> f64 {
    let spread_component = spread_percent * 3.5;
    let volume_component = if volume < 100_000.0 {
        35.0
    } else if volume < 500_000.0 {
        22.0
    } else {
        12.0
    };
    (spread_component + volume_component).min(95.0)
}

fn estimate_liquidity_depth(volume: f64, spread_percent: f64) -> f64 {
    let volume_score = (volume / 50_000.0).min(80.0);
    let spread_penalty = spread_percent * 4.0;
    (volume_score - spread_penalty).clamp(5.0, 95.0)
}
