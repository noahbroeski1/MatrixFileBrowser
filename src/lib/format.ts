// SPDX-License-Identifier: GPL-3.0-or-later
// Formatting helpers shared across views.

import type { Entry } from './types';

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return '—';
  if (bytes < 1024) return `${bytes} B`;
  const units = ['KB', 'MB', 'GB', 'TB', 'PB'];
  let value = bytes / 1024;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value < 10 ? value.toFixed(1) : Math.round(value)} ${units[unit]}`;
}

export function formatDate(ms: number): string {
  if (!ms) return '—';
  const date = new Date(ms);
  const now = Date.now();
  const diff = now - ms;
  if (diff < 60_000) return 'just now';
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)} min ago`;
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)} h ago`;
  if (diff < 7 * 86_400_000) return `${Math.floor(diff / 86_400_000)} d ago`;
  return date.toLocaleDateString(undefined, {
    year: date.getFullYear() === new Date().getFullYear() ? undefined : 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

export function formatMode(mode: number): string {
  const bits = ['r', 'w', 'x'];
  let out = '';
  for (let shift = 6; shift >= 0; shift -= 3) {
    for (let i = 0; i < 3; i += 1) {
      out += mode & (1 << (shift + (2 - i))) ? bits[i] : '-';
    }
  }
  return out;
}

export function parentOf(path: string): string {
  if (path === '/' || path === '') return '/';
  const trimmed = path.replace(/\/+$/, '');
  const idx = trimmed.lastIndexOf('/');
  if (idx <= 0) return '/';
  return trimmed.slice(0, idx);
}

export function joinPath(base: string, name: string): string {
  return base === '/' ? `/${name}` : `${base}/${name}`;
}

export function basename(path: string): string {
  const trimmed = path.replace(/\/+$/, '');
  const idx = trimmed.lastIndexOf('/');
  return idx === -1 ? trimmed : trimmed.slice(idx + 1) || '/';
}

export function isImage(ext: string | null): boolean {
  if (!ext) return false;
  return ['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'tif', 'tiff', 'avif', 'heic', 'heif', 'ico', 'svg'].includes(ext);
}

export function isVideo(ext: string | null): boolean {
  if (!ext) return false;
  return ['mp4', 'm4v', 'mkv', 'webm', 'mov', 'avi', 'wmv', 'flv', 'mpg', 'mpeg', '3gp', 'ts'].includes(ext);
}

export function isText(ext: string | null): boolean {
  if (!ext) return false;
  return [
    'txt', 'md', 'markdown', 'json', 'toml', 'yaml', 'yml', 'rs', 'ts', 'js', 'tsx', 'jsx',
    'py', 'c', 'h', 'cpp', 'hpp', 'go', 'java', 'sh', 'bash', 'zsh', 'fish', 'css', 'html',
    'xml', 'csv', 'log', 'ini', 'conf', 'env', 'sql', 'lua', 'rb', 'php', 'kt', 'swift',
  ].includes(ext);
}

/** Icon name for an entry, matching the set in Icon.svelte. */
export function iconFor(entry: Entry): string {
  if (entry.kind === 'dir') return 'folder';
  const ext = entry.ext ?? '';
  if (isImage(ext)) return 'image';
  if (isVideo(ext)) return 'video';
  if (['mp3', 'flac', 'wav', 'ogg', 'm4a', 'opus'].includes(ext)) return 'music';
  if (['zip', 'tar', 'gz', 'xz', 'zst', '7z', 'rar', 'rpm', 'deb', 'iso'].includes(ext)) return 'archive';
  if (ext === 'pdf') return 'pdf';
  if (isText(ext)) return 'code';
  return 'file';
}
