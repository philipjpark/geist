export interface Entity {
  entity_type: string;
  value: string;
  confidence: number;
}

export interface Signal {
  type: string;
  theme: string;
  direction: string;
  ticker?: string;
}

export interface Constraints {
  optimization_mode: string;
  market_scope: string[];
  real_trade_execution: boolean;
}

export interface SemanticIR {
  intent: string;
  domain: string;
  entities: Entity[];
  signal: Signal;
  objectives: string[];
  constraints: Constraints;
  desired_outputs: string[];
  confidence: number;
  ambiguity_flags: string[];
  raw_text: string;
  market_type: string;
}

export interface AssetGraphNode {
  id: string;
  symbol: string;
  asset_class: string;
  relationship_type: string;
  confidence: number;
  source: string;
  bid?: number;
  ask?: number;
  spread_percent?: number;
}

export interface AssetGraphEdge {
  id: string;
  source: string;
  target: string;
  relationship_type: string;
  confidence: number;
}

export interface AssetGraph {
  nodes: AssetGraphNode[];
  edges: AssetGraphEdge[];
  source: string;
}

export interface DagNode {
  id: string;
  label: string;
  execution_layer: string;
  risk: "low" | "medium" | "high" | string;
}

export interface DagEdge {
  id: string;
  source: string;
  target: string;
  dependency_type: string;
  weight: number;
}

export interface ExecutionDAG {
  nodes: DagNode[];
  edges: DagEdge[];
  contention_score: number;
  parallelizability_score: number;
}

export interface RiskScore {
  liquidity_risk: number;
  spread_risk: number;
  volatility_risk: number;
  otc_disclosure_risk: number;
  contention_risk: number;
  execution_difficulty: number;
  route_confidence: number;
  composite: number;
}

export interface MonadProof {
  signal_hash: string;
  graph_hash: string;
  score: number;
  metadata_uri: string;
  tx_hash?: string;
}

export interface ParseResponse {
  semantic_ir: SemanticIR;
}

export interface AnalyzeResponse {
  asset_graph: AssetGraph;
  execution_dag: ExecutionDAG;
  risk_scores: RiskScore;
  recommendations: string[];
  monad_proof: MonadProof;
}

export interface PipelineState {
  raw_text: string;
  semantic_ir: SemanticIR | null;
  result: AnalyzeResponse | null;
}
