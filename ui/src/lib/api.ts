// SPDX-License-Identifier: GPL-3.0-or-later
// The single seam between the Svelte UI and the Rust shell. Everything above
// this file talks to the `Backend` interface, so the browser mock and the real
// Tauri commands are interchangeable.

import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type {
  Backend,
  DirPage,
  ListRequest,
  Place,
  ThumbRef,
  TreeStats,
} from './types';
import { createMockBackend } from './mock';

export const isTauri =
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in (window as unknown as Record<string, unknown>);

const tauriBackend: Backend = {
  listDir: (req: ListRequest) =>
    invoke<DirPage>('list_dir', {
      path: req.path,
      offset: req.offset,
      limit: req.limit,
      sort: req.sort,
      descending: req.descending,
      dirsFirst: req.dirsFirst,
      showHidden: req.showHidden,
      filter: req.filter ?? null,
    }),
  treeStats: (path: string, depth: number) => invoke<TreeStats>('tree_stats', { path, depth }),
  thumbnails: (paths: string[], px: number) => invoke<ThumbRef[]>('thumbnails', { paths, px }),
  places: () => invoke<Place[]>('places'),
  homeDir: () => invoke<string>('home_dir'),
  reveal: (path: string) => invoke<void>('reveal', { path }),
  openPath: (path: string) => invoke<void>('open_path', { path }),
  previewText: (path: string, maxBytes: number) =>
    invoke<string>('preview_text', { path, maxBytes }),
  watchStart: (paths: string[]) => invoke<void>('watch_start', { paths }),
  onChanged: async (cb: (dirs: string[]) => void) => {
    const unlisten = await listen<string[]>('fs://changed', (event) => cb(event.payload));
    return unlisten;
  },
};

export const backend: Backend = isTauri ? tauriBackend : createMockBackend();

/** Turn a cached thumbnail path into something the webview can render. */
export function assetUrl(path: string): string {
  if (!path) return '';
  if (path.startsWith('data:') || path.startsWith('http') || path.startsWith('blob:')) {
    return path;
  }
  return convertFileSrc(path);
}
