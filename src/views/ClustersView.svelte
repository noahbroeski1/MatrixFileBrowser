// SPDX-License-Identifier: GPL-3.0-or-later
<script lang="ts">
  import type { Entry, TreeStats } from '../lib/types';
  import { formatBytes } from '../lib/format';
  import Icon from '../components/Icon.svelte';
  import EntryIcon from '../components/EntryIcon.svelte';

  interface Props {
    root: TreeStats | null;
    loading: boolean;
    onopen: (entry: Entry) => void;
  }

  let { root, loading, onopen }: Props = $props();

  type Mode = 'name' | 'type' | 'size';

  let mode = $state<Mode>('name');
  let expanded = $state<Set<string>>(new Set());

  const STOP = new Set([
    'the', 'a', 'an', 'of', 'in', 'on', 'at', 'to', 'for', 'and', 'or', 'is', 'it',
    'img', 'image', 'photo', 'pic', 'file', 'copy', 'new', 'old', 'final', 'draft',
  ]);

  function tokens(name: string): string[] {
    return name
      .toLowerCase()
      .replace(/\.[^.]+$/, '')
      .split(/[^a-z0-9]+/)
      .filter((t) => t.length >= 3 && !/^\d+$/.test(t) && !STOP.has(t));
  }

  function dominantToken(name: string): string {
    const ts = tokens(name);
    if (ts.length === 0) return '(untitled)';
    // prefer the longest token; ties broken by first occurrence
    return ts.reduce((best, t) => (t.length > best.length ? t : best), ts[0]);
  }

  function sizeBucket(size: number): string {
    if (size < 100_000) return 'small (<100 KB)';
    if (size < 10_000_000) return 'medium (100 KB–10 MB)';
    if (size < 100_000_000) return 'large (10–100 MB)';
    return 'huge (>100 MB)';
  }

  interface Cluster {
    key: string;
    label: string;
    files: Entry[];
    totalBytes: number;
    newestMs: number;
  }

  let clusters = $derived.by((): Cluster[] => {
    if (!root) return [];
    const files: Entry[] = [];
    const walk = (node: TreeStats) => {
      if (node.node.kind === 'file') files.push(node.node);
      for (const c of node.children) walk(c);
    };
    walk(root);

    const groups = new Map<string, Entry[]>();
    for (const f of files) {
      let key: string;
      if (mode === 'type') key = f.ext ?? '(no ext)';
      else if (mode === 'size') key = sizeBucket(f.size);
      else key = dominantToken(f.name);
      if (!groups.has(key)) groups.set(key, []);
      groups.get(key)!.push(f);
    }

    const out: Cluster[] = [];
    for (const [key, files] of groups) {
      out.push({
        key,
        label: key,
        files,
        totalBytes: files.reduce((s, f) => s + f.size, 0),
        newestMs: files.reduce((m, f) => Math.max(m, f.mtimeMs), 0),
      });
    }
    out.sort((a, b) => b.totalBytes - a.totalBytes);
    return out;
  });

  let totalFiles = $derived(clusters.reduce((s, c) => s + c.files.length, 0));

  function toggle(key: string) {
    const next = new Set(expanded);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    expanded = next;
  }

  function hueFor(key: string): number {
    let hash = 0;
    for (let i = 0; i < key.length; i += 1) hash = (hash * 31 + key.charCodeAt(i)) | 0;
    return ((hash % 360) + 360) % 360;
  }
</script>

