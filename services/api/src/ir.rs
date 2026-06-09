use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseRequest {
    pub text: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(rename = "market_type", default = "default_market_type")]
    pub market_type: String,
}

fn default_mode() -> String {
    "balanced".into()
}

fn default_market_type() -> String {
    "cross_asset".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResponse {
    #[serde(rename = "semantic_ir")]
    pub semantic_ir: SemanticIR,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeRequest {
    #[serde(rename = "semantic_ir")]
    pub semantic_ir: SemanticIR,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeResponse {
    #[serde(rename = "asset_graph")]
    pub asset_graph: AssetGraph,
    #[serde(rename = "execution_dag")]
    pub execution_dag: ExecutionDAG,
    #[serde(rename = "risk_scores")]
    pub risk_scores: RiskScore,
    pub recommendations: Vec<String>,
    #[serde(rename = "monad_proof")]
    pub monad_proof: MonadProof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticIR {
    pub intent: String,
    pub domain: String,
    pub entities: Vec<Entity>,
    pub signal: Signal,
    pub objectives: Vec<String>,
    pub constraints: Constraints,
    #[serde(rename = "desired_outputs")]
    pub desired_outputs: Vec<String>,
    pub confidence: f32,
    #[serde(rename = "ambiguity_flags")]
    pub ambiguity_flags: Vec<String>,
    #[serde(rename = "raw_text")]
    pub raw_text: String,
    #[serde(rename = "market_type")]
    pub market_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    #[serde(rename = "entity_type")]
    pub entity_type: String,
    pub value: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    #[serde(rename = "type")]
    pub r#type: String,
    pub theme: String,
    pub direction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    #[serde(rename = "optimization_mode")]
    pub optimization_mode: String,
    #[serde(rename = "market_scope")]
    pub market_scope: Vec<String>,
    #[serde(rename = "real_trade_execution")]
    pub real_trade_execution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetGraph {
    pub nodes: Vec<AssetGraphNode>,
    pub edges: Vec<AssetGraphEdge>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetGraphNode {
    pub id: String,
    pub symbol: String,
    #[serde(rename = "asset_class")]
    pub asset_class: String,
    #[serde(rename = "relationship_type")]
    pub relationship_type: String,
    pub confidence: f32,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask: Option<f64>,
    #[serde(rename = "spread_percent", skip_serializing_if = "Option::is_none")]
    pub spread_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetGraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(rename = "relationship_type")]
    pub relationship_type: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionDAG {
    pub nodes: Vec<DagNode>,
    pub edges: Vec<DagEdge>,
    #[serde(rename = "contention_score")]
    pub contention_score: u8,
    #[serde(rename = "parallelizability_score")]
    pub parallelizability_score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagNode {
    pub id: String,
    pub label: String,
    #[serde(rename = "execution_layer")]
    pub execution_layer: String,
    pub risk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(rename = "dependency_type")]
    pub dependency_type: String,
    pub weight: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    #[serde(rename = "liquidity_risk")]
    pub liquidity_risk: u8,
    #[serde(rename = "spread_risk")]
    pub spread_risk: u8,
    #[serde(rename = "volatility_risk")]
    pub volatility_risk: u8,
    #[serde(rename = "otc_disclosure_risk")]
    pub otc_disclosure_risk: u8,
    #[serde(rename = "contention_risk")]
    pub contention_risk: u8,
    #[serde(rename = "execution_difficulty")]
    pub execution_difficulty: u8,
    #[serde(rename = "route_confidence")]
    pub route_confidence: u8,
    pub composite: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonadProof {
    #[serde(rename = "signal_hash")]
    pub signal_hash: String,
    #[serde(rename = "graph_hash")]
    pub graph_hash: String,
    pub score: u8,
    #[serde(rename = "metadata_uri")]
    pub metadata_uri: String,
    #[serde(rename = "tx_hash", skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
}

pub fn parse_text(text: &str, mode: &str, market_type_hint: &str) -> SemanticIR {
    let trimmed = text.trim();
    let lower = trimmed.to_lowercase();
    let mode = normalize_mode(mode);

    let is_ticker = regex::Regex::new(r"^[A-Za-z]{1,5}$")
        .unwrap()
        .is_match(trimmed);

    let market_type = if is_ticker || market_type_hint == "otc" {
        "otc"
    } else {
        "cross_asset"
    };

    let mut ambiguity_flags: Vec<String> = Vec::new();
    let mut confidence: f32 = 0.92;

    let (intent, domain, signal, entities) = if market_type == "otc" {
        let ticker = trimmed.to_uppercase();
        (
            "analyze_fragmented_market_execution".into(),
            "fragmented_markets".into(),
            Signal {
                r#type: "otc_ticker".into(),
                theme: format!("{} OTC", ticker),
                direction: "neutral".into(),
                ticker: Some(ticker.clone()),
            },
            vec![Entity {
                entity_type: "ticker".into(),
                value: ticker,
                confidence: 0.95,
            }],
        )
    } else if lower.contains("ai") || lower.contains("compute") || lower.contains("inference") {
        (
            "analyze_execution_opportunity".into(),
            "asset_agnostic_trading".into(),
            Signal {
                r#type: "macro_theme".into(),
                theme: "AI infrastructure".into(),
                direction: "bullish".into(),
                ticker: None,
            },
            vec![
                Entity {
                    entity_type: "theme".into(),
                    value: "AI infrastructure".into(),
                    confidence: 0.9,
                },
                Entity {
                    entity_type: "sector".into(),
                    value: "semiconductors".into(),
                    confidence: 0.82,
                },
            ],
        )
    } else if lower.contains("oil") || lower.contains("energy") {
        (
            "analyze_execution_opportunity".into(),
            "asset_agnostic_trading".into(),
            Signal {
                r#type: "macro_theme".into(),
                theme: "Energy commodities".into(),
                direction: "neutral".into(),
                ticker: None,
            },
            vec![Entity {
                entity_type: "theme".into(),
                value: "Energy".into(),
                confidence: 0.88,
            }],
        )
    } else {
        confidence = 0.72;
        ambiguity_flags.push("broad_narrative".into());
        (
            "analyze_execution_opportunity".into(),
            "asset_agnostic_trading".into(),
            Signal {
                r#type: "macro_theme".into(),
                theme: trimmed.into(),
                direction: "neutral".into(),
                ticker: None,
            },
            vec![Entity {
                entity_type: "narrative".into(),
                value: trimmed.into(),
                confidence: 0.7,
            }],
        )
    };

    if trimmed.len() < 3 {
        ambiguity_flags.push("input_too_short".into());
        confidence = confidence.min(0.5);
    }

    SemanticIR {
        intent,
        domain,
        entities,
        signal,
        objectives: default_objectives(),
        constraints: Constraints {
            optimization_mode: mode,
            market_scope: default_market_scope(),
            real_trade_execution: false,
        },
        desired_outputs: default_desired_outputs(),
        confidence,
        ambiguity_flags,
        raw_text: trimmed.into(),
        market_type: market_type.into(),
    }
}

fn normalize_mode(mode: &str) -> String {
    match mode.to_lowercase().as_str() {
        "conservative" | "aggressive" | "balanced" => mode.to_lowercase(),
        _ => "balanced".into(),
    }
}

fn default_objectives() -> Vec<String> {
    vec![
        "discover_related_assets".into(),
        "build_execution_graph".into(),
        "score_execution_risk".into(),
        "recommend_execution_routes".into(),
    ]
}

fn default_market_scope() -> Vec<String> {
    vec![
        "equities".into(),
        "etfs".into(),
        "crypto".into(),
        "otc".into(),
        "fx".into(),
        "futures".into(),
    ]
}

fn default_desired_outputs() -> Vec<String> {
    vec![
        "semantic_ir".into(),
        "asset_graph".into(),
        "execution_dag".into(),
        "risk_scores".into(),
        "recommendations".into(),
        "monad_proof_hash".into(),
    ]
}
