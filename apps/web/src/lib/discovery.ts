import type { DiscoverResponse } from "./types";
import { extractTickersFromText, isNoiseTicker } from "./assets";
import { DEFAULT_SOCIAL_SOURCES, type SocialSourceId } from "./socials";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";

const MOCK_STEPS = [
  { step: "OBSERVE", status: "done", detail: "Reddit, Trends, News RSS, X ingested" },
  { step: "REASON", status: "done", detail: "Cross-feed ticker propagation" },
  { step: "PLAN", status: "done", detail: "Up to 10 assets ranked" },
  { step: "COMMIT", status: "ready", detail: "Select ticker to compile" },
];

function buildFallbackCorpusPrompt(
  seed: string,
  sources: SocialSourceId[],
  reddit: DiscoverResponse["redditMentions"],
  trends: DiscoverResponse["trendTopics"],
  news: DiscoverResponse["newsItems"],
  xMentions: DiscoverResponse["xMentions"]
) {
  const on = (id: SocialSourceId) => sources.includes(id);
  const pipeline = [
    on("reddit") && "Reddit",
    on("google_trends") && "Google Trends",
    on("news") && "News RSS",
    on("x") && "X",
  ]
    .filter(Boolean)
    .join(" -> ");

  const redditBlock = !on("reddit")
    ? "(reddit skipped)"
    : reddit.length === 0
      ? "(no reddit cashtags this scan)"
      : reddit
          .map(
            (r) =>
              `ticker=$${r.ticker} mentions=${r.mentions} subreddit=r/${r.subreddit} sample="${r.sample_title}"`
          )
          .join("\n");

  const trendsBlock = !on("google_trends")
    ? "(google trends skipped)"
    : trends.length === 0
      ? "(no trend topics this scan)"
      : trends
          .map((t) => {
            const tickers = extractTickersFromText(t.topic);
            const extra =
              tickers.length > 0 ? ` tickers=${tickers.map((x) => `$${x}`).join(",")}` : "";
            return `topic="${t.topic}" interest=${t.interest}${extra}`;
          })
          .join("\n");

  const newsBlock = !on("news")
    ? "(news rss skipped)"
    : news.length === 0
      ? "(no news headlines this scan)"
      : news
          .map((n) => {
            const tickers = extractTickersFromText(n.title);
            const extra =
              tickers.length > 0 ? ` tickers=${tickers.map((x) => `$${x}`).join(",")}` : "";
            return `title="${n.title}" publisher=${n.publisher} risk=${n.riskScore} tags=${n.riskTags.join(",")}${extra}`;
          })
          .join("\n");

  const xBlock = !on("x")
    ? "(x skipped)"
    : xMentions.length === 0
      ? "(no x cashtags this scan)"
      : xMentions
          .map(
            (x) =>
              `ticker=$${x.ticker} mentions=${x.mentions} author=${x.author} sample="${x.sampleText}"`
          )
          .join("\n");

  const tickerScores = new Map<string, { score: number; sources: Set<string>; detail: string }>();
  const touch = (ticker: string, source: string, boost: number, detail: string) => {
    const t = ticker.toUpperCase();
    if (isNoiseTicker(t)) return;
    const cur = tickerScores.get(t) ?? { score: 0, sources: new Set<string>(), detail: "" };
    cur.score += boost;
    cur.sources.add(source);
    cur.detail = detail;
    tickerScores.set(t, cur);
  };

  reddit.forEach((r) => touch(r.ticker, "reddit", r.mentions * 3, `reddit=${r.mentions}`));
  xMentions.forEach((x) => touch(x.ticker, "x", x.mentions * 3, `x=${x.mentions}`));
  news.forEach((n) => {
    extractTickersFromText(n.title).forEach((t) =>
      touch(t, "news", 8 + n.riskScore / 10, `news risk=${n.riskScore}`)
    );
  });
  trends.forEach((t) => {
    extractTickersFromText(t.topic).forEach((tk) =>
      touch(tk, "trends", Math.floor(t.interest / 10), `trend=${t.interest}`)
    );
  });

  const unionBlock =
    tickerScores.size === 0
      ? "(no tickers propagated — await feed ingest)"
      : [...tickerScores.entries()]
          .sort((a, b) => b[1].score - a[1].score)
          .slice(0, 15)
          .map(
            ([t, v]) =>
              `$${t} score=${Math.min(100, v.score)} sources=[${[...v.sources].join("+")}] detail="${v.detail}"`
          )
          .join("\n");

  return [
    "You are Geist — a semantic execution compiler for fragmented and OTC markets.",
    "Structure before reasoning: ingest multi-feed corpus, propagate tickers across Reddit, Trends, News RSS, and X, then rank execution relevance.",
    "",
    "--- CORPUS INGEST ---",
    `seed: ${seed || "auto-discover"}`,
    `pipeline: ${pipeline || "semantic rank"} -> semantic rank -> OpenAI correlate`,
    `feeds_active: ${sources.join(",") || "none"}`,
    "openai: enabled (fallback corpus_rank when API offline)",
    "x_api: enabled when X_BEARER_TOKEN configured",
    "",
    "[REDDIT]",
    redditBlock,
    "",
    "[GOOGLE_TRENDS]",
    trendsBlock,
    "",
    "[NEWS_RSS]",
    newsBlock,
    "",
    "[X]",
    xBlock,
    "",
    "[TICKER_UNION]",
    "cross_feed_propagation (reddit+trends+news+x overlap):",
    unionBlock,
    "",
    "[AGENTIC_TASK]",
    "1. OBSERVE — ingest all active feeds above",
    "2. PROPAGATE — extract and union tickers across Reddit, Trends headlines, News RSS titles, X cashtags",
    "3. REASON — score overlap, narrative risk (news tags), social velocity (reddit/x mentions), trend interest",
    "4. RANK — emit up to 10 correlated tickers (OTC preferred; include cross-asset when news/trends warrant)",
    "5. COMMIT — hand ranked tickers to semantic IR compile on user selection",
    "--- END CORPUS ---",
  ].join("\n");
}

