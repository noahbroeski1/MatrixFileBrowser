<script lang="ts">
  // SPDX-License-Identifier: GPL-3.0-or-later
  import type { Entry } from '../lib/types';
  import { formatBytes, formatDate } from '../lib/format';
  import EntryIcon from '../components/EntryIcon.svelte';

  interface Props {
    entries: Entry[];
    selected: Set<string>;
    onselect: (entry: Entry, event: MouseEvent) => void;
    onopen: (entry: Entry) => void;
    oncontext?: (entry: Entry, event: MouseEvent) => void;
    onloadmore: () => void;
    hasmore: boolean;
    loadingMore: boolean;
  }

  let { entries, selected, onselect, onopen, oncontext, onloadmore, hasmore, loadingMore }: Props = $props();

  const ROW = 30;

  let scroller: HTMLDivElement | null = $state(null);
  let viewportH = $state(600);
  let scrollTop = $state(0);

  let start = $derived(Math.max(0, Math.floor(scrollTop / ROW) - 10));
  let count = $derived(Math.ceil(viewportH / ROW) + 22);
  let visible = $derived(entries.slice(start, start + count));
  let totalHeight = $derived(entries.length * ROW);

  $effect(() => {
    const el = scroller;
    if (!el) return;
    viewportH = el.clientHeight;
    const observer = new ResizeObserver(() => {
      viewportH = el.clientHeight;
    });
    observer.observe(el);
    return () => observer.disconnect();
  });

  // Keep the keyboard-selected row in view without fighting manual scrolling.
  $effect(() => {
    const el = scroller;
    if (!el || selected.size !== 1) return;
    const id = selected.values().next().value;
    const index = entries.findIndex((entry) => entry.id === id);
    if (index < 0) return;
    const y = index * ROW;
    const top = el.scrollTop;
    const bottom = top + el.clientHeight;
    if (y < top) el.scrollTop = y;
    else if (y + ROW > bottom) el.scrollTop = y + ROW - el.clientHeight;
  });

  function onScroll(event: Event) {
    const el = event.currentTarget as HTMLDivElement;
    scrollTop = el.scrollTop;
    if (hasmore && el.scrollTop + el.clientHeight > el.scrollHeight - 600) onloadmore();
  }
</script>

<div class="list-wrap">
  <div class="head">
    <span>Name</span>
    <span class="right">Size</span>
    <span>Modified</span>
  </div>

  <div class="scroller" bind:this={scroller} onscroll={onScroll}>
    <div class="spacer" style="height:{totalHeight}px">
      {#each visible as entry, index (entry.id)}
        <button
          class="row"
          class:selected={selected.has(entry.id)}
          class:hidden-file={entry.isHidden}
          style="transform: translateY({(start + index) * ROW}px)"
          onclick={(event) => onselect(entry, event)}
          ondblclick={() => onopen(entry)}
          oncontextmenu={(event) => { event.preventDefault(); oncontext?.(entry, event); }}
          title={entry.path}
        >
          <span class="name-cell">
            <EntryIcon {entry} size={15} />
            <span class="name truncate">{entry.name}</span>
          </span>
          <span class="size mono">{formatBytes(entry.size)}</span>
          <span class="date">{formatDate(entry.mtimeMs)}</span>
        </button>
      {/each}
    </div>
    {#if loadingMore}
      <div class="loading-more">Loading more…</div>
    {/if}
  </div>
</div>

<style>
  .list-wrap {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .head,
  .row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 92px 132px;
    align-items: center;
    gap: 12px;
    padding: 0 14px;
  }

  .head {
    height: 28px;
    flex: none;
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: var(--text-faint);
    border-bottom: 1px solid var(--line-soft);
  }

  .right {
    text-align: right;
  }

  .scroller {
    position: relative;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .spacer {
    position: relative;
    width: 100%;
  }

  .row {
    position: absolute;
    inset: 0 0 auto 0;
    height: 30px;
    width: 100%;
    text-align: left;
    border-radius: 0;
    color: var(--text-dim);
    font-size: 12.5px;
    will-change: transform;
  }

  .row:hover {
    background: var(--bg-2);
    color: var(--text);
  }

  .row.selected {
    background: rgba(94, 234, 212, 0.1);
    color: var(--text);
    box-shadow: inset 2px 0 0 var(--accent);
  }

  .row.hidden-file .name {
    opacity: 0.55;
  }

  .name-cell {
    display: flex;
    align-items: center;
    gap: 9px;
    min-width: 0;
  }

  .name-cell :global(svg) {
    flex: none;
    color: var(--text-faint);
  }

  .row.selected .name-cell :global(svg) {
    color: var(--accent);
  }

  .name {
    min-width: 0;
  }

  .size {
    text-align: right;
    font-size: 11.5px;
    color: var(--text-faint);
  }

  .date {
    font-size: 11.5px;
    color: var(--text-faint);
  }

  .loading-more {
    position: sticky;
    bottom: 0;
    padding: 7px;
    text-align: center;
    font-size: 11px;
    color: var(--text-faint);
    background: linear-gradient(to top, var(--bg), transparent);
  }
</style>
