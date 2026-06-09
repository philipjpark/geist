use sha2::{Digest, Sha256};

use crate::ir::{
    AnalyzeResponse, AssetGraph, AssetGraphEdge, AssetGraphNode, DagEdge, DagNode, ExecutionDAG,
    MonadProof, RiskScore, SemanticIR,
};
use crate::massive::{fetch_otc_data, mock_otc_data, OtcMarketData};

pub async fn analyze_from_ir(ir: &SemanticIR) -> AnalyzeResponse {
    if ir.market_type == "otc" {
        analyze_otc_ir(ir).await
    } else {
        analyze_cross_asset_ir(ir)
    }
}

async fn analyze_otc_ir(ir: &SemanticIR) -> AnalyzeResponse {
    let ticker = ir
        .signal
        .ticker
        .clone()
        .or_else(|| ir.entities.first().map(|e| e.value.clone()))
        .unwrap_or_else(|| ir.raw_text.to_uppercase());

    let otc = fetch_otc_data(&ticker)
        .await
        .unwrap_or_else(|| mock_otc_data(&ticker));
    let mode = &ir.constraints.optimization_mode;

    let spread_risk = score_spread_risk(otc.spread_percent);
    let liquidity_risk = score_liquidity_risk(otc.volume, otc.volatility);
    let volatility_risk = (otc.volatility * 0.6).min(100.0) as u8;
    let disclosure_risk = score_disclosure_risk(otc.disclosure_quality);
    let contention = score_otc_contention(&otc, mode);
    let route_confidence = score_route_confidence(&otc, mode);
    let parallelizability = 100_u8.saturating_sub(contention);
    let execution_difficulty =
        weighted_execution_difficulty(liquidity_risk, spread_risk, contention, disclosure_risk);

    let asset_graph = build_otc_asset_graph(&otc, &ticker);
    let execution_dag = build_otc_dag(&ticker, &otc, contention, spread_risk, liquidity_risk);
    let risk_scores = RiskScore {
        liquidity_risk,
        spread_risk,
        volatility_risk,
        otc_disclosure_risk: disclosure_risk,
        contention_risk: contention,
        execution_difficulty,
        route_confidence,
        composite: composite_score(
            parallelizability,
            contention,
            liquidity_risk,
            spread_risk,
            execution_difficulty,
        ),
    };

    let monad_proof = build_monad_proof(ir, &execution_dag, risk_scores.composite);

    AnalyzeResponse {
        asset_graph,
        execution_dag,
        risk_scores,
        recommendations: vec![],
        monad_proof,
    }
}

fn analyze_cross_asset_ir(ir: &SemanticIR) -> AnalyzeResponse {
    let mode = &ir.constraints.optimization_mode;
    let theme = &ir.signal.theme;
    let asset_graph = build_cross_asset_graph(theme, &ir.signal);
    let contention = score_contention(&asset_graph.nodes, mode);
    let spread_risk = match mode.as_str() {
        "conservative" => 28,
        "aggressive" => 58,
        _ => 42,
    };
    let liquidity_risk = match mode.as_str() {
        "conservative" => 30,
        "aggressive" => 55,
        _ => 40,
    };
    let volatility_risk = match ir.signal.direction.as_str() {
        "bullish" => 48,
        "bearish" => 62,
        _ => 40,
    };
    let parallelizability = 100_u8.saturating_sub(contention);
    let execution_difficulty =
        weighted_execution_difficulty(liquidity_risk, spread_risk, contention, 25);

    let execution_dag = build_cross_asset_dag(theme, &asset_graph, contention);
    let risk_scores = RiskScore {
        liquidity_risk,
        spread_risk,
        volatility_risk,
        otc_disclosure_risk: 0,
        contention_risk: contention,
        execution_difficulty,
        route_confidence: parallelizability,
        composite: composite_score(
            parallelizability,
            contention,
            liquidity_risk,
            spread_risk,
            execution_difficulty,
        ),
    };

    let monad_proof = build_monad_proof(ir, &execution_dag, risk_scores.composite);

    AnalyzeResponse {
        asset_graph,
        execution_dag,
        risk_scores,
        recommendations: vec![],
        monad_proof,
    }
}

