// SPDX-License-Identifier: GPL-3.0-or-later
// A deterministic in-browser mock of the Rust backend. This lets the whole UI
// (treemap, mosaic, preview, navigation) run under plain `npm run dev` without
// the Tauri shell, which is invaluable for design work and CI screenshots.

import type {
  Backend,
  DirPage,
  Entry,
  ListRequest,
  Place,
  ThumbKind,
  ThumbRef,
  TreeStats,
} from './types';
import { isImage, isVideo } from './format';

const HOME = '/home/demo';
const NOW = Date.now();
const DAY = 86_400_000;

interface Spec {
  name: string;
  kind: 'dir' | 'file';
  size?: [number, number];
  age?: number;
  children?: Spec[];
}

interface Node {
  name: string;
  path: string;
  kind: 'dir' | 'file';
  size: number;
  mtime: number;
  children?: Node[];
}

const f = (name: string, size: [number, number], age = 900): Spec => ({
  name,
  kind: 'file',
  size,
  age,
});

const d = (name: string, children: Spec[]): Spec => ({ name, kind: 'dir', children });

function fnv1a(value: string): number {
  let hash = 2166136261;
  for (let i = 0; i < value.length; i += 1) {
    hash = Math.imul(hash ^ value.charCodeAt(i), 16777619);
  }
  return hash >>> 0;
}

