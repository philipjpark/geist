export const GEIST_TERMS = {
  compiler: {
    label: "Semantic execution compiler",
    hint: "Compiles human intent into typed execution intelligence — not a chatbot, broker, or trading app.",
  },
  discovery: {
    label: "Discovery",
    hint: "Ingests fragmented feeds to surface candidate signals before semantic compilation.",
  },
  semantic_ir: {
    label: "Semantic IR",
    hint: "Typed structured meaning — raw language stops here and everything downstream consumes this.",
  },
  risk_engine: {
    label: "Risk Engine",
    hint: "Deterministic orchestration scoring of liquidity, spread, contention, and route confidence.",
  },
  execution_dag: {
    label: "Execution Graph",
    hint: "Compiled dependency plan with parallel paths and contention points — intelligence, not trade execution.",
  },
  coordination_rail: {
    label: "Coordination Rail",
    hint: "Verification layer that timestamps semantic and graph hashes on Monad's coordination substrate.",
  },
  otc: {
    label: "OTC",
    hint: "Fragmented off-exchange venue routed through disclosure and liquidity scoring.",
  },
  stock: {
    label: "Stock",
    hint: "Listed equity venue routed through cross-asset execution graph generation.",
  },
  structure_before_reasoning: {
    label: "Structure before reasoning",
    hint: "Parse and structure first, then reason — externalize cognition before probabilistic steps.",
  },
} as const;

export type GeistTermId = keyof typeof GEIST_TERMS;

export const PIPELINE_TERM_IDS: GeistTermId[] = [
  "discovery",
  "semantic_ir",
  "risk_engine",
  "coordination_rail",
];