fn build_otc_asset_graph(otc: &OtcMarketData, ticker: &str) -> AssetGraph {
    let nodes = vec![
        ag_node(
            &format!("asset-{}", ticker),
            &otc.symbol,
            "otc_equity",
            "primary_leg",
            0.95,
            &otc.source,
            Some(otc.bid),
            Some(otc.ask),
            Some(otc.spread_percent),
        ),
        ag_node("asset-ETH", "ETH", "crypto", "parallel_leg", 0.78, "derived", None, None, None),
    ];
    let edges = vec![AssetGraphEdge {
        id: "e-otc-eth".into(),
        source: format!("asset-{}", ticker),
        target: "asset-ETH".into(),
        relationship_type: "narrative_parallel".into(),
        confidence: 0.72,
    }];
    AssetGraph {
        nodes,
        edges,
        source: otc.source.clone(),
    }
}

fn build_cross_asset_graph(theme: &str, signal: &crate::ir::Signal) -> AssetGraph {
    let lower = theme.to_lowercase();
    let nodes: Vec<AssetGraphNode> = if lower.contains("ai") || lower.contains("infrastructure") {
        vec![
            ag_node("asset-NVDA", "NVDA", "equity", "compute_proxy", 0.9, "theme_map", None, None, None),
            ag_node("asset-SMH", "SMH", "etf", "sector_basket", 0.85, "theme_map", None, None, None),
            ag_node("asset-ETH", "ETH", "crypto", "onchain_beta", 0.8, "theme_map", None, None, None),
            ag_node("asset-USDC", "USDC/MON", "onchain_pool", "settlement", 0.88, "theme_map", None, None, None),
        ]
    } else if lower.contains("energy") || lower.contains("oil") {
        vec![
            ag_node("asset-WTI", "WTI", "futures", "commodity_leg", 0.88, "theme_map", None, None, None),
            ag_node("asset-XLE", "XLE", "etf", "sector_basket", 0.84, "theme_map", None, None, None),
            ag_node("asset-MON", "MON/USDC", "onchain_pool", "settlement", 0.86, "theme_map", None, None, None),
        ]
    } else {
        vec![
            ag_node("asset-SPY", "SPY", "etf", "market_beta", 0.75, "default_map", None, None, None),
            ag_node("asset-ETH", "ETH", "crypto", "onchain_beta", 0.7, "default_map", None, None, None),
            ag_node("asset-MON", "MON", "crypto", "coordination", 0.82, "default_map", None, None, None),
        ]
    };

    let edges: Vec<AssetGraphEdge> = nodes
        .windows(2)
        .enumerate()
        .map(|(i, pair)| AssetGraphEdge {
            id: format!("e-ag-{}", i),
            source: pair[0].id.clone(),
            target: pair[1].id.clone(),
            relationship_type: if signal.direction == "bullish" {
                "correlated_upside".into()
            } else {
                "correlated_exposure".into()
            },
            confidence: 0.7,
        })
        .collect();

    AssetGraph {
        nodes,
        edges,
        source: "deterministic_theme_map".into(),
    }
}

fn ag_node(
    id: &str,
    symbol: &str,
    asset_class: &str,
    relationship_type: &str,
    confidence: f32,
    source: &str,
    bid: Option<f64>,
    ask: Option<f64>,
    spread_percent: Option<f64>,
) -> AssetGraphNode {
    AssetGraphNode {
        id: id.into(),
        symbol: symbol.into(),
        asset_class: asset_class.into(),
        relationship_type: relationship_type.into(),
        confidence,
        source: source.into(),
        bid,
        ask,
        spread_percent,
    }
}

