// SPDX-License-Identifier: GPL-3.0-or-later
// Squarified treemap layout (Bruls, Huizing & van Wijk) plus palette helpers.

import type { Entry } from './types';

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

/**
 * Lay out `values` (positive weights) inside the rectangle (x, y, w, h).
 * Returns one rect per value, in the same order.
 */
export function squarify(values: number[], x: number, y: number, w: number, h: number): Rect[] {
  const total = values.reduce((sum, value) => sum + value, 0);
  if (total <= 0 || w <= 0 || h <= 0) return [];

  const scale = (w * h) / total;
  const scaled = values.map((value) => Math.max(value * scale, 1e-6));
  const out: Rect[] = [];

  let index = 0;
  let rx = x;
  let ry = y;
  let rw = w;
  let rh = h;

  while (index < scaled.length) {
    const side = Math.min(rw, rh);
    if (side <= 0) break;

    let row: number[] = [];
    let sum = 0;
    let best = Infinity;
    let cursor = index;

    while (cursor < scaled.length) {
      const candidate = scaled[cursor];
      const nextSum = sum + candidate;
      const nextRow = [...row, candidate];
      const ratio = worstRatio(nextRow, nextSum, side);
      if (row.length > 0 && ratio > best) break;
      row = nextRow;
      sum = nextSum;
      best = ratio;
      cursor += 1;
    }

    if (rw >= rh) {
      const columnWidth = sum / rh;
      let offset = ry;
      for (const value of row) {
        const height = (value / sum) * rh;
        out.push({ x: rx, y: offset, w: columnWidth, h: height });
        offset += height;
      }
      rx += columnWidth;
      rw -= columnWidth;
    } else {
      const rowHeight = sum / rw;
      let offset = rx;
      for (const value of row) {
        const width = (value / sum) * rw;
        out.push({ x: offset, y: ry, w: width, h: rowHeight });
        offset += width;
      }
      ry += rowHeight;
      rh -= rowHeight;
    }

    index = cursor;
  }

  return out;
}

function worstRatio(row: number[], sum: number, side: number): number {
  let max = 0;
  let min = Infinity;
  for (const value of row) {
    if (value > max) max = value;
    if (value < min) min = value;
  }
  const sumSquared = sum * sum;
  const sideSquared = side * side;
  return Math.max((sideSquared * max) / sumSquared, sumSquared / (sideSquared * min));
}

export type ColorMode = 'type' | 'age' | 'size';

function hashString(value: string): number {
  let hash = 2166136261;
  for (let i = 0; i < value.length; i += 1) {
    hash = Math.imul(hash ^ value.charCodeAt(i), 16777619);
  }
  return hash >>> 0;
}

const KNOWN_HUES: Record<string, number> = {
  dir: 168,
  rs: 24,
  ts: 210,
  js: 48,
  json: 48,
  md: 200,
  txt: 200,
  pdf: 0,
  jpg: 280,
  jpeg: 280,
  png: 280,
  gif: 300,
  webp: 300,
  mp4: 330,
  mkv: 330,
  mov: 330,
  mp3: 120,
  flac: 120,
  zip: 38,
  rpm: 38,
  iso: 14,
  toml: 20,
  yaml: 20,
  css: 190,
  html: 18,
};

export function colorFor(entry: Entry, mode: ColorMode, ageMs = entry.mtimeMs): string {
  if (mode === 'age') {
    const days = Math.min(1095, Math.max(0, (Date.now() - ageMs) / 86_400_000));
    const t = days / 1095;
    const hue = 162 + t * 116;
    return `hsl(${hue} 52% ${40 - t * 8}%)`;
  }

  if (mode === 'size') {
    const t = Math.min(1, Math.log10(Math.max(1, entry.size)) / 10);
    const hue = 168 + t * 42;
    return `hsl(${hue} 58% ${34 + t * 18}%)`;
  }

  const key = entry.kind === 'dir' ? `dir:${entry.name}` : entry.ext ?? 'file';
  if (entry.kind === 'dir') {
    // Folders stay in a cohesive teal → indigo band so the map reads as one
    // surface while neighbouring folders remain distinguishable.
    const hue = 150 + (hashString(key) % 110);
    return `hsl(${hue} 46% 38%)`;
  }
  const hue = KNOWN_HUES[key] ?? hashString(key) % 360;
  return `hsl(${hue} 48% 40%)`;
}

export function withAlpha(color: string, alpha: number): string {
  // Colors here are always hsl(...) triples.
  return color.replace('hsl(', 'hsla(').replace(')', ` / ${alpha})`);
}
