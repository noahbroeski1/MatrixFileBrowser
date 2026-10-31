// SPDX-License-Identifier: GPL-3.0-or-later
<script lang="ts">
  import type { Entry, TreeStats } from '../lib/types';
  import { formatBytes, formatDate } from '../lib/format';
  import Icon from '../components/Icon.svelte';
  import EntryIcon from '../components/EntryIcon.svelte';

  interface Props {
    root: TreeStats | null;
    loading: boolean;
    path: string;
    onopen: (entry: Entry) => void;
  }

  let { root, loading, path, onopen }: Props = $props();

  // Filters
  let text = $state('');
  let exts = $state('');
  let minSize = $state('');
  let maxSize = $state('');
  let dateFrom = $state('');
  let dateTo = $state('');
  let showHidden = $state(false);
  let sort = $state<'name' | 'size' | 'modified'>('name');
  let descending = $state(false);

  let allFiles = $derived.by((): Entry[] => {
    if (!root) return [];
    const out: Entry[] = [];
    const walk = (node: TreeStats) => {
      if (node.node.kind === 'file') out.push(node.node);
      for (const child of node.children) walk(child);
    };
    walk(root);
    return out;
  });

  let filtered = $derived.by((): Entry[] => {
    const needle = text.trim().toLowerCase();
    const extList = exts
      .split(',')
      .map((e) => e.trim().toLowerCase().replace(/^\./, ''))
      .filter(Boolean);
    const min = minSize.trim() ? Number(minSize) : null;
    const max = maxSize.trim() ? Number(maxSize) : null;
    const fromMs = dateFrom ? new Date(dateFrom).getTime() : null;
    const toMs = dateTo ? new Date(dateTo).getTime() + 86_400_000 : null;

    const out = allFiles.filter((entry) => {
      if (!showHidden && entry.isHidden) return false;
      if (needle && !entry.name.toLowerCase().includes(needle) && !entry.path.toLowerCase().includes(needle)) {
        return false;
      }
      if (extList.length > 0 && !extList.includes(entry.ext ?? '')) return false;
      if (min !== null && entry.size < min) return false;
      if (max !== null && entry.size > max) return false;
      if (fromMs !== null && entry.mtimeMs < fromMs) return false;
      if (toMs !== null && entry.mtimeMs > toMs) return false;
      return true;
    });

    out.sort((a, b) => {
      let cmp = 0;
      switch (sort) {
        case 'size':
          cmp = a.size - b.size;
          break;
        case 'modified':
          cmp = a.mtimeMs - b.mtimeMs;
          break;
        default:
          cmp = a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' });
      }
      return descending ? -cmp : cmp;
    });

    return out.slice(0, 500);
  });

  function clearFilters() {
    text = '';
    exts = '';
    minSize = '';
    maxSize = '';
    dateFrom = '';
    dateTo = '';
    showHidden = false;
  }

  function parentOf(path: string): string {
    const idx = path.lastIndexOf('/');
    return idx <= 0 ? '/' : path.slice(0, idx);
  }
</script>