fn build_otc_dag(
    ticker: &str,
    otc: &OtcMarketData,
    contention: u8,
    spread_risk: u8,
    liquidity_risk: u8,
) -> ExecutionDAG {
    let nodes = vec![
        dag_node("parse", "Semantic Parse", "compiler", "low"),
        dag_node("signal", ticker, "signal", "medium"),
        dag_node("otc_venue", "OTC Venue", "discovery", risk_label(spread_risk)),
        dag_node("liquidity", &format!("Pool {:.0}", otc.volume), "liquidity", risk_label(liquidity_risk)),
        dag_node("asset", &format!("{} ${:.3}", otc.symbol, otc.bid), "routing", risk_label(spread_risk)),
        dag_node("crypto_par", "ETH parallel", "routing", "medium"),
        dag_node("risk", "Risk Engine", "scoring", risk_label(contention)),
        dag_node("proof", "Monad Proof", "verification", "low"),
    ];
    let edges = vec![
        dag_edge("e0", "parse", "signal", "compile", 2),
        dag_edge("e1", "signal", "otc_venue", "discover", 2),
        dag_edge("e2", "signal", "crypto_par", "parallel", 2),
        dag_edge("e3", "otc_venue", "liquidity", "depth_scan", 2),
        dag_edge("e4", "liquidity", "asset", "quote", 2),
        dag_edge("e5", "asset", "risk", "score", 1),
        dag_edge("e6", "crypto_par", "risk", "score", 1),
        dag_edge("e7", "risk", "proof", "register", 2),
    ];
    ExecutionDAG {
        nodes,
        edges,
        contention_score: contention,
        parallelizability_score: 100_u8.saturating_sub(contention),
    }
}

fn build_cross_asset_dag(theme: &str, graph: &AssetGraph, contention: u8) -> ExecutionDAG {
    let mut nodes = vec![
        dag_node("parse", "Semantic Parse", "compiler", "low"),
        dag_node("signal", theme, "signal", "low"),
        dag_node("asset_graph", "Asset Graph", "relationship", "low"),
        dag_node("risk", "Risk Engine", "scoring", risk_label(contention)),
        dag_node("proof", "Monad Proof", "verification", "low"),
    ];
    let mut edges = vec![
        dag_edge("e0", "parse", "signal", "compile", 2),
        dag_edge("e1", "signal", "asset_graph", "map", 2),
        dag_edge("e2", "asset_graph", "risk", "score", 2),
        dag_edge("e3", "risk", "proof", "register", 2),
    ];

    for (i, n) in graph.nodes.iter().enumerate() {
        let id = format!("route-{}", i);
        nodes.push(dag_node(
            &id,
            &n.symbol,
            "execution",
            if n.asset_class == "crypto" { "medium" } else { "low" },
        ));
        edges.push(dag_edge(
            &format!("e-route-{}", i),
            "asset_graph",
            &id,
            "route",
            1,
        ));
        edges.push(dag_edge(
            &format!("e-route-risk-{}", i),
            &id,
            "risk",
            "aggregate",
            1,
        ));
    }

    ExecutionDAG {
        nodes,
        edges,
        contention_score: contention,
        parallelizability_score: 100_u8.saturating_sub(contention),
    }
}

fn dag_node(id: &str, label: &str, layer: &str, risk: &str) -> DagNode {
    DagNode {
        id: id.into(),
        label: label.into(),
        execution_layer: layer.into(),
        risk: risk.into(),
    }
}

fn dag_edge(id: &str, source: &str, target: &str, dep: &str, weight: u8) -> DagEdge {
    DagEdge {
        id: id.into(),
        source: source.into(),
        target: target.into(),
        dependency_type: dep.into(),
        weight,
    }
}