<div class="clusters">
  <div class="bar">
    <div class="modes" role="group" aria-label="Cluster by">
      <button class:active={mode === 'name'} onclick={() => (mode = 'name')}>Name</button>
      <button class:active={mode === 'type'} onclick={() => (mode = 'type')}>Type</button>
      <button class:active={mode === 'size'} onclick={() => (mode = 'size')}>Size</button>
    </div>
    <span class="summary">
      {#if loading}
        <span class="pulse"><span class="dot"></span>loading…</span>
      {:else}
        {clusters.length} clusters · {totalFiles} files
      {/if}
    </span>
  </div>

  <div class="grid">
    {#if loading}
      <div class="state">
        <span class="spinner"></span>
        <p>Indexing this folder…</p>
      </div>
    {:else if clusters.length === 0}
      <div class="state">
        <Icon name="sparkles" size={26} />
        <p>No files to cluster.</p>
      </div>
    {:else}
      {#each clusters as cluster (cluster.key)}
        <div
          class="card"
          style="--hue:{hueFor(cluster.key)}"
          class:expanded={expanded.has(cluster.key)}
        >
          <button class="card-head" onclick={() => toggle(cluster.key)}>
            <span class="swatch"></span>
            <span class="title truncate">{cluster.label}</span>
            <span class="meta">
              {cluster.files.length} file{cluster.files.length === 1 ? '' : 's'} · {formatBytes(cluster.totalBytes)}
            </span>
            <Icon name={expanded.has(cluster.key) ? 'chevron-right' : 'chevron-right'} size={12} />
          </button>

          {#if expanded.has(cluster.key)}
            <div class="files">
              {#each cluster.files.slice(0, 40) as entry (entry.id)}
                <button class="file" onclick={() => onopen(entry)} title={entry.path}>
                  <EntryIcon {entry} size={13} />
                  <span class="truncate">{entry.name}</span>
                  <span class="fsize">{formatBytes(entry.size)}</span>
                </button>
              {/each}
              {#if cluster.files.length > 40}
                <div class="more">…and {cluster.files.length - 40} more</div>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .clusters {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .bar {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 38px;
    padding: 0 12px;
    flex: none;
    border-bottom: 1px solid var(--line-soft);
    background: var(--bg);
  }

  .modes {
    display: flex;
    gap: 2px;
    padding: 2px;
    background: var(--bg-2);
    border: 1px solid var(--line-soft);
    border-radius: 6px;
  }

  .modes button {
    padding: 3px 10px;
    border-radius: 4px;
    color: var(--text-faint);
    font-size: 10.5px;
    text-transform: capitalize;
  }

  .modes button:hover {
    color: var(--text-dim);
  }

  .modes button.active {
    background: var(--bg-4);
    color: var(--accent);
  }

  .summary {
    margin-left: auto;
    font-size: 11px;
    color: var(--text-faint);
  }

  .pulse {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    color: var(--accent);
  }

  .dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent);
    animation: blink 900ms ease-in-out infinite;
  }

  @keyframes blink {
    50% {
      opacity: 0.25;
    }
  }

  .grid {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 12px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 10px;
    align-content: start;
  }

  .card {
    background: var(--bg-1);
    border: 1px solid var(--line-soft);
    border-radius: var(--radius);
    overflow: hidden;
    transition: border-color 120ms ease;
  }

  .card:hover {
    border-color: var(--bg-4);
  }

  .card.expanded {
    border-color: rgba(94, 234, 212, 0.3);
  }

  .card-head {
    display: grid;
    grid-template-columns: auto 1fr auto auto;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 10px 12px;
    text-align: left;
  }

  .swatch {
    width: 10px;
    height: 10px;
    border-radius: 3px;
    background: hsl(var(--hue) 55% 50%);
    flex: none;
  }

  .title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
  }

  .meta {
    font-size: 10.5px;
    color: var(--text-faint);
    white-space: nowrap;
  }

  .card-head :global(svg) {
    color: var(--text-faint);
    transition: transform 120ms ease;
  }

  .card.expanded .card-head :global(svg) {
    transform: rotate(90deg);
  }

  .files {
    border-top: 1px solid var(--line-soft);
    max-height: 260px;
    overflow-y: auto;
  }

  .file {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 8px;
    width: 100%;
    height: 26px;
    padding: 0 12px;
    text-align: left;
    font-size: 11.5px;
    color: var(--text-dim);
  }

  .file:hover {
    background: var(--bg-2);
    color: var(--text);
  }

  .file :global(svg) {
    color: var(--text-faint);
  }

  .fsize {
    font-size: 10px;
    color: var(--text-faint);
    font-family: var(--mono);
  }

  .more {
    padding: 6px 12px;
    font-size: 10.5px;
    color: var(--text-faint);
    font-style: italic;
  }

  .state {
    grid-column: 1 / -1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 40px;
    color: var(--text-faint);
    text-align: center;
    font-size: 12.5px;
  }

  .state p {
    margin: 0;
  }

  .spinner {
    width: 18px;
    height: 18px;
    border: 2px solid var(--bg-4);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 700ms linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
