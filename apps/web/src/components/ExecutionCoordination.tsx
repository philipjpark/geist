"use client";

import type { AnalyzeResponse } from "@/lib/ir";
import { CoordinationRail } from "@/components/CoordinationRail";
import { TermHint } from "@/components/TermHint";

export function ExecutionCoordination({ result }: { result: AnalyzeResponse | null }) {
  const ready = !!result?.risk_scores && !!result?.monad_proof;

  return (
    <section className="space-y-2">
      <div className="flex items-center gap-2 px-1">
        <span className="font-mono text-[10px] font-bold uppercase tracking-[0.2em] text-rh-muted">
          <TermHint term="coordination_rail" />
        </span>
        {ready && (
          <span className="font-mono text-[10px] text-rh-green">rail ready · commit available</span>
        )}
      </div>

      <CoordinationRail
        proof={result?.monad_proof ?? null}
        dag={result?.execution_dag ?? null}
        compositeScore={result?.risk_scores?.composite}
        ready={ready}
      />
    </section>
  );
}
