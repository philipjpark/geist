"use client";

import { GEIST_TERMS, PIPELINE_TERM_IDS } from "@/lib/terms";

type StepStatus = "pending" | "active" | "done";

function stepStatus(index: number, activeStep: number): StepStatus {
  if (activeStep < 0) return "pending";
  if (index < activeStep) return "done";
  if (index === activeStep) return "active";
  return "pending";
}

export function PipelineRail({ activeStep }: { activeStep: number }) {
  return (
    <section
      className="rounded-lg border border-rh-border bg-rh-surface p-4"
      aria-label="Compilation pipeline"
    >
      <ol className="grid gap-3 sm:grid-cols-2 lg:grid-cols-5 lg:gap-2">
        {PIPELINE_TERM_IDS.map((termId, i) => {
          const { label, hint } = GEIST_TERMS[termId];
          const status = stepStatus(i, activeStep);
          const isLast = i === PIPELINE_TERM_IDS.length - 1;

          return (
            <li key={termId} className="relative flex min-w-0 flex-col">
              {!isLast && (
                <span
                  className={`absolute left-[calc(50%+1.25rem)] top-5 hidden h-px w-[calc(100%-2.5rem)] lg:block ${
                    status === "done" ? "bg-rh-green" : "bg-rh-border"
                  }`}
                  aria-hidden
                />
              )}

              <div className="flex items-start gap-2.5">
                <span
                  className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-full border-2 font-mono text-sm font-bold transition ${
                    status === "done"
                      ? "border-rh-green bg-rh-green text-rh-on-green"
                      : status === "active"
                        ? "animate-pulse-green border-rh-green bg-rh-green/15 text-rh-green"
                        : "border-rh-border bg-rh-canvas text-rh-muted"
                  }`}
                >
                  {status === "done" ? "✓" : i + 1}
                </span>

                <div className="min-w-0 flex-1 pt-0.5">
                  <div
                    className={`font-mono text-[11px] font-bold uppercase tracking-wider ${
                      status === "pending" ? "text-rh-muted" : "text-rh-green"
                    }`}
                  >
                    {label}
                  </div>
                  <p
                    className={`mt-1 font-mono text-[10px] leading-relaxed ${
                      status === "pending"
                        ? "text-rh-muted/70"
                        : status === "active"
                          ? "text-rh-ink"
                          : "text-rh-muted"
                    }`}
                  >
                    {hint}
                  </p>
                  {status === "active" && (
                    <span className="mt-1.5 inline-block font-mono text-[9px] font-bold uppercase tracking-widest text-rh-accent">
                      In progress
                    </span>
                  )}
                </div>
              </div>
            </li>
          );
        })}
      </ol>
    </section>
  );
}
