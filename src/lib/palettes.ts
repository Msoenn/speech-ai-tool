// Color palettes for the app theme. The default (slate) is also encoded in
// index.css @theme so the very first paint is correct; selecting a palette
// overrides those same CSS variables at runtime via applyPalette().

export type PaletteId = "slate" | "graphite" | "nord" | "ink";

// Token keys mirror the --color-* custom properties defined in index.css.
export interface PaletteTokens {
  bg: string;
  surface: string;
  "surface-hover": string;
  primary: string;
  "primary-hover": string;
  accent: string;
  text: string;
  "text-muted": string;
  success: string;
  warning: string;
  error: string;
  "error-hover": string;
  recording: string;
  "recording-hover": string;
  processing: string;
}

export interface Palette {
  id: PaletteId;
  name: string;
  blurb: string;
  tokens: PaletteTokens;
}

export const PALETTES: Record<PaletteId, Palette> = {
  slate: {
    id: "slate",
    name: "Slate & Teal",
    blurb: "Cool neutral slate, calm teal accent.",
    tokens: {
      bg: "#0e1216", surface: "#161c23", "surface-hover": "#1f2731",
      primary: "#0d9488", "primary-hover": "#14b8a6", accent: "#2dd4bf",
      text: "#e7ecf0", "text-muted": "#8794a1",
      success: "#4ade80", warning: "#fbbf24", error: "#f87171", "error-hover": "#ef4444",
      recording: "#fb7185", "recording-hover": "#f43f5e", processing: "#fbbf24",
    },
  },
  graphite: {
    id: "graphite",
    name: "Graphite & Amber",
    blurb: "Warm near-black, premium amber glow.",
    tokens: {
      bg: "#16130f", surface: "#1f1b16", "surface-hover": "#29231b",
      primary: "#a16207", "primary-hover": "#ca8a04", accent: "#f59e0b",
      text: "#f0e9df", "text-muted": "#a1917d",
      success: "#5cc98d", warning: "#fbbf24", error: "#f2777a", "error-hover": "#e5484d",
      recording: "#fb923c", "recording-hover": "#f97316", processing: "#fbbf24",
    },
  },
  nord: {
    id: "nord",
    name: "Nord Frost",
    blurb: "Muted arctic blue-greys, easy on the eyes.",
    tokens: {
      bg: "#2e3440", surface: "#3b4252", "surface-hover": "#434c5e",
      primary: "#5e81ac", "primary-hover": "#81a1c1", accent: "#88c0d0",
      text: "#eceff4", "text-muted": "#9aa5b8",
      success: "#a3be8c", warning: "#ebcb8b", error: "#bf616a", "error-hover": "#a5545c",
      recording: "#bf616a", "recording-hover": "#a5545c", processing: "#ebcb8b",
    },
  },
  ink: {
    id: "ink",
    name: "Ink & Mauve",
    blurb: "Deep indigo ink, dusty lilac accent.",
    tokens: {
      bg: "#14131c", surface: "#1c1b28", "surface-hover": "#262436",
      primary: "#6c5ce0", "primary-hover": "#8172e8", accent: "#c4a7e7",
      text: "#eae7f2", "text-muted": "#948da8",
      success: "#4ade80", warning: "#fbbf24", error: "#f87171", "error-hover": "#ef4444",
      recording: "#d98bb0", "recording-hover": "#c76fa0", processing: "#fbbf24",
    },
  },
};

export const DEFAULT_PALETTE: PaletteId = "slate";
export const PALETTE_ORDER: PaletteId[] = ["slate", "graphite", "nord", "ink"];

const STORAGE_KEY = "sat-palette";

export function isPaletteId(value: unknown): value is PaletteId {
  return typeof value === "string" && value in PALETTES;
}

/** Override the --color-* CSS variables on :root so every token-based class
 *  re-themes instantly. */
export function applyPalette(id: PaletteId): void {
  const palette = PALETTES[id] ?? PALETTES[DEFAULT_PALETTE];
  const root = document.documentElement;
  for (const [key, value] of Object.entries(palette.tokens)) {
    root.style.setProperty(`--color-${key}`, value);
  }
  try {
    localStorage.setItem(STORAGE_KEY, palette.id);
  } catch {
    // localStorage unavailable — settings remains the source of truth.
  }
}

/** The palette cached from the last session, for a flash-free first paint
 *  before async settings load. Falls back to the default. */
export function cachedPalette(): PaletteId {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (isPaletteId(stored)) return stored;
  } catch {
    // ignore
  }
  return DEFAULT_PALETTE;
}
