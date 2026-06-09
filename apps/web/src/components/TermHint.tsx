"use client";

import type { ReactNode } from "react";
import { GEIST_TERMS, type GeistTermId } from "@/lib/terms";

export function TermHint({
  term,
  children,
  className = "",
}: {
  term: GeistTermId;
  children?: ReactNode;
  className?: string;
}) {
  const { label, hint } = GEIST_TERMS[term];

  return (
    <span
      className={`group relative inline-flex cursor-help items-center border-b border-dashed border-rh-muted/40 ${className}`}
      tabIndex={0}
      aria-label={`${label}: ${hint}`}
    >
      <span>{children ?? label}</span>
      <span
        role="tooltip"
        className="pointer-events-none absolute bottom-[calc(100%+6px)] left-0 z-50 hidden w-[min(18rem,calc(100vw-2rem))] rounded-lg border border-rh-border bg-rh-surface px-2.5 py-2 font-mono text-[10px] font-normal normal-case leading-relaxed tracking-normal text-rh-ink shadow-rh group-hover:block group-focus-within:block"
      >
        <span className="mb-1 block font-bold uppercase tracking-wider text-rh-green">
          {label}
        </span>
        {hint}
      </span>
    </span>
  );
}
