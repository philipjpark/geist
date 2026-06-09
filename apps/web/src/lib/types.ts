export type Mode = "conservative" | "balanced" | "aggressive";
export type MarketType = "cross_asset" | "otc";

export interface AssetNode {
  symbol: string;
  name: string;
  asset_type: string;
  venue: string;
  route_role: string;
  bid?: number;
  ask?: number;
  spread_percent?: number;
}

export interface GraphNodeDto {
  id: string;
  label: string;
  kind: string;
  risk: "low" | "medium" | "high";
}

export interface GraphEdgeDto {
  id: string;
  source: string;
  target: string;
  label: string;
  weight: number;
}

export interface Scores {
  parallelizability: number;
  contention: number;
  liquidityRisk: number;
  spreadRisk: number;
  otcDisclosureRisk?: number;
  routeConfidence?: number;
  executionDifficulty: number;
}

export interface AnalysisResponse {
  query: string;
  mode: Mode;
  market_type: MarketType;
  summary: string;
  assets: AssetNode[];
  nodes: GraphNodeDto[];
  edges: GraphEdgeDto[];
  scores: Scores;
  recommendations: string[];
}

export interface RedditMention {
  ticker: string;
  mentions: number;
  subreddit: string;
  sample_title: string;
}

export interface XMention {
  ticker: string;
  mentions: number;
  author: string;
  sampleText: string;
}

export interface TrendTopic {
  topic: string;
  interest: number;
  source: string;
}

export interface CorrelatedTicker {
  ticker: string;
  score: number;
  reason: string;
  legType?: string;
}

export interface AgentStep {
  step: string;
  status: string;
  detail: string;
}

export interface CryptoParallel {
  symbol: string;
  rationale: string;
}

export interface NewsItem {
  title: string;
  link: string;
  publisher: string;
  published: string;
  riskTags: string[];
  riskScore: number;
  source: string;
}

export interface DiscoverSources {
  reddit: string;
  google_trends: string;
  stocktwits: string;
  x: string;
  whatsapp: string;
  telegram: string;
  tiktok: string;
  news: string;
  agent: string;
}

export interface DiscoverResponse {
  seed: string;
  sources: DiscoverSources;
  redditMentions: RedditMention[];
  trendTopics: TrendTopic[];
  newsItems: NewsItem[];
  xMentions: XMention[];
  correlatedTickers: CorrelatedTicker[];
  agentSummary: string;
  agentSteps: AgentStep[];
  thesisInsight: string;
  cryptoParallel: CryptoParallel;
  suggestedQuery: string;
  corpusPrompt?: string;
}
