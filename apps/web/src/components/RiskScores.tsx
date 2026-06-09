import type { RiskScore } from "@/lib/ir";
import { TermHint } from "@/components/TermHint";

type ScoreItem = {
  code: string;
  label: string;
  value: number;
  invert?: boolean;
};

function effectiveRisk(value: number, invert = false) {
  return invert ? 100 - value : value;
}

function scoreStyle(value: number, invert = false) {
  const v = effectiveRisk(value, invert);
  if (v <= 35) {
    return { bg: "var(--score-good)", text: "var(--score-good-text)", signal: "▲" };
  }
  if (v <= 65) {
    return { bg: "var(--score-mid)", text: "var(--score-mid-text)", signal: "■" };
  }
  return { bg: "var(--score-bad)", text: "var(--score-bad-text)", signal: "▼" };
}

function ScoreCell({ code, label, value, invert = false }: ScoreItem) {
  const style = scoreStyle(value, invert);
  const fill = invert ? value : 100 - value;

  return (
    <div className="flex w-[100px] shrink-0 flex-col px-3 py-2.5 sm:w-[108px]" style={{ background: style.bg }}>
      <div className="flex items-center justify-between gap-1">
        <span className="font-mono text-[10px] font-bold tracking-wider text-rh-muted">{code}</span>
        <span className="font-mono text-[10px]" style={{ color: style.text }}>{style.signal}</span>
      </div>
      <div className="mt-1 font-mono text-2xl font-bold tabular-nums leading-none" style={{ color: style.text }}>
        {value.toString().padStart(2, "0")}
      </div>
      <div className="mt-0.5 truncate font-mono text-[9px] uppercase tracking-wide text-rh-muted">{label}</div>
      <div className="mt-2 flex h-1 gap-px overflow-hidden rounded-sm bg-rh-border/40">
        {Array.from({ length: 10 }).map((_, i) => (
          <div
            key={i}
            className="flex-1 rounded-sm"
            style={{
              background: i < Math.round(fill / 10) ? style.text : "transparent",
              opacity: i < Math.round(fill / 10) ? 0.85 : 0.2,
            }}
          />
        ))}
      </div>
    </div>
  );
}

export function RiskScores({ scores }: { scores: RiskScore | null }) {
  if (!scores) {
    return (
      <section className="rounded-lg border border-dashed border-rh-border bg-rh-surface p-4">
        <div className="font-mono text-[10px] font-bold uppercase tracking-widest text-rh-muted">
          <TermHint term="risk_engine" />
        </div>
        <p className="mt-2 font-mono text-xs text-rh-muted">
          Deterministic orchestration scoring — awaiting semantic IR compile…
        </p>
      </section>
    );
  }

  const items: ScoreItem[] = [
    { code: "LIQ", label: "Liquidity", value: scores.liquidity_risk },
    { code: "SPR", label: "Spread", value: scores.spread_risk },
    { code: "VOL", label: "Volatility", value: scores.volatility_risk },
    { code: "CONT", label: "Contention", value: scores.contention_risk },
    { code: "EXEC", label: "Exec Diff", value: scores.execution_difficulty },
    { code: "ROUT", label: "Route Conf", value: scores.route_confidence, invert: true },
    ...(scores.otc_disclosure_risk > 0
      ? [{ code: "DISC", label: "Disclosure", value: scores.otc_disclosure_risk }]
      : []),
  ];

  return (
    <section className="overflow-hidden rounded-lg border border-rh-border bg-rh-surface shadow-rh">
      <div className="flex items-center justify-between border-b border-rh-border bg-rh-surface-2 px-3 py-2">
        <div className="flex items-center gap-2">
          <span className="inline-block h-2 w-2 animate-pulse rounded-full bg-rh-green-bright" />
          <span className="font-mono text-[10px] font-bold uppercase tracking-[0.2em] text-rh-muted">
            <TermHint term="risk_engine" />
          </span>
        </div>
        <div className="flex items-baseline gap-1.5">
          <span className="font-mono text-[10px] uppercase tracking-wider text-rh-accent">Composite</span>
          <span className="font-mono text-lg font-bold tabular-nums text-rh-accent">{scores.composite}</span>
        </div>
      </div>
      <div className="overflow-x-auto">
        <div className="flex min-w-max divide-x divide-rh-border">
          {items.map((item) => (
            <ScoreCell key={item.code} {...item} />
          ))}
        </div>
      </div>
    </section>
  );
}
