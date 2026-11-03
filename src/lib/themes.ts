// SPDX-License-Identifier: GPL-3.0-or-later
// Theme definitions for MatrixFileBrowser.
// Each theme is a flat map of CSS custom property -> value.
// To add a new theme, append an entry to THEMES and ship the file.

export interface Theme {
  id: string;
  name: string;
  // Optional preview swatch (hex) shown in the picker.
  swatch: string;
  vars: Record<string, string>;
}

const darkBase = (bg: string, bg1: string, bg2: string, bg3: string, bg4: string, line: string, lineSoft: string) => ({
  'color-scheme': 'dark',
  '--bg': bg,
  '--bg-1': bg1,
  '--bg-2': bg2,
  '--bg-3': bg3,
  '--bg-4': bg4,
  '--line': line,
  '--line-soft': lineSoft,
  '--text': '#e8ecf3',
  '--text-dim': '#9aa4b6',
  '--text-faint': '#6b7487',
  '--blue': '#60a5fa',
  '--danger': '#f87171',
  '--warn': '#fbbf24',
  '--radius': '10px',
  '--radius-sm': '7px',
  '--font': 'ui-sans-serif, system-ui, -apple-system, "Segoe UI", Inter, Cantarell, sans-serif',
  '--mono': 'ui-monospace, "JetBrains Mono", "Cascadia Code", Menlo, Consolas, monospace',
  '--shadow': '0 12px 34px rgba(0, 0, 0, 0.45)',
});

export const THEMES: Theme[] = [
  {
    id: 'dark',
    name: 'Dark (Blue-Grey)',
    swatch: '#5eead4',
    vars: {
      ...darkBase('#0c0f13', '#11151b', '#171c24', '#1e242e', '#262d39', '#252c37', '#1b212a'),
      '--accent': '#5eead4',
      '--accent-strong': '#2dd4bf',
      '--accent-ink': '#04201c',
    },
  },
  {
    id: 'dark-green',
    name: 'Dark (Green)',
    swatch: '#4ade80',
    vars: {
      ...darkBase('#0a120e', '#0f1a14', '#14221a', '#1a2b22', '#213429', '#1e3028', '#16241e'),
      '--accent': '#4ade80',
      '--accent-strong': '#22c55e',
      '--accent-ink': '#052e16',
    },
  },
  {
    id: 'dark-orange',
    name: 'Dark (Orange)',
    swatch: '#fb923c',
    vars: {
      ...darkBase('#130e0a', '#1b140f', '#241a14', '#2e211a', '#392921', '#33231b', '#241a14'),
      '--accent': '#fb923c',
      '--accent-strong': '#f97316',
      '--accent-ink': '#431407',
    },
  },
  {
    id: 'dark-purple',
    name: 'Dark (Purple)',
    swatch: '#c084fc',
    vars: {
      ...darkBase('#0e0a13', '#150f1b', '#1c1424', '#241a2e', '#2d2139', '#281b33', '#1c1424'),
      '--accent': '#c084fc',
      '--accent-strong': '#a855f7',
      '--accent-ink': '#2e1065',
    },
  },
  {
    id: 'dark-rose',
    name: 'Dark (Rose)',
    swatch: '#fb7185',
    vars: {
      ...darkBase('#130a0e', '#1b0f14', '#24141a', '#2e1a21', '#392129', '#331b23', '#24141a'),
      '--accent': '#fb7185',
      '--accent-strong': '#f43f5e',
      '--accent-ink': '#4c0519',
    },
  },
  {
    id: 'dark-amber',
    name: 'Dark (Amber)',
    swatch: '#fbbf24',
    vars: {
      ...darkBase('#13100a', '#1b170f', '#241e14', '#2e261a', '#392f21', '#332a1b', '#241e14'),
      '--accent': '#fbbf24',
      '--accent-strong': '#f59e0b',
      '--accent-ink': '#451a03',
    },
  },
  {
    id: 'dark-cyan',
    name: 'Dark (Cyan)',
    swatch: '#22d3ee',
    vars: {
      ...darkBase('#0a1013', '#0f171b', '#141e24', '#1a262e', '#212f39', '#1b2833', '#141e24'),
      '--accent': '#22d3ee',
      '--accent-strong': '#06b6d4',
      '--accent-ink': '#083344',
    },
  },
];

export const DEFAULT_THEME_ID = 'dark';

export function getTheme(id: string): Theme {
  return THEMES.find((t) => t.id === id) ?? THEMES[0];
}

export function applyTheme(theme: Theme) {
  const root = document.documentElement;
  for (const [key, value] of Object.entries(theme.vars)) {
    if (key === 'color-scheme') {
      root.style.colorScheme = value;
    } else {
      root.style.setProperty(key, value);
    }
  }
}

export function loadSavedThemeId(): string {
  try {
    return localStorage.getItem('matrix-theme') ?? DEFAULT_THEME_ID;
  } catch {
    return DEFAULT_THEME_ID;
  }
}

export function saveThemeId(id: string) {
  try {
    localStorage.setItem('matrix-theme', id);
  } catch {}
}
