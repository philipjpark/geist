const LISTED_STOCKS = new Set([
  "AAPL",
  "AMD",
  "AMZN",
  "GOOG",
  "GOOGL",
  "META",
  "MSFT",
  "NVDA",
  "QQQ",
  "SMH",
  "SPY",
  "TSLA",
]);

export type AssetVenue = "otc" | "stock";

export interface AssetClassification {
  symbol: string;
  venue: AssetVenue;
  marketType: "otc" | "cross_asset";
}

export function classifyAsset(input: string): AssetClassification {
  const trimmed = input.trim();
  const upper = trimmed.toUpperCase();

  if (/^[A-Za-z]{1,5}$/.test(trimmed)) {
    if (LISTED_STOCKS.has(upper)) {
      return { symbol: upper, venue: "stock", marketType: "cross_asset" };
    }
    return { symbol: upper, venue: "otc", marketType: "otc" };
  }

  return { symbol: trimmed, venue: "stock", marketType: "cross_asset" };
}

const TICKER_RE = /\$?([A-Z]{2,5})\b/g;
const NOISE = new Set([
  "SEC",
  "OTC",
  "CEO",
  "FDA",
  "IPO",
  "ETF",
  "LLC",
  "INC",
  "THE",
  "FOR",
  "AND",
  "GROUP",
  "FLAGS",
  "EASED",
  "FRAUD",
  "NEWS",
  "RSS",
]);

export function isNoiseTicker(ticker: string): boolean {
  return NOISE.has(ticker.toUpperCase());
}

export function extractTickersFromText(text: string): string[] {
  const found = new Set<string>();
  for (const match of text.toUpperCase().matchAll(TICKER_RE)) {
    const t = match[1];
    if (!isNoiseTicker(t)) found.add(t);
  }
  return [...found];
}
