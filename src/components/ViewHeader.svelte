<script lang="ts">
  // SPDX-License-Identifier: GPL-3.0-or-later
  import type { SortKey, ViewId } from '../lib/types';
  import Icon from './Icon.svelte';

  interface Props {
    total: number;
    shown: number;
    sort: SortKey;
    descending: boolean;
    showHidden: boolean;
    view: ViewId;
    tileSize: number;
    onsort: (sort: SortKey) => void;
    ondesc: () => void;
    onhidden: () => void;
    ontile: (size: number) => void;
  }

  let {
    total,
    shown,
    sort,
    descending,
    showHidden,
    view,
    tileSize,
    onsort,
    ondesc,
    onhidden,
    ontile,
  }: Props = $props();
</script>

<div class="view-header">
  <span class="count">
    {shown}{#if total > shown}<span class="muted"> of {total}</span>{/if}
    <span class="muted">items</span>
  </span>

  <span class="spacer"></span>

  {#if view === 'mosaic'}
    <label class="density" title="Thumbnail size">
      <Icon name="grid" size={13} />
      <input
        type="range"
        min="110"
        max="260"
        step="10"
        value={tileSize}
        oninput={(event) => ontile(Number((event.target as HTMLInputElement).value))}
        aria-label="Thumbnail size"
      />
    </label>
  {/if}

  <button class="chip" class:active={showHidden} onclick={onhidden} title="Show hidden files">
    <Icon name={showHidden ? 'eye' : 'eye-off'} size={14} />
    <span>Hidden</span>
  </button>

  <div class="sort">
    <Icon name="sort" size={13} />
    <select
      value={sort}
      onchange={(event) => onsort((event.target as HTMLSelectElement).value as SortKey)}
      aria-label="Sort by"
    >
      <option value="name">Name</option>
      <option value="size">Size</option>
      <option value="modified">Modified</option>
      <option value="type">Type</option>
    </select>
    <button class="dir" onclick={ondesc} title="Toggle sort direction">
      {descending ? '↓' : '↑'}
    </button>
  </div>
</div>

<style>
  .view-header {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 38px;
    padding: 0 12px;
    border-bottom: 1px solid var(--line-soft);
    background: var(--bg);
    flex: none;
  }

  .count {
    font-size: 11.5px;
    color: var(--text-dim);
  }

  .muted {
    color: var(--text-faint);
  }

  .spacer {
    flex: 1;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 26px;
    padding: 0 9px;
    border-radius: 999px;
    border: 1px solid var(--line);
    color: var(--text-faint);
    font-size: 11px;
    transition: color 120ms ease, border-color 120ms ease, background 120ms ease;
  }

  .chip:hover {
    color: var(--text-dim);
    background: var(--bg-2);
  }

  .chip.active {
    color: var(--accent);
    border-color: rgba(94, 234, 212, 0.35);
    background: rgba(94, 234, 212, 0.08);
  }

  .density {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    color: var(--text-faint);
  }

  .density input {
    width: 92px;
    accent-color: var(--accent);
  }

  .sort {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 26px;
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
    padding: 0 2px;
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
</style>