<div class="query">
  <div class="filters">
    <div class="row">
      <label>
        <span>Name contains</span>
        <input bind:value={text} placeholder="search…" spellcheck="false" />
      </label>
      <label>
        <span>Extensions</span>
        <input bind:value={exts} placeholder="pdf, jpg, rs" spellcheck="false" />
      </label>
    </div>
    <div class="row">
      <label>
        <span>Min size (bytes)</span>
        <input bind:value={minSize} type="number" min="0" placeholder="0" />
      </label>
      <label>
        <span>Max size (bytes)</span>
        <input bind:value={maxSize} type="number" min="0" placeholder="∞" />
      </label>
      <label>
        <span>Modified after</span>
        <input bind:value={dateFrom} type="date" />
      </label>
      <label>
        <span>Modified before</span>
        <input bind:value={dateTo} type="date" />
      </label>
    </div>
    <div class="row actions">
      <label class="checkbox">
        <input type="checkbox" bind:checked={showHidden} />
        <span>Show hidden</span>
      </label>
      <span class="spacer"></span>
      <button class="chip" onclick={clearFilters}>Clear</button>
      <div class="sort">
        <select bind:value={sort} aria-label="Sort by">
          <option value="name">Name</option>
          <option value="size">Size</option>
          <option value="modified">Modified</option>
        </select>
        <button class="dir" onclick={() => (descending = !descending)}>
          {descending ? '↓' : '↑'}
        </button>
      </div>
    </div>
  </div>

  <div class="summary">
    {#if loading}
      <span class="pulse"><span class="dot"></span>loading tree…</span>
    {:else}
      <span>{filtered.length}{#if filtered.length === 500}+{/if} of {allFiles.length} files match</span>
    {/if}
  </div>

  <div class="results">
    {#if loading}
      <div class="state"><span class="spinner"></span><p>Aggregating {path}…</p></div>
    {:else if filtered.length === 0}
      <div class="state">
        <Icon name="search" size={24} />
        <p>No files match these filters.</p>
      </div>
    {:else}
      <div class="head">
        <span>Name</span>
        <span class="right">Size</span>
        <span>Modified</span>
        <span>Folder</span>
      </div>
      {#each filtered as entry (entry.id)}
        <button
          class="row"
          onclick={() => onopen(entry)}
          ondblclick={() => onopen(entry)}
          title={entry.path}
        >
          <span class="name-cell">
            <EntryIcon {entry} size={14} />
            <span class="name truncate">{entry.name}</span>
          </span>
          <span class="size mono">{formatBytes(entry.size)}</span>
          <span class="date">{formatDate(entry.mtimeMs)}</span>
          <span class="folder truncate" title={parentOf(entry.path)}>{parentOf(entry.path)}</span>
        </button>
      {/each}
    {/if}
  </div>
</div>

<style>
  .query {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .filters {
    padding: 12px;
    border-bottom: 1px solid var(--line-soft);
    background: var(--bg);
    display: flex;
    flex-direction: column;
    gap: 10px;
    flex: none;
  }

  .row {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }

  .row.actions {
    align-items: center;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 3px;
    flex: 1;
    min-width: 140px;
  }

  label span {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: var(--text-faint);
  }

  label input {
    height: 30px;
    padding: 0 9px;
    background: var(--bg-2);
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
    color: var(--text);
    font-size: 12px;
  }

  label input:focus {
    border-color: var(--accent);
    outline: none;
  }

  .checkbox {
    flex-direction: row;
    align-items: center;
    gap: 6px;
    min-width: 0;
    flex: 0;
  }

  .checkbox span {
    text-transform: none;
    letter-spacing: 0;
    font-size: 12px;
    color: var(--text-dim);
  }

  .checkbox input {
    width: 14px;
    height: 14px;
    accent-color: var(--accent);
  }

  .spacer {
    flex: 1;
  }

  .chip {
    height: 28px;
    padding: 0 11px;
    border-radius: 999px;
    border: 1px solid var(--line);
    color: var(--text-dim);
    font-size: 11px;
  }

  .chip:hover {
    background: var(--bg-3);
    color: var(--text);
  }

  .sort {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 28px;
    padding: 0 5px 0 8px;
    border-radius: 999px;
    border: 1px solid var(--line);
    color: var(--text-faint);
  }

  .sort select {
    appearance: none;
    background: transparent;
    border: 0;
    outline: none;
    color: var(--text-dim);
    font-size: 11px;
    cursor: pointer;
  }

  .sort select option {
    background: var(--bg-2);
    color: var(--text);
  }

  .dir {
    width: 20px;
    height: 20px;
    border-radius: 5px;
    color: var(--text-dim);
    font-size: 12px;
  }

  .dir:hover {
    background: var(--bg-3);
    color: var(--text);
  }

  .summary {
    padding: 6px 12px;
    font-size: 11px;
    color: var(--text-faint);
    border-bottom: 1px solid var(--line-soft);
    flex: none;
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

  .results {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .head,
  .row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 80px 110px minmax(0, 1fr);
    align-items: center;
    gap: 10px;
    padding: 0 14px;
  }

  .head {
    height: 26px;
    position: sticky;
    top: 0;
    background: var(--bg);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: var(--text-faint);
    border-bottom: 1px solid var(--line-soft);
    z-index: 1;
  }

  .right {
    text-align: right;
  }

  .row {
    height: 28px;
    text-align: left;
    color: var(--text-dim);
    font-size: 12px;
    border-radius: 0;
  }

  .row:hover {
    background: var(--bg-2);
    color: var(--text);
  }

  .name-cell {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .name-cell :global(svg) {
    flex: none;
    color: var(--text-faint);
  }

  .size {
    text-align: right;
    font-size: 11px;
    color: var(--text-faint);
  }

  .date {
    font-size: 11px;
    color: var(--text-faint);
  }

  .folder {
    font-size: 10.5px;
    color: var(--text-faint);
    font-family: var(--mono);
  }

  .state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    height: 100%;
    padding: 30px;
    color: var(--text-faint);
    text-align: center;
    font-size: 12.5px;
  }

  .state p {
    margin: 0;
    max-width: 420px;
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
