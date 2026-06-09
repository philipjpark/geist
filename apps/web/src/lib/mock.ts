import type { AnalyzeResponse, SemanticIR } from "./ir";
import { inferMarketType } from "./socials";

function defaultObjectives() {
  return [
    "discover_related_assets",
    "build_execution_graph",
    "score_execution_risk",
    "recommend_execution_routes",
  ];
}

function defaultScope() {
  return ["equities", "etfs", "crypto", "otc", "fx", "futures"];
}

function defaultOutputs() {
  return [
    "semantic_ir",
    "asset_graph",
    "execution_dag",
    "risk_scores",
    "recommendations",
    "monad_proof_hash",
  ];
}

export function fallbackParse(
  text: string,
  mode = "balanced",
  marketTypeHint?: string
): { semantic_ir: SemanticIR } {
  const trimmed = text.trim();
  const marketType = marketTypeHint ?? inferMarketType(trimmed);
  const isTicker = /^[A-Za-z]{1,5}$/.test(trimmed);

  const semantic_ir: SemanticIR = isTicker || marketType === "otc"
    ? {
        intent: "analyze_fragmented_market_execution",
        domain: "fragmented_markets",
        entities: [{ entity_type: "ticker", value: trimmed.toUpperCase(), confidence: 0.95 }],
        signal: {
          type: "otc_ticker",
          theme: `${trimmed.toUpperCase()} OTC`,
          direction: "neutral",
          ticker: trimmed.toUpperCase(),
        },
        objectives: defaultObjectives(),
        constraints: {
          optimization_mode: mode,
          market_scope: defaultScope(),
          real_trade_execution: false,
        },
        desired_outputs: defaultOutputs(),
        confidence: 0.92,
        ambiguity_flags: [],
        raw_text: trimmed,
        market_type: "otc",
      }
    : {
        intent: "analyze_execution_opportunity",
        domain: "asset_agnostic_trading",
        entities: [
          { entity_type: "theme", value: "AI infrastructure", confidence: 0.9 },
          { entity_type: "sector", value: "semiconductors", confidence: 0.82 },
        ],
        signal: {
          type: "macro_theme",
          theme: trimmed.toLowerCase().includes("ai") ? "AI infrastructure" : trimmed,
          direction: "bullish",
        },
        objectives: defaultObjectives(),
        constraints: {
          optimization_mode: mode,
          market_scope: defaultScope(),
          real_trade_execution: false,
        },
        desired_outputs: defaultOutputs(),
        confidence: 0.88,
        ambiguity_flags: [],
        raw_text: trimmed,
        market_type: "cross_asset",
      };

  return { semantic_ir };
}

export function fallbackAnalyze(ir: SemanticIR): AnalyzeResponse {
  const contention = 42;
  const parallelizability = 58;

  const asset_graph =
    ir.market_type === "otc"
      ? {
          nodes: [
            {
              id: `asset-${ir.signal.ticker ?? ir.raw_text}`,
              symbol: (ir.signal.ticker ?? ir.raw_text).toUpperCase(),
              asset_class: "otc_equity",
              relationship_type: "primary_leg",
              confidence: 0.95,
              source: "mock",
              bid: 0.28,
              ask: 0.31,
              spread_percent: 10.17,
            },
            {
              id: "asset-ETH",
              symbol: "ETH",
              asset_class: "crypto",
              relationship_type: "parallel_leg",
              confidence: 0.78,
              source: "derived",
            },
          ],
          edges: [
            {
              id: "e-otc-eth",
              source: `asset-${ir.signal.ticker ?? ir.raw_text}`,
              target: "asset-ETH",
              relationship_type: "narrative_parallel",
              confidence: 0.72,
            },
          ],
          source: "mock",
        }
      : {
          nodes: [
            { id: "asset-NVDA", symbol: "NVDA", asset_class: "equity", relationship_type: "compute_proxy", confidence: 0.9, source: "theme_map" },
            { id: "asset-SMH", symbol: "SMH", asset_class: "etf", relationship_type: "sector_basket", confidence: 0.85, source: "theme_map" },
            { id: "asset-ETH", symbol: "ETH", asset_class: "crypto", relationship_type: "onchain_beta", confidence: 0.8, source: "theme_map" },
          ],
          edges: [
            { id: "e-0", source: "asset-NVDA", target: "asset-SMH", relationship_type: "correlated_upside", confidence: 0.7 },
            { id: "e-1", source: "asset-SMH", target: "asset-ETH", relationship_type: "correlated_upside", confidence: 0.7 },
          ],
          source: "mock",
        };

  const execution_dag = {
    nodes: [
      { id: "parse", label: "Semantic Parse", execution_layer: "compiler", risk: "low" },
      { id: "signal", label: ir.raw_text, execution_layer: "signal", risk: "low" },
      { id: "asset_graph", label: "Asset Graph", execution_layer: "relationship", risk: "low" },
      { id: "risk", label: "Risk Engine", execution_layer: "scoring", risk: "medium" },
      { id: "proof", label: "Monad Proof", execution_layer: "verification", risk: "low" },
    ],
    edges: [
      { id: "e0", source: "parse", target: "signal", dependency_type: "compile", weight: 2 },
      { id: "e1", source: "signal", target: "asset_graph", dependency_type: "map", weight: 2 },
      { id: "e2", source: "asset_graph", target: "risk", dependency_type: "score", weight: 2 },
      { id: "e3", source: "risk", target: "proof", dependency_type: "register", weight: 2 },
    ],
    contention_score: contention,
    parallelizability_score: parallelizability,
  };

  const risk_scores = {
    liquidity_risk: 40,
    spread_risk: 42,
    volatility_risk: 40,
    otc_disclosure_risk: ir.market_type === "otc" ? 52 : 0,
    contention_risk: contention,
    execution_difficulty: 48,
    route_confidence: 58,
    composite: 55,
  };

  return {
    asset_graph,
    execution_dag,
    risk_scores,
    recommendations: [],
    monad_proof: {
      signal_hash: "0xmocksignal0000000000000000000000000000000000000000000000000000",
      graph_hash: "0xmockgraph000000000000000000000000000000000000000000000000000000",
      score: risk_scores.composite,
      metadata_uri: `geist://${ir.market_type}/${encodeURIComponent(ir.raw_text)}`,
    },
  };
}