fn build_monad_proof(ir: &SemanticIR, dag: &ExecutionDAG, score: u8) -> MonadProof {
    let signal_hash = hash_hex(&serde_json::to_string(ir).unwrap_or_default());
    let graph_hash = hash_hex(&serde_json::to_string(dag).unwrap_or_default());
    let metadata_uri = format!(
        "geist://{}/{}/{}",
        ir.market_type,
        ir.intent,
        urlencoding_simple(&ir.raw_text)
    );

    MonadProof {
        signal_hash,
        graph_hash,
        score,
        metadata_uri,
        tx_hash: None,
    }
}

fn urlencoding_simple(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".into(),
            '/' => "%2F".into(),
            _ if c.is_ascii_alphanumeric() || c == '-' || c == '_' => c.to_string(),
            _ => "%".into(),
        })
        .collect()
}

fn hash_hex(data: &str) -> String {
    let digest = Sha256::digest(data.as_bytes());
    format!("0x{}", digest.iter().map(|b| format!("{:02x}", b)).collect::<String>())
}

fn composite_score(
    parallelizability: u8,
    contention: u8,
    liquidity: u8,
    spread: u8,
    difficulty: u8,
) -> u8 {
    let favorable = parallelizability as f64 + (100 - contention) as f64 + (100 - difficulty) as f64;
    let risk_penalty = (liquidity as f64 + spread as f64) / 2.0;
    ((favorable / 3.0) - risk_penalty * 0.15)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn score_spread_risk(spread_percent: f64) -> u8 {
    (spread_percent * 20.0).min(100.0) as u8
}

fn score_liquidity_risk(volume: f64, volatility: f64) -> u8 {
    let low_volume_penalty = if volume < 100_000.0 {
        45.0
    } else if volume < 500_000.0 {
        28.0
    } else {
        12.0
    };
    (low_volume_penalty + volatility * 0.4).min(100.0) as u8
}

fn score_disclosure_risk(disclosure_quality: f64) -> u8 {
    (100.0 - disclosure_quality).min(100.0).max(0.0) as u8
}

fn score_route_confidence(otc: &OtcMarketData, mode: &str) -> u8 {
    let base = 100_u8
        .saturating_sub(score_spread_risk(otc.spread_percent) / 2)
        .saturating_sub(score_liquidity_risk(otc.volume, otc.volatility) / 3);
    match mode {
        "aggressive" => base.saturating_sub(12),
        "conservative" => base.saturating_add(8).min(100),
        _ => base,
    }
}

fn score_otc_contention(otc: &OtcMarketData, mode: &str) -> u8 {
    let base = score_spread_risk(otc.spread_percent) / 2
        + score_liquidity_risk(otc.volume, otc.volatility) / 3
        + score_disclosure_risk(otc.disclosure_quality) / 4;
    match mode {
        "conservative" => base.saturating_sub(10),
        "aggressive" => (base + 15).min(100),
        _ => base,
    }
}

fn score_contention(nodes: &[AssetGraphNode], mode: &str) -> u8 {
    let crypto = nodes.iter().filter(|n| n.asset_class == "crypto").count() as u8;
    let onchain = nodes
        .iter()
        .filter(|n| n.asset_class == "onchain_pool")
        .count() as u8;
    let base = crypto * 10 + onchain * 8 + nodes.len() as u8 * 3;
    match mode {
        "conservative" => base.saturating_sub(12).min(100),
        "aggressive" => (base + 15).min(100),
        _ => base.min(100),
    }
}

fn weighted_execution_difficulty(
    liquidity_risk: u8,
    spread_risk: u8,
    contention: u8,
    disclosure_risk: u8,
) -> u8 {
    let weighted = (liquidity_risk as f64 * 0.35)
        + (spread_risk as f64 * 0.30)
        + (contention as f64 * 0.25)
        + (disclosure_risk as f64 * 0.10);
    weighted.round().min(100.0) as u8
}

fn risk_label(score: u8) -> &'static str {
    match score {
        0..=30 => "low",
        31..=65 => "medium",
        _ => "high",
    }
}
