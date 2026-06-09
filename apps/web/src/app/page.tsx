"use client";

import { useState } from "react";
import { ExecutionCoordination } from "@/components/ExecutionCoordination";
import { RiskScores } from "@/components/RiskScores";
import { SelectedAsset } from "@/components/SelectedAsset";
import { SemanticIRPanel } from "@/components/SemanticIRPanel";
import { GeistLogo } from "@/components/GeistLogo";
import { TrendDiscovery } from "@/components/TrendDiscovery";
import { analyzeIR, parseInput } from "@/lib/api";
import { classifyAsset } from "@/lib/assets";
import { discoverSignals } from "@/lib/discovery";
import type { AnalyzeResponse, SemanticIR } from "@/lib/ir";
import { DEFAULT_SOCIAL_SOURCES, type CorpusSourceId } from "@/lib/socials";
import type { DiscoverResponse } from "@/lib/types";
import { PipelineRail } from "@/components/PipelineRail";
import { TermHint } from "@/components/TermHint";

export default function Home() {
  const [corpusSources, setCorpusSources] =
    useState<CorpusSourceId[]>(DEFAULT_SOCIAL_SOURCES);
  const [selectedTicker, setSelectedTicker] = useState<string | null>(null);
  const [discovering, setDiscovering] = useState(false);
  const [discovery, setDiscovery] = useState<DiscoverResponse | null>(null);
  const [semanticIr, setSemanticIr] = useState<SemanticIR | null>(null);
  const [result, setResult] = useState<AnalyzeResponse | null>(null);
  const [pipelineError, setPipelineError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [analyzing, setAnalyzing] = useState(false);

  async function runScan() {
    setDiscovering(true);
    setDiscovery(null);
    setSelectedTicker(null);
    setSemanticIr(null);
    setResult(null);
    setPipelineError(null);
    try {
      const res = await discoverSignals("", corpusSources);
      setDiscovery(res);
    } catch {
      /* discoverSignals falls back silently */
    } finally {
      setDiscovering(false);
    }
  }

  async function selectTicker(ticker: string) {
    const { symbol, marketType } = classifyAsset(ticker);
    setSelectedTicker(symbol);
    setLoading(true);
    setAnalyzing(false);
    setPipelineError(null);
    setSemanticIr(null);
    setResult(null);

    try {
      const { semantic_ir } = await parseInput(symbol, "balanced", marketType);
      setSemanticIr(semantic_ir);
      setAnalyzing(true);
      const analysis = await analyzeIR(semantic_ir);
      setResult(analysis);
    } catch (e) {
      setPipelineError(e instanceof Error ? e.message : "Pipeline failed");
    } finally {
      setLoading(false);
      setAnalyzing(false);
    }
  }

  const activeStep = result?.monad_proof
    ? 3
    : result?.risk_scores
      ? 2
      : semanticIr || loading || analyzing
        ? 1
        : discovery
          ? 0
          : -1;

  return (
    <main className="mx-auto min-h-screen max-w-7xl space-y-4 p-3 md:p-5">
      <header className="border-b border-rh-border pb-5 pt-1">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:gap-6">
          <GeistLogo size={88} priority className="shrink-0 shadow-rh ring-2 ring-rh-border" />
          <div className="min-w-0">
            <span className="font-mono text-sm font-semibold uppercase tracking-[0.35em] text-rh-green md:text-base">
              Geist
            </span>
            <h1 className="mt-2 text-2xl font-black leading-tight text-rh-ink md:text-4xl">
            From Social Signal to Execution Intelligence
            </h1>
            <p className="mt-2 font-mono text-[10px] text-rh-muted">
              <TermHint term="compiler" className="text-rh-muted" /> ·{" "}
              <TermHint term="structure_before_reasoning" className="text-rh-muted" />
            </p>
          </div>
        </div>
      </header>

      <PipelineRail activeStep={activeStep} />

      <TrendDiscovery
        discovery={discovery}
        loading={discovering}
        selectedTicker={selectedTicker}
        corpusSources={corpusSources}
        setCorpusSources={setCorpusSources}
        onScan={runScan}
        onSelectTicker={selectTicker}
      />

      <SelectedAsset ticker={selectedTicker} compiling={loading || analyzing} />

      {pipelineError && (
        <p className="rounded-lg border border-rh-danger/40 bg-rh-danger/10 px-3 py-2 font-mono text-xs text-rh-danger">
          {pipelineError}
        </p>
      )}

      <SemanticIRPanel ir={semanticIr} loading={loading && !semanticIr} analyzing={analyzing} />
      <RiskScores scores={result?.risk_scores ?? null} />
      <ExecutionCoordination result={result} />
    </main>
  );
}
