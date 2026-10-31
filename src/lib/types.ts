// SPDX-License-Identifier: GPL-3.0-or-later
// Shared types between the Rust shell and the views.

export type EntryKind = 'dir' | 'file' | 'symlink' | 'other';

export interface Entry {
  id: string;
  path: string;
  name: string;
  kind: EntryKind;
  size: number;
  mtimeMs: number;
  ctimeMs: number;
  isHidden: boolean;
  mode: number;
  ext: string | null;
  mime: string | null;
}

export interface DirPage {
  path: string;
  entries: Entry[];
  nextOffset: number | null;
  total: number;
}

export interface TreeStats {
  node: Entry;
  children: TreeStats[];
  totalBytes: number;
  fileCount: number;
  newestMs: number;
  truncated: boolean;
}

export type ThumbKind = 'image' | 'video' | 'pdf';

export interface ThumbRef {
  path: string;
  thumbPath: string;
  width: number;
  height: number;
  cached: boolean;
  kind: ThumbKind;
}

export type PlaceKind = 'home' | 'xdg' | 'mount' | 'trash' | 'root';

export interface Place {
  id: string;
  label: string;
  path: string;
  kind: PlaceKind;
}

export type SortKey = 'name' | 'size' | 'modified' | 'type';
export type ViewId = 'list' | 'treemap' | 'mosaic' | 'sunburst' | 'query';

export interface ListRequest {
  path: string;
  offset: number;
  limit: number;
  sort: SortKey;
  descending: boolean;
  dirsFirst: boolean;
  showHidden: boolean;
  filter?: string;
}

export interface Backend {
  listDir(req: ListRequest): Promise<DirPage>;
  treeStats(path: string, depth: number): Promise<TreeStats>;
  thumbnails(paths: string[], px: number): Promise<ThumbRef[]>;
  places(): Promise<Place[]>;
  homeDir(): Promise<string>;
  reveal(path: string): Promise<void>;
  openPath(path: string): Promise<void>;
  previewText(path: string, maxBytes: number): Promise<string>;
  watchStart(paths: string[]): Promise<void>;
  onChanged(cb: (dirs: string[]) => void): Promise<() => void>;
}
