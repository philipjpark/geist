"use client";

const EXAMPLES = ["AI compute demand surge", "CYDY"];

export function SignalInput({
  query,
  setQuery,
  onCompile,
  loading,
  analyzing = false,
}: {
  query: string;
  setQuery: (v: string) => void;
  onCompile: () => void;
  loading: boolean;
  analyzing?: boolean;
}) {
  return (
    <section className="rounded-lg border border-rh-border bg-rh-surface p-4">
      <div className="mb-3 font-mono text-[10px] font-bold uppercase tracking-[0.2em] text-rh-muted">
        Raw Input
      </div>

      <div className="mb-3 flex flex-wrap gap-1.5">
        {EXAMPLES.map((ex) => (
          <button
            key={ex}
            type="button"
            onClick={() => setQuery(ex)}
            className="rounded-full border border-rh-border bg-rh-surface-2 px-2.5 py-0.5 font-mono text-[11px] text-rh-muted transition hover:border-rh-green hover:text-rh-green"
          >
            {ex}
          </button>
        ))}
      </div>

      <textarea
        className="h-16 w-full resize-none rounded-lg border border-rh-border bg-rh-canvas p-3 font-mono text-sm text-rh-ink outline-none focus:border-rh-green"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="Natural language market signal"
      />

      <div className="mt-3 flex justify-end">
        <button
          type="button"
          onClick={onCompile}
          disabled={loading || !query.trim()}
          className="rounded-full bg-rh-green px-5 py-2 font-mono text-sm font-bold text-rh-on-green transition hover:bg-rh-green-hover disabled:opacity-50"
        >
          {analyzing ? "Risk + DAG…" : loading ? "Parsing IR…" : "Compile → Analyze"}
        </button>
      </div>
    </section>
  );
}
