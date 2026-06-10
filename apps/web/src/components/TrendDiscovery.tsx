"use client";

import type { ReactNode } from "react";
import type { DiscoverResponse, TrendTopic } from "@/lib/types";
import { extractTickersFromText } from "@/lib/assets";
import { TermHint } from "@/components/TermHint";
import { SocialSources } from "@/components/SocialSources";
import type { CorpusSourceId } from "@/lib/socials";

function riskTagColor(tag: string) {
  if (tag === "fraud_warning" || tag === "sec_action" || tag === "market_halt") {
    return "bg-red-100 text-red-800";
  }
  if (tag === "dilution" || tag === "otc_risk") return "bg-rh-warning/20 text-rh-warning";
  if (tag === "disclosure" || tag === "pr_narrative") return "bg-rh-green/15 text-rh-green";
  return "bg-rh-surface-2 text-rh-muted";
}

export function TrendDiscovery({
  discovery,
  loading,
  selectedTicker,
  corpusSources,
  setCorpusSources,
  onScan,
  onSelectTicker,
}: {
  discovery: DiscoverResponse | null;
  loading: boolean;
  selectedTicker: string | null;
  corpusSources: CorpusSourceId[];
  setCorpusSources: (v: CorpusSourceId[]) => void;
  onScan: () => void;
  onSelectTicker: (ticker: string) => void;
}) {
  const showReddit = corpusSources.includes("reddit");
  const showTrends = corpusSources.includes("google_trends");
  const showNews = corpusSources.includes("news");
  const showX = corpusSources.includes("x");
  const columnCount = [showReddit, showTrends, showNews, showX].filter(Boolean).length;

  return (
    <section className="rounded-lg border border-rh-border bg-rh-surface p-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <div className="font-mono text-[10px] font-bold uppercase tracking-widest text-rh-muted">
            <TermHint term="discovery" />
          </div>
        </div>
        <button
          type="button"
          onClick={onScan}
          disabled={loading}
          className="rounded-full bg-rh-green px-5 py-2 font-mono text-sm font-bold text-rh-on-green transition hover:bg-rh-green-hover disabled:opacity-50"
        >
          {loading ? "Scanning socials…" : "Scan socials"}
        </button>
      </div>

      <div className="mt-3">
        <SocialSources selected={corpusSources} onChange={setCorpusSources} />
      </div>

      {!loading && !discovery && (
        <p className="mt-3 rounded-lg border border-dashed border-rh-border bg-rh-canvas px-3 py-4 font-mono text-xs text-rh-muted">
          No scan yet — run discovery to surface tickers, then select one to compile.
        </p>
      )}

      {discovery && (
        <>
          <div
            className="mt-3 grid gap-3"
            style={{
              gridTemplateColumns: `repeat(${Math.max(columnCount, 1)}, minmax(0, 1fr))`,
            }}
          >
            {showReddit && (
              <ScanColumn title="Reddit">
                {(discovery.redditMentions ?? []).map((r) => (
                  <TickerRow
                    key={`${r.ticker}-${r.subreddit}`}
                    ticker={r.ticker}
                    meta={`${r.mentions}x · r/${r.subreddit}`}
                    selected={selectedTicker === r.ticker}
                    onSelect={onSelectTicker}
                  />
                ))}
              </ScanColumn>
            )}

            {showTrends && (
              <ScanColumn title="Trends">
                {(discovery.trendTopics ?? []).map((t) => (
                  <TrendRow key={t.topic} topic={t} onSelectTicker={onSelectTicker} />
                ))}
              </ScanColumn>
            )}

            {showNews && (
              <ScanColumn title="News RSS">
                {(discovery.newsItems ?? []).slice(0, 5).map((n, i) => {
                  const tickers = extractTickersFromText(n.title);
                  return (
                    <div
                      key={`${n.title}-${i}`}
                      className="rounded-md border border-rh-border bg-rh-canvas px-2 py-1.5"
                    >
                      <div className="line-clamp-2 text-[11px] text-rh-ink">{n.title}</div>
                      <div className="mt-1 flex flex-wrap items-center gap-1">
                        <span className="text-[10px] font-bold text-rh-warning">{n.riskScore}</span>
                        {n.riskTags?.map((tag) => (
                          <span
                            key={tag}
                            className={`rounded px-1 py-0.5 text-[8px] font-semibold uppercase ${riskTagColor(tag)}`}
                          >
                            {tag.replace(/_/g, " ")}
                          </span>
                        ))}
                        {tickers.map((t) => (
                          <button
                            key={t}
                            type="button"
                            onClick={() => onSelectTicker(t)}
                            className={`inline-flex items-center gap-0.5 rounded border px-1 py-0.5 font-mono text-[9px] font-bold transition ${
                              selectedTicker === t
                                ? "border-rh-green bg-rh-green/15 text-rh-green"
                                : "border-rh-border text-rh-muted hover:border-rh-green"
                            }`}
                          >
                            ${t}
                          </button>
                        ))}
                      </div>
                    </div>
                  );
                })}
              </ScanColumn>
            )}

            {showX && (
              <ScanColumn title="X">
                {(discovery.xMentions ?? []).map((x) => (
                  <TickerRow
                    key={`${x.ticker}-${x.author}`}
                    ticker={x.ticker}
                    meta={`${x.mentions}x · ${x.author}`}
                    selected={selectedTicker === x.ticker}
                    onSelect={onSelectTicker}
                  />
                ))}
              </ScanColumn>
            )}
          </div>

          {discovery.corpusPrompt && (
            <details className="mt-3 rounded-lg border border-rh-border bg-rh-canvas">
              <summary className="cursor-pointer px-3 py-2 font-mono text-[10px] font-semibold uppercase tracking-widest text-rh-muted">
                Corpus system prompt
              </summary>
              <pre className="max-h-80 overflow-auto whitespace-pre-wrap px-3 pb-3 font-mono text-[10px] leading-relaxed text-rh-muted">
                {discovery.corpusPrompt}
              </pre>
            </details>
          )}
        </>
      )}
    </section>
  );
}