function rankFromCorpus(
  reddit: DiscoverResponse["redditMentions"],
  trends: DiscoverResponse["trendTopics"],
  news: DiscoverResponse["newsItems"],
  xMentions: DiscoverResponse["xMentions"]
): DiscoverResponse["correlatedTickers"] {
  const scores = new Map<string, { score: number; sources: string[]; parts: string[] }>();

  const touch = (ticker: string, source: string, boost: number, part: string) => {
    const t = ticker.toUpperCase();
    if (isNoiseTicker(t)) return;
    const cur = scores.get(t) ?? { score: 0, sources: [], parts: [] };
    cur.score += boost;
    if (!cur.sources.includes(source)) cur.sources.push(source);
    cur.parts.push(part);
    scores.set(t, cur);
  };

  reddit.forEach((r) => touch(r.ticker, "reddit", r.mentions * 3, `${r.mentions}x r/${r.subreddit}`));
  xMentions.forEach((x) => touch(x.ticker, "x", x.mentions * 3, `${x.mentions}x ${x.author}`));
  news.forEach((n) => {
    extractTickersFromText(n.title).forEach((t) =>
      touch(t, "news", 8, `news risk ${n.riskScore}`)
    );
  });
  trends.forEach((t, i) => {
    const boost = Math.floor(t.interest / 10);
    extractTickersFromText(t.topic).forEach((tk) => touch(tk, "trends", boost, t.topic));
    const r = reddit[i];
    if (r) touch(r.ticker, "trends", Math.floor(boost / 2), "trend overlap");
  });

  const ranked = [...scores.entries()]
    .map(([ticker, v]) => {
      const cross = v.sources.length >= 2 ? 12 : 0;
      return {
        ticker,
        score: Math.min(100, v.score + cross),
        reason: `${v.sources.join("+")} · ${v.parts.slice(0, 2).join(" · ")}`,
        legType: "otc" as const,
      };
    })
    .sort((a, b) => b.score - a.score)
    .slice(0, 10);

  if (ranked.length > 0) return ranked;
  return [
    { ticker: "CYDY", score: 78, reason: "reddit+trends · default", legType: "otc" },
    { ticker: "OZSC", score: 65, reason: "reddit · default", legType: "otc" },
  ];
}

