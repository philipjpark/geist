"use client";

import { CORPUS_SOURCES, type CorpusSourceId } from "@/lib/socials";

export function SocialSources({
  selected,
  onChange,
}: {
  selected: CorpusSourceId[];
  onChange: (next: CorpusSourceId[]) => void;
}) {
  function toggle(id: CorpusSourceId) {
    if (selected.includes(id)) {
      onChange(selected.filter((s) => s !== id));
    } else {
      onChange([...selected, id]);
    }
  }

  return (
    <div className="flex flex-wrap gap-2">
      {CORPUS_SOURCES.map((s) => {
        const on = selected.includes(s.id);
        return (
          <label
            key={s.id}
            className={`flex cursor-pointer items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs transition ${
              on
                ? "border-rh-green bg-rh-green/10 text-rh-green"
                : "border-rh-border bg-rh-canvas text-rh-muted hover:border-rh-green/50"
            }`}
          >
            <input
              type="checkbox"
              checked={on}
              disabled={false}
              onChange={() => toggle(s.id)}
              className="h-3 w-3 cursor-pointer accent-[var(--rh-green)]"
            />
            {s.label}
          </label>
        );
      })}
    </div>
  );
}
