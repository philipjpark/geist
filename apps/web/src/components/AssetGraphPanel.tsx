import type { AssetGraph } from "@/lib/ir";

export function AssetGraphPanel({ graph }: { graph: AssetGraph | null }) {
  if (!graph) {
    return (
      <section className="rounded-lg border border-dashed border-rh-border bg-rh-surface p-4">
        <div className="font-mono text-[10px] font-bold uppercase tracking-[0.2em] text-rh-muted">
          Asset Graph
        </div>
        <p className="mt-2 font-mono text-xs text-rh-muted">—</p>
      </section>
    );
  }

  return (
    <section className="overflow-hidden rounded-lg border border-rh-border bg-rh-surface">
      <div className="flex items-center justify-between border-b border-rh-border bg-rh-surface-2 px-3 py-2">
        <span className="font-mono text-[10px] font-bold uppercase tracking-[0.2em] text-rh-muted">
          Asset Graph
        </span>
        <span className="font-mono text-[10px] text-rh-muted">{graph.source}</span>
      </div>

      <div className="divide-y divide-rh-border">
        {graph.nodes.map((n) => (
          <div key={n.id} className="px-3 py-2 font-mono text-xs">
            <div className="flex items-center gap-2">
              <span className="font-bold text-rh-green-bright">{n.symbol}</span>
              <span className="text-rh-muted">{n.asset_class}</span>
              <span className="ml-auto text-[10px] text-rh-accent">{n.relationship_type}</span>
            </div>
            {n.bid != null && n.ask != null && (
              <div className="mt-1 text-[10px] text-rh-muted">
                {n.bid.toFixed(3)} / {n.ask.toFixed(3)}
                {n.spread_percent != null && ` · ${n.spread_percent.toFixed(1)}%`}
              </div>
            )}
          </div>
        ))}
      </div>

      {graph.edges.length > 0 && (
        <div className="border-t border-rh-border bg-rh-canvas px-3 py-2">
          <div className="font-mono text-[9px] uppercase tracking-wider text-rh-muted">Edges</div>
          {graph.edges.map((e) => (
            <div key={e.id} className="mt-1 font-mono text-[10px] text-rh-muted">
              {e.source} → {e.target} · {e.relationship_type}
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
