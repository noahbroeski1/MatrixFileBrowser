<script lang="ts">
  // SPDX-License-Identifier: GPL-3.0-or-later
  import { tick } from 'svelte';
  import type { ViewId } from '../lib/types';
  import Icon from './Icon.svelte';
  import Breadcrumb from './Breadcrumb.svelte';

  interface Props {
    path: string;
    filter: string;
    view: ViewId;
    canBack: boolean;
    canForward: boolean;
    canUp: boolean;
    previewOpen: boolean;
    filterFocus: number;
    locationFocus: number;
    onback: () => void;
    onforward: () => void;
    onup: () => void;
    onreload: () => void;
    onnavigate: (path: string) => void;
    onfilter: (value: string) => void;
    onview: (view: ViewId) => void;
    ontogglepreview: () => void;
  }

  let {
    path,
    filter = $bindable(''),
    view,
    canBack,
    canForward,
    canUp,
    previewOpen,
    filterFocus,
    locationFocus,
    onback,
    onforward,
    onup,
    onreload,
    onnavigate,
    onfilter,
    onview,
    ontogglepreview,
  }: Props = $props();

  let editing = $state(false);
  let draft = $state('');
  let locationEl: HTMLInputElement | null = $state(null);
  let filterEl: HTMLInputElement | null = $state(null);
  let handledLocationFocus = 0;
  let handledFilterFocus = 0;

  $effect(() => {
    if (locationFocus > 0 && locationFocus !== handledLocationFocus) {
      handledLocationFocus = locationFocus;
      void startEditing();
    }
  });

  $effect(() => {
    if (filterFocus > 0 && filterFocus !== handledFilterFocus) {
      handledFilterFocus = filterFocus;
      filterEl?.focus();
      filterEl?.select();
    }
  });

  async function startEditing() {
    editing = true;
    draft = path;
    await tick();
    locationEl?.focus();
    locationEl?.select();
  }

  function submitLocation() {
    const next = draft.trim();
    editing = false;
    if (next && next !== path) onnavigate(next);
  }

  function onLocationKey(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault();
      submitLocation();
    } else if (event.key === 'Escape') {
      event.preventDefault();
      editing = false;
    }
  }

  function clearFilter() {
    filter = '';
    onfilter('');
    filterEl?.focus();
  }
</script>

<header class="toolbar">
  <div class="group">
    <button class="icon-btn" onclick={onback} disabled={!canBack} title="Back (Alt+Left)">
      <Icon name="chevron-left" size={17} />
    </button>
    <button class="icon-btn" onclick={onforward} disabled={!canForward} title="Forward (Alt+Right)">
      <Icon name="chevron-right" size={17} />
    </button>
    <button class="icon-btn" onclick={onup} disabled={!canUp} title="Up (Backspace)">
      <Icon name="arrow-up" size={16} />
    </button>
    <button class="icon-btn" onclick={onreload} title="Reload (F5)">
      <Icon name="refresh" size={15} />
    </button>
  </div>

  <div class="location">
    {#if editing}
      <input
        bind:this={locationEl}
        bind:value={draft}
        class="location-input mono"
        spellcheck="false"
        autocomplete="off"
        onkeydown={onLocationKey}
        onblur={() => (editing = false)}
        aria-label="Location"
      />
    {:else}
      <Breadcrumb {path} onnavigate={onnavigate} />
      <button class="icon-btn tiny" onclick={startEditing} title="Edit location (Ctrl+L)">
        <Icon name="code" size={13} />
      </button>
    {/if}
  </div>

  <label class="search">
    <Icon name="search" size={14} />
    <input
      bind:this={filterEl}
      bind:value={filter}
      oninput={() => onfilter(filter)}
      placeholder="Filter this folder…"
      spellcheck="false"
      autocomplete="off"
      aria-label="Filter"
    />
    {#if filter}
      <button class="clear" onclick={clearFilter} title="Clear filter">
        <Icon name="x" size={12} />
      </button>
    {/if}
  </label>

  <div class="segmented" role="group" aria-label="View">
    <button class:active={view === 'list'} onclick={() => onview('list')} title="List view (Ctrl+1)">
      <Icon name="list" size={15} />
    </button>
    <button class:active={view === 'treemap'} onclick={() => onview('treemap')} title="Treemap (Ctrl+2)">
      <Icon name="treemap" size={15} />
    </button>
    <button class:active={view === 'mosaic'} onclick={() => onview('mosaic')} title="Mosaic (Ctrl+3)">
      <Icon name="grid" size={15} />
    </button>
    <button class:active={view === 'sunburst'} onclick={() => onview('sunburst')} title="Sunburst (Ctrl+4)">
      <Icon name="sunburst" size={15} />
    </button>
    <button class:active={view === 'query'} onclick={() => onview('query')} title="Query workbench (Ctrl+5)">
      <Icon name="search" size={15} />
    </button>
    <button class:active={view === 'clusters'} onclick={() => onview('clusters')} title="Clusters (Ctrl+6)">
      <Icon name="sparkles" size={15} />
    </button>
  </div>

  <button
    class="icon-btn"
    class:active={previewOpen}
    onclick={ontogglepreview}
    title="Toggle preview pane"
  >
    <Icon name="panel-right" size={15} />
  </button>
</header>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 52px;
    padding: 0 12px;
    background: var(--bg-1);
    border-bottom: 1px solid var(--line);
    flex: none;
  }

  .group {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border-radius: var(--radius-sm);
    color: var(--text-dim);
    transition: background 120ms ease, color 120ms ease;
  }

  .icon-btn:hover:not(:disabled) {
    background: var(--bg-3);
    color: var(--text);
  }

  .icon-btn:disabled {
    opacity: 0.3;
    cursor: default;
  }

  .icon-btn.tiny {
    width: 24px;
    height: 24px;
  }

  .icon-btn.active {
    color: var(--accent);
    background: var(--bg-3);
  }

  .location {
    display: flex;
    align-items: center;
    gap: 2px;
    flex: 1;
    min-width: 120px;
    height: 34px;
    padding: 0 4px 0 8px;
    background: var(--bg-2);
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .location-input {
    width: 100%;
    height: 100%;
    background: transparent;
    border: 0;
    outline: none;
    font-size: 12.5px;
  }

  .search {
    display: flex;
    align-items: center;
    gap: 7px;
    width: 230px;
    height: 34px;
    padding: 0 9px;
    background: var(--bg-2);
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
    color: var(--text-faint);
  }

  .search:focus-within {
    border-color: var(--accent);
  }

  .search input {
    flex: 1;
    min-width: 0;
    background: transparent;
    border: 0;
    outline: none;
    font-size: 12.5px;
    color: var(--text);
  }

  .search input::placeholder {
    color: var(--text-faint);
  }

  .clear {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: 5px;
    color: var(--text-dim);
  }

  .clear:hover {
    background: var(--bg-4);
    color: var(--text);
  }

  .segmented {
    display: flex;
    gap: 2px;
    padding: 3px;
    background: var(--bg-2);
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
  }

  .segmented button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 26px;
    border-radius: 5px;
    color: var(--text-dim);
    transition: background 120ms ease, color 120ms ease;
  }

  .segmented button:hover {
    color: var(--text);
  }

  .segmented button.active {
    background: var(--bg-4);
    color: var(--accent);
    box-shadow: inset 0 0 0 1px rgba(94, 234, 212, 0.18);
  }

  @media (max-width: 1000px) {
    .search {
      width: 150px;
    }
  }
</style>
