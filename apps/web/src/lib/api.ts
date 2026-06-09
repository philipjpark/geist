import type { AnalyzeResponse, ParseResponse, SemanticIR } from "./ir";
import { fallbackAnalyze, fallbackParse } from "./mock";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";
const TIMEOUT_MS = 20_000;

async function fetchJson<T>(url: string, body: unknown): Promise<T> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), TIMEOUT_MS);
  try {
    const res = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      signal: controller.signal,
    });
    if (!res.ok) throw new Error(`${url} ${res.status}`);
    return await res.json();
  } finally {
    clearTimeout(timer);
  }
}

export async function parseInput(
  text: string,
  mode = "balanced",
  marketType = "cross_asset"
): Promise<ParseResponse> {
  try {
    return await fetchJson<ParseResponse>(`${API_URL}/parse`, {
      text,
      mode,
      market_type: marketType,
    });
  } catch {
    return fallbackParse(text, mode, marketType);
  }
}

export async function analyzeIR(semanticIr: SemanticIR): Promise<AnalyzeResponse> {
  try {
    return await fetchJson<AnalyzeResponse>(`${API_URL}/analyze`, { semantic_ir: semanticIr });
  } catch {
    return fallbackAnalyze(semanticIr);
  }
}

export async function runPipeline(
  text: string,
  mode = "balanced",
  marketType = "cross_asset"
): Promise<{ semantic_ir: SemanticIR; result: AnalyzeResponse }> {
  const { semantic_ir } = await parseInput(text, mode, marketType);
  const result = await analyzeIR(semantic_ir);
  return { semantic_ir, result };
}
