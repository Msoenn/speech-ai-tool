import { PALETTES, PALETTE_ORDER, type PaletteId } from "../lib/palettes";

interface ThemePickerProps {
  value: string;
  onChange: (id: PaletteId) => void;
}

// Small swatch keys shown as a preview row on each palette card.
const PREVIEW_KEYS = ["bg", "surface", "primary", "accent", "recording"] as const;

export default function ThemePicker({ value, onChange }: ThemePickerProps) {
  return (
    <div className="grid grid-cols-2 gap-3">
      {PALETTE_ORDER.map((id) => {
        const p = PALETTES[id];
        const selected = value === id;
        return (
          <button
            key={id}
            type="button"
            onClick={() => onChange(id)}
            aria-pressed={selected}
            className={`text-left rounded-lg p-3 border transition-colors focus:outline-none focus:ring-1 focus:ring-accent ${
              selected
                ? "border-accent bg-surface-hover"
                : "border-primary/40 bg-bg hover:bg-surface-hover"
            }`}
          >
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-text">{p.name}</span>
              {selected && <span className="text-xs text-accent">Active</span>}
            </div>
            <div className="flex gap-1 mb-2">
              {PREVIEW_KEYS.map((k) => (
                <span
                  key={k}
                  className="h-5 w-5 rounded"
                  style={{ backgroundColor: p.tokens[k], boxShadow: "inset 0 0 0 1px rgba(255,255,255,0.08)" }}
                />
              ))}
            </div>
            <span className="text-xs text-text-muted">{p.blurb}</span>
          </button>
        );
      })}
    </div>
  );
}
