import { classifyAsset } from "./assets";

/** Corpus feeds selectable in discovery UI */
export const CORPUS_SOURCES = [
  { id: "reddit", label: "Reddit" },
  { id: "google_trends", label: "Trends" },
  { id: "news", label: "News RSS" },
  { id: "x", label: "X" },
] as const;

export type CorpusSourceId = (typeof CORPUS_SOURCES)[number]["id"];

export const SOCIAL_SOURCES = CORPUS_SOURCES;

export type SocialSourceId = CorpusSourceId;

export const DEFAULT_SOCIAL_SOURCES: SocialSourceId[] = [
  "reddit",
  "google_trends",
  "news",
  "x",
];

export function inferMarketType(query: string): "cross_asset" | "otc" {
  return classifyAsset(query).marketType;
}
