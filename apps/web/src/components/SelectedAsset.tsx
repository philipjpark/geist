"use client";

export function SelectedAsset({
  ticker,
  compiling,
}: {
  ticker: string | null;
  compiling: boolean;
}) {
  if (!ticker) return null;

  return (
    <div className="flex flex-wrap items-center gap-2 rounded-lg border border-rh-green/30 bg-rh-green/5 px-3 py-2">
      <span className="font-mono text-[10px] uppercase tracking-widest text-rh-muted">
        Selected
      </span>
      <span className="font-mono text-sm font-bold text-rh-green">${ticker}</span>
      {compiling && (
        <span className="font-mono text-[10px] text-rh-muted">compiling…</span>
      )}
    </div>
  );
}