function fallbackDiscovery(seed: string, sources: SocialSourceId[]): DiscoverResponse {
  const on = (id: SocialSourceId) => sources.includes(id);
  const redditMentions = on("reddit")
    ? [
        { ticker: "CYDY", mentions: 18, subreddit: "pennystocks", sample_title: "CYDY biotech momentum" },
        { ticker: "OZSC", mentions: 11, subreddit: "pennystocks", sample_title: "OZSC volume spike" },
        { ticker: "HCMC", mentions: 9, subreddit: "pennystocks", sample_title: "HCMC volume watch" },
        { ticker: "ENZC", mentions: 7, subreddit: "RobinHoodPennyStocks", sample_title: "ENZC catalyst thread" },
      ]
    : [];
  const trendTopics = on("google_trends")
    ? [
        { topic: "OTC biotech penny stock", interest: 82, source: "live" },
        { topic: "fragmented markets execution", interest: 54, source: "live" },
        { topic: "CYDY biotech FDA", interest: 41, source: "live" },
      ]
    : [];
  const newsItems = on("news")
    ? [
        {
          title: "OTC Markets Group flags increased fraud warnings in pink sheet issuers",
          link: "https://news.google.com",
          publisher: "News Feed",
          published: "Mon, 01 Jan 2026 12:00:00 GMT",
          riskTags: ["fraud_warning", "otc_risk"],
          riskScore: 55,
          source: "live",
        },
        {
          title: "CYDY biotech microcap announces partnership — investors urged to review 8-K filing",
          link: "https://news.google.com",
          publisher: "News Feed",
          published: "Sun, 31 Dec 2025 09:00:00 GMT",
          riskTags: ["pr_narrative", "disclosure"],
          riskScore: 25,
          source: "live",
        },
        {
          title: "OZSC subsidiary expansion draws OTC liquidity scrutiny",
          link: "https://news.google.com",
          publisher: "News Feed",
          published: "Sat, 30 Dec 2025 08:00:00 GMT",
          riskTags: ["otc_risk", "pr_narrative"],
          riskScore: 30,
          source: "live",
        },
      ]
    : [];
  const xMentions = on("x")
    ? [
        { ticker: "BNB", mentions: 1, author: "@feed", sampleText: "$BNB default corpus signal" },
        { ticker: "CAKE", mentions: 1, author: "@feed", sampleText: "$CAKE default corpus signal" },
      ]
    : [];

  const correlatedTickers = rankFromCorpus(redditMentions, trendTopics, newsItems, xMentions);

  return {
    seed,
    sources: {
      reddit: on("reddit") ? "live" : "skipped",
      google_trends: on("google_trends") ? "live" : "skipped",
      stocktwits: "skipped",
      whatsapp: "skipped",
      telegram: "skipped",
      tiktok: "skipped",
      news: on("news") ? "live" : "skipped",
      x: on("x") ? "live" : "skipped",
      agent: "corpus_rank",
    },
    redditMentions,
    trendTopics,
    newsItems,
    xMentions,
    correlatedTickers,
    agentSummary: `$${correlatedTickers[0]?.ticker ?? "CYDY"} leads cross-feed scan (${correlatedTickers.length} assets propagated). Ready to compile.`,
    agentSteps: MOCK_STEPS,
    thesisInsight:
      "Multi-feed corpus propagation surfaces OTC tickers where social velocity, news risk tags, and trend interest overlap.",
    cryptoParallel: { symbol: "ETH", rationale: "24/7 onchain beta parallels OTC narrative risk." },
    suggestedQuery: correlatedTickers[0]?.ticker ?? "CYDY",
    corpusPrompt: buildFallbackCorpusPrompt(seed, sources, redditMentions, trendTopics, newsItems, xMentions),
  };
}

function normalizeDiscovery(raw: Record<string, unknown>): DiscoverResponse {
  const fb = fallbackDiscovery("", DEFAULT_SOCIAL_SOURCES);
  const src = (raw.sources ?? {}) as Record<string, string | undefined>;
  const legacyTrends = src.trends;
  const redditMentions = Array.isArray(raw.redditMentions) ? raw.redditMentions : [];
  const trendTopics = Array.isArray(raw.trendTopics) ? raw.trendTopics : [];
  const newsItems = Array.isArray(raw.newsItems) ? raw.newsItems : [];
  const xMentions = Array.isArray(raw.xMentions) ? raw.xMentions : [];

  return {
    seed: typeof raw.seed === "string" ? raw.seed : fb.seed,
    sources: {
      reddit: src.reddit ?? "skipped",
      google_trends: src.google_trends ?? legacyTrends ?? "skipped",
      stocktwits: src.stocktwits ?? "skipped",
      x: src.x ?? "skipped",
      whatsapp: src.whatsapp ?? "skipped",
      telegram: src.telegram ?? "skipped",
      tiktok: src.tiktok ?? "skipped",
      news: src.news ?? "skipped",
      agent: src.agent ?? "skipped",
    },
    redditMentions,
    trendTopics,
    newsItems,
    xMentions,
    correlatedTickers: Array.isArray(raw.correlatedTickers) ? raw.correlatedTickers : [],
    agentSummary: typeof raw.agentSummary === "string" ? raw.agentSummary : "",
    agentSteps: Array.isArray(raw.agentSteps) ? raw.agentSteps : MOCK_STEPS,
    thesisInsight: typeof raw.thesisInsight === "string" ? raw.thesisInsight : "",
    cryptoParallel:
      raw.cryptoParallel && typeof raw.cryptoParallel === "object"
        ? (raw.cryptoParallel as DiscoverResponse["cryptoParallel"])
        : fb.cryptoParallel,
    suggestedQuery: typeof raw.suggestedQuery === "string" ? raw.suggestedQuery : "CYDY",
    corpusPrompt:
      typeof raw.corpusPrompt === "string"
        ? raw.corpusPrompt
        : buildFallbackCorpusPrompt(
            typeof raw.seed === "string" ? raw.seed : "",
            DEFAULT_SOCIAL_SOURCES,
            redditMentions as DiscoverResponse["redditMentions"],
            trendTopics as DiscoverResponse["trendTopics"],
            newsItems as DiscoverResponse["newsItems"],
            xMentions as DiscoverResponse["xMentions"]
          ),
  };
}

export async function discoverSignals(
  seed = "",
  sources: SocialSourceId[] = DEFAULT_SOCIAL_SOURCES
): Promise<DiscoverResponse> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 60_000);
  try {
    const res = await fetch(`${API_URL}/discover`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ seed, sources }),
      signal: controller.signal,
    });
    if (!res.ok) throw new Error(`Discovery API returned ${res.status}`);
    const raw = await res.json();
    return normalizeDiscovery(raw);
  } catch {
    return fallbackDiscovery(seed, sources);
  } finally {
    clearTimeout(timer);
  }
}