function TrendRow({
  topic,
  onSelectTicker,
}: {
  topic: TrendTopic;
  onSelectTicker: (ticker: string) => void;
}) {
  const tickers = extractTickersFromText(topic.topic);
  return (
    <div className="rounded-md border border-rh-border bg-rh-canvas px-2 py-1.5">
      <div className="truncate text-[11px] text-rh-ink">{topic.topic}</div>
      <div className="mt-1 flex items-center gap-2">
        <div className="h-1 flex-1 overflow-hidden rounded-full bg-rh-surface">
          <div
            className="h-full rounded-full bg-rh-warning"
            style={{ width: `${topic.interest}%` }}
          />
        </div>
        <span className="text-[10px] text-rh-muted">{topic.interest}</span>
      </div>
      {tickers.length > 0 && (
        <div className="mt-1.5 flex flex-wrap gap-1">
          {tickers.map((t) => (
            <button
              key={t}
              type="button"
              onClick={() => onSelectTicker(t)}
              className="font-mono text-[9px] font-bold text-rh-green hover:underline"
            >
              ${t}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function ScanColumn({ title, children }: { title: string; children: ReactNode }) {
  return (
    <div className="rounded-lg border border-rh-border bg-rh-canvas p-3">
      <div className="text-[10px] font-semibold uppercase tracking-widest text-rh-muted">
        {title}
      </div>
      <div className="mt-2 space-y-1.5">{children}</div>
    </div>
  );
}

function TickerRow({
  ticker,
  meta,
  selected,
  onSelect,
}: {
  ticker: string;
  meta: string;
  selected: boolean;
  onSelect: (t: string) => void;
}) {
  return (
    <button
      type="button"
      onClick={() => onSelect(ticker)}
      className={`flex w-full items-center justify-between rounded-md border px-2 py-1.5 text-left text-sm transition ${
        selected
          ? "border-rh-green bg-rh-green/10"
          : "border-rh-border bg-rh-surface hover:border-rh-green"
      }`}
    >
      <span className="font-bold text-rh-green">${ticker}</span>
      <span className="text-[10px] text-rh-muted">{meta}</span>
    </button>
  );
}
