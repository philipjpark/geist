import type { SemanticIR } from "@/lib/ir";
import { TermHint } from "@/components/TermHint";

export function SemanticIRPanel({
  ir,
  loading = false,
  analyzing = false,
}: {
  ir: SemanticIR | null;
  loading?: boolean;
  analyzing?: boolean;
}) {
  if (!ir) {
    return (
      <section className="rounded-lg border border-dashed border-rh-border bg-rh-surface p-4">
        <div className="font-mono text-[10px] font-bold uppercase tracking-[0.2em] text-rh-muted">
          Semantic IR
        </div>
        <p className="mt-2 font-mono text-xs text-rh-muted">
          {loading
            ? "Parser: natural language → typed semantic IR…"
            : "Structure before reasoning — awaiting compile…"}
        </p>
      </section>
    );
  }

  return (
    <section className="overflow-hidden rounded-lg border border-rh-border bg-rh-surface">
      <div className="flex items-center justify-between border-b border-rh-border bg-rh-surface-2 px-3 py-2">
        <span className="font-mono text-[10px] font-bold uppercase tracking-[0.2em] text-rh-muted">
          <TermHint term="semantic_ir" />
        </span>
        <span className="font-mono text-[10px] text-rh-accent">
          {analyzing ? "analyzing…" : `conf ${(ir.confidence * 100).toFixed(0)}%`}
        </span>
      </div>
      <pre className="max-h-[280px] overflow-auto p-3 font-mono text-[11px] leading-relaxed text-rh-ink">
        {JSON.stringify(ir, null, 2)}
      </pre>
    </section>
  );
}