function rng(seed: number): () => number {
  let state = seed || 1;
  return () => {
    state = (state + 0x6d2b79f5) | 0;
    let t = Math.imul(state ^ (state >>> 15), 1 | state);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function materialize(spec: Spec, parent: string): Node {
  const path = parent === '/' ? `/${spec.name}` : `${parent}/${spec.name}`;
  const random = rng(fnv1a(path));
  const mtime = NOW - Math.floor(random() * (spec.age ?? 900)) * DAY - Math.floor(random() * DAY);

  if (spec.kind === 'dir') {
    return {
      name: spec.name,
      path,
      kind: 'dir',
      size: 0,
      mtime,
      children: (spec.children ?? []).map((child) => materialize(child, path)),
    };
  }

  const [min, max] = spec.size ?? [1_024, 1_000_000];
  return {
    name: spec.name,
    path,
    kind: 'file',
    size: Math.floor(min + random() * (max - min)),
    mtime,
  };
}

function photos(prefix: string, count: number, from: number): Spec[] {
  return Array.from({ length: count }, (_, i) => {
    const index = String(from + i).padStart(4, '0');
    return f(`${prefix}_${index}.jpg`, [1_200_000, 9_500_000], 700);
  });
}

const spec = d('demo', [
  d('Documents', [
    d('Invoices', Array.from({ length: 16 }, (_, i) => f(`invoice-2025-${String(i + 1).padStart(3, '0')}.pdf`, [90_000, 850_000], 400))),
    d('Notes', Array.from({ length: 28 }, (_, i) => f(`note-${String(i + 1).padStart(2, '0')}.md`, [1_200, 18_000], 500))),
    d('Reports', Array.from({ length: 11 }, (_, i) => f(`report-q${(i % 4) + 1}-${2023 + (i % 3)}.pdf`, [400_000, 4_200_000], 700))),
    f('resume.pdf', [180_000, 240_000], 120),
    f('budget-2026.ods', [40_000, 90_000], 60),
    f('notes.txt', [2_000, 9_000], 20),
  ]),
  d('Downloads', [
    ...Array.from({ length: 8 }, (_, i) => f(`archive-${i + 1}.zip`, [8_000_000, 480_000_000], 300)),
    ...Array.from({ length: 5 }, (_, i) => f(`package-${i + 1}.rpm`, [2_000_000, 90_000_000], 200)),
    f('fedora-workstation.iso', [1_800_000_000, 2_400_000_000], 90),
    f('ubuntu-server.iso', [1_500_000_000, 2_100_000_000], 150),
    f('screenshot-2026-10-30.png', [400_000, 2_600_000], 1),
  ]),
  d('Pictures', [
    d('2024', photos('IMG', 48, 1200)),
    d('2025', photos('IMG', 56, 3400)),
    d('Wallpapers', photos('wall', 24, 1)),
    d('Screenshots', photos('shot', 18, 900)),
    ...photos('loose', 14, 7000),
  ]),
  d('Music', [
    d('Albums', [
      d('Neon Drift', Array.from({ length: 11 }, (_, i) => f(`${String(i + 1).padStart(2, '0')} - track.flac`, [18_000_000, 60_000_000], 600))),
      d('Glass Harbor', Array.from({ length: 9 }, (_, i) => f(`${String(i + 1).padStart(2, '0')} - track.mp3`, [4_000_000, 12_000_000], 800))),
    ]),
    ...Array.from({ length: 22 }, (_, i) => f(`single-${String(i + 1).padStart(2, '0')}.mp3`, [3_500_000, 11_000_000], 700)),
  ]),
  d('Videos', [
    ...Array.from({ length: 9 }, (_, i) => f(`clip-${String(i + 1).padStart(2, '0')}.mp4`, [80_000_000, 1_400_000_000], 500)),
    ...Array.from({ length: 5 }, (_, i) => f(`recording-${i + 1}.mkv`, [200_000_000, 3_200_000_000], 350)),
  ]),
  d('Projects', [
    d('matrix-file-browser', [
      d('crates', [
        d('matrix-core', [f('lib.rs', [2_000, 9_000], 1), f('entry.rs', [3_000, 12_000], 1), f('Cargo.toml', [400, 1_200], 1)]),
        d('matrix-thumbs', [f('lib.rs', [4_000, 16_000], 1), f('Cargo.toml', [400, 1_200], 1)]),
        d('matrix-aggregate', [f('lib.rs', [5_000, 18_000], 1), f('Cargo.toml', [400, 1_200], 1)]),
      ]),
      d('ui', [d('src', [f('App.svelte', [8_000, 30_000], 1), f('TreemapView.svelte', [6_000, 24_000], 1), f('main.ts', [300, 900], 1)])]),
      f('README.md', [1_500, 5_000], 1),
      f('LICENSE', [30_000, 40_000], 1),
      f('Cargo.toml', [600, 1_800], 1),
    ]),
    d('orb', [d('src', [f('main.rs', [3_000, 14_000], 40), f('render.rs', [6_000, 22_000], 40)]), f('README.md', [900, 3_000], 40)]),
    d('website', [d('src', [f('index.html', [2_000, 8_000], 120), f('styles.css', [3_000, 12_000], 120)]), f('package.json', [300, 900], 120)]),
    f('scratch.md', [200, 2_000], 3),
  ]),
  d('.config', [
    d('matrix', [f('settings.json', [200, 2_000], 5)]),
    f('kdeglobals', [2_000, 8_000], 30),
    f('gitconfig', [300, 1_200], 300),
  ]),
  f('README.md', [1_000, 4_000], 10),
  f('todo.txt', [200, 1_500], 1),
  f('notes.md', [3_000, 14_000], 4),
  f('screenshot.png', [900_000, 3_400_000], 2),
  f('.bashrc', [1_000, 4_000], 400),
  f('.profile', [300, 900], 400),
]);

const root = materialize(d('home', [spec]), '/');
const index = new Map<string, Node>();

function indexTree(node: Node): void {
  index.set(node.path, node);
  node.children?.forEach(indexTree);
}
indexTree(root);

function toEntry(node: Node): Entry {
  const ext = node.kind === 'file' ? node.name.split('.').pop()?.toLowerCase() ?? null : null;
  const extOrNull = ext && ext !== node.name.toLowerCase() ? ext : null;
  return {
    id: (fnv1a(node.path).toString(16) + fnv1a(`salt:${node.path}`).toString(16)).padStart(16, '0').slice(0, 16),
    path: node.path,
    name: node.name,
    kind: node.kind,
    size: node.size,
    mtimeMs: node.mtime,
    ctimeMs: node.mtime,
    isHidden: node.name.startsWith('.'),
    mode: node.kind === 'dir' ? 0o40755 : 0o100644,
    ext: extOrNull,
    mime: node.kind === 'dir' ? null : guessMime(extOrNull),
  };
}

function guessMime(ext: string | null): string | null {
  if (!ext) return null;
  const map: Record<string, string> = {
    jpg: 'image/jpeg',
    jpeg: 'image/jpeg',
    png: 'image/png',
    gif: 'image/gif',
    webp: 'image/webp',
    svg: 'image/svg+xml',
    mp4: 'video/mp4',
    mkv: 'video/x-matroska',
    mp3: 'audio/mpeg',
    flac: 'audio/flac',
    pdf: 'application/pdf',
    zip: 'application/zip',
    md: 'text/markdown',
    txt: 'text/plain',
    rs: 'text/x-rust',
    toml: 'text/x-toml',
    json: 'application/json',
  };
  return map[ext] ?? 'application/octet-stream';
}

function sortEntries(entries: Entry[], req: ListRequest): Entry[] {
  const sorted = [...entries];
  sorted.sort((a, b) => {
    if (req.dirsFirst) {
      const aDir = a.kind === 'dir' ? 1 : 0;
      const bDir = b.kind === 'dir' ? 1 : 0;
      if (aDir !== bDir) return bDir - aDir;
    }
    let cmp = 0;
    switch (req.sort) {
      case 'size':
        cmp = a.size - b.size;
        break;
      case 'modified':
        cmp = a.mtimeMs - b.mtimeMs;
        break;
      case 'type':
        cmp = (a.ext ?? '').localeCompare(b.ext ?? '') || a.name.localeCompare(b.name);
        break;
      default:
        cmp = a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' });
    }
    return req.descending ? -cmp : cmp;
  });
  return sorted;
}

function buildStats(node: Node, depth: number): TreeStats {
  const entry = toEntry(node);
  if (node.kind === 'file') {
    return { node: entry, children: [], totalBytes: node.size, fileCount: 1, newestMs: node.mtime, truncated: false };
  }

  const children = (node.children ?? []).map((child) => buildStats(child, depth - 1));
  let totalBytes = 0;
  let fileCount = 0;
  let newestMs = node.mtime;
  for (const child of children) {
    totalBytes += child.totalBytes;
    fileCount += child.fileCount;
    newestMs = Math.max(newestMs, child.newestMs);
  }

  return {
    node: entry,
    children: depth > 0 ? children : [],
    totalBytes,
    fileCount,
    newestMs,
    truncated: depth <= 0 && children.length > 0,
  };
}

function thumbKind(ext: string | null): ThumbKind | null {
  if (isImage(ext)) return 'image';
  if (isVideo(ext)) return 'video';
  if (ext === 'pdf') return 'pdf';
  return null;
}

function placeholderSvg(path: string): string {
  const seed = fnv1a(path);
  const random = rng(seed);
  const hue = seed % 360;
  const hue2 = (hue + 35 + (seed % 90)) % 360;
  const blobs = Array.from({ length: 4 }, () => {
    const cx = Math.floor(random() * 400);
    const cy = Math.floor(random() * 300);
    const r = 30 + Math.floor(random() * 130);
    return `<circle cx="${cx}" cy="${cy}" r="${r}" fill="hsl(${(hue2 + random() * 80) % 360} 70% 60%)" opacity="${(0.08 + random() * 0.22).toFixed(2)}"/>`;
  }).join('');
  // No text in the placeholder: the tile caption already names the file.
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300" viewBox="0 0 400 300">
    <defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="hsl(${hue} 45% 16%)"/>
      <stop offset="1" stop-color="hsl(${hue2} 55% 38%)"/>
    </linearGradient></defs>
    <rect width="400" height="300" fill="url(#g)"/>
    ${blobs}
  </svg>`;
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
}

function delay(ms: number): Promise<void> {
  const instant =
    typeof location !== 'undefined' && new URLSearchParams(location.search).has('instant');
  if (instant) return Promise.resolve();
  return new Promise((resolve) => setTimeout(resolve, ms));
}

const SAMPLE_TEXT = `MatrixFileBrowser notes
========================

This is a mock preview. Under the Tauri shell the same pane streams the real
first kilobytes of the file from Rust.

- Treemap: area-weighted rectangles by size
- Mosaic: adaptive thumbnail wall
- Sunburst, clusters and the spatial palace are on the roadmap

Everything here is deterministic, so screenshots stay stable between runs.
`;

export function createMockBackend(): Backend {
  return {
    async listDir(req: ListRequest): Promise<DirPage> {
      await delay(45);
      const node = index.get(req.path);
      if (!node || node.kind !== 'dir') {
        throw new Error(`Cannot open ${req.path}`);
      }

      let entries = (node.children ?? []).map(toEntry);
      if (!req.showHidden) entries = entries.filter((entry) => !entry.isHidden);
      if (req.filter) {
        const needle = req.filter.toLowerCase();
        entries = entries.filter((entry) => entry.name.toLowerCase().includes(needle));
      }
      entries = sortEntries(entries, req);

      const total = entries.length;
      const slice = entries.slice(req.offset, req.offset + req.limit);
      const nextOffset = req.offset + slice.length < total ? req.offset + slice.length : null;
      return { path: req.path, entries: slice, nextOffset, total };
    },

    async treeStats(path: string, depth: number): Promise<TreeStats> {
      await delay(320);
      const node = index.get(path);
      if (!node) throw new Error(`Cannot aggregate ${path}`);
      return buildStats(node, Math.max(0, Math.min(depth, 8)));
    },

    async thumbnails(paths: string[], _px: number): Promise<ThumbRef[]> {
      await delay(25);
      const out: ThumbRef[] = [];
      for (const path of paths) {
        const node = index.get(path);
        if (!node || node.kind !== 'file') continue;
        const ext = node.name.split('.').pop()?.toLowerCase() ?? null;
        const kind = thumbKind(ext);
        if (!kind) continue;
        out.push({
          path,
          thumbPath: placeholderSvg(path),
          width: 400,
          height: 300,
          cached: true,
          kind,
        });
      }
      return out;
    },

    async places(): Promise<Place[]> {
      const mk = (label: string, path: string, kind: Place['kind']): Place => ({
        id: path,
        label,
        path,
        kind,
      });
      return [
        mk('Home', HOME, 'home'),
        mk('Documents', `${HOME}/Documents`, 'xdg'),
        mk('Downloads', `${HOME}/Downloads`, 'xdg'),
        mk('Pictures', `${HOME}/Pictures`, 'xdg'),
        mk('Music', `${HOME}/Music`, 'xdg'),
        mk('Videos', `${HOME}/Videos`, 'xdg'),
        mk('Projects', `${HOME}/Projects`, 'xdg'),
        mk('Trash', `${HOME}/.local/share/Trash/files`, 'trash'),
        mk('Filesystem', '/', 'root'),
      ];
    },

    async homeDir(): Promise<string> {
      return HOME;
    },

    async reveal(path: string): Promise<void> {
      console.info('[mock] reveal', path);
    },

    async openPath(path: string): Promise<void> {
      console.info('[mock] open', path);
    },

    async previewText(_path: string, _maxBytes: number): Promise<string> {
      await delay(20);
      return SAMPLE_TEXT;
    },

    async watchStart(_paths: string[]): Promise<void> {
      // No-op in the browser.
    },

    async onChanged(): Promise<() => void> {
      return () => {};
    },
  };
}
