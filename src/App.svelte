<script lang="ts">
  // SPDX-License-Identifier: GPL-3.0-or-later
  import { onMount } from 'svelte';
  import { backend, isTauri } from './lib/api';
  import type { Entry, Place, SortKey, TreeStats, ViewId } from './lib/types';
  import { parentOf } from './lib/format';
  import Toolbar from './components/Toolbar.svelte';
  import Sidebar from './components/Sidebar.svelte';
  import ViewHeader from './components/ViewHeader.svelte';
  import PreviewPane from './components/PreviewPane.svelte';
  import StatusBar from './components/StatusBar.svelte';
  import QuickLook from './components/QuickLook.svelte';
  import ContextMenu from './components/ContextMenu.svelte';
  import Icon from './components/Icon.svelte';
  import ThemePicker from './components/ThemePicker.svelte';
  import ThemeEditor from './components/ThemeEditor.svelte';
  import { getTheme, applyTheme, loadSavedThemeId, saveThemeId } from './lib/themes';
  import ListView from './views/ListView.svelte';
  import TreemapView from './views/TreemapView.svelte';
  import MosaicView from './views/MosaicView.svelte';
  import SunburstView from './views/SunburstView.svelte';
  import QueryView from './views/QueryView.svelte';
  import ClustersView from './views/ClustersView.svelte';
  import PalaceView from './views/PalaceView.svelte';

  let path = $state('');
  let entries = $state<Entry[]>([]);
  let total = $state(0);
  let nextOffset = $state<number | null>(null);
  let loading = $state(true);
  let loadingMore = $state(false);
  let error = $state<string | null>(null);

  let view = $state<ViewId>('list');
  let sort = $state<SortKey>('name');
  let descending = $state(false);
  let dirsFirst = $state(true);
  let showHidden = $state(false);
  let filter = $state('');

  let selected = $state<Set<string>>(new Set());
  let anchor = $state<string | null>(null);
  let preview = $state<Entry | null>(null);
  let quickLook = $state<Entry | null>(null);
  let contextMenu = $state<{ x: number; y: number; entry: Entry } | null>(null);

  let places = $state<Place[]>([]);
  let previewOpen = $state(true);
  let tileSize = $state(168);
  let themeId = $state(loadSavedThemeId());
  let themeEditorOpen = $state(false);

  let tree = $state<TreeStats | null>(null);
  let treeLoading = $state(false);

  let navHistory = $state<string[]>([]);
  let historyIndex = $state(-1);

  let filterFocus = $state(0);
  let locationFocus = $state(0);

  let home = $state('');
  let navToken = 0;
  let treeToken = 0;
  let filterTimer: ReturnType<typeof setTimeout> | null = null;

  let selectedCount = $derived(selected.size);
  let selectedSize = $derived(
    entries.filter((entry) => selected.has(entry.id)).reduce((sum, entry) => sum + entry.size, 0),
  );
  let canBack = $derived(historyIndex > 0);
  let canForward = $derived(historyIndex >= 0 && historyIndex < navHistory.length - 1);
  let canUp = $derived(path !== '' && path !== '/');
  let hasMore = $derived(nextOffset !== null);

  function filterValue(): string | undefined {
    const value = filter.trim();
    return value.length > 0 ? value : undefined;
  }

  function pushHistory(next: string) {
    if (historyIndex >= 0 && navHistory[historyIndex] === next) return;
    navHistory = [...navHistory.slice(0, historyIndex + 1), next];
    historyIndex = navHistory.length - 1;
  }

  async function navigate(next: string, push = true) {
    if (!next) return;
    const token = ++navToken;
    loading = true;
    error = null;

    try {
      const page = await backend.listDir({
        path: next,
        offset: 0,
        limit: 500,
        sort,
        descending,
        dirsFirst,
        showHidden,
        filter: filterValue(),
      });
      if (token !== navToken) return;

      path = page.path;
      entries = page.entries;
      total = page.total;
      nextOffset = page.nextOffset;
      selected = new Set();
      anchor = null;
      preview = null;
      quickLook = null;
      tree = null;

      if (push) pushHistory(page.path);
      // Only index when a tree-dependent view is actually open.
      if (view === 'treemap' || view === 'sunburst' || view === 'query' || view === 'clusters' || view === 'palace') void loadTree();
      if (isTauri) backend.watchStart([page.path]).catch(() => {});
      updateHash(page.path);
    } catch (cause) {
      if (token !== navToken) return;
      error = cause instanceof Error ? cause.message : String(cause);
      entries = [];
      total = 0;
      nextOffset = null;
    } finally {
      if (token === navToken) loading = false;
    }
  }

  async function reload() {
    if (!path) return;
    const token = ++navToken;
    loading = true;
    error = null;

    try {
      const page = await backend.listDir({
        path,
        offset: 0,
        limit: 500,
        sort,
        descending,
        dirsFirst,
        showHidden,
        filter: filterValue(),
      });
      if (token !== navToken) return;

      entries = page.entries;
      total = page.total;
      nextOffset = page.nextOffset;

      const ids = new Set(page.entries.map((entry) => entry.id));
      selected = new Set([...selected].filter((id) => ids.has(id)));
      if (preview && !ids.has(preview.id)) preview = null;
      if (view === 'treemap' || view === 'sunburst' || view === 'query' || view === 'clusters' || view === 'palace') void loadTree();
    } catch (cause) {
      if (token !== navToken) return;
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      if (token === navToken) loading = false;
    }
  }

  async function loadMore() {
    if (nextOffset === null || loadingMore || loading) return;
    const token = navToken;
    loadingMore = true;

    try {
      const page = await backend.listDir({
        path,
        offset: nextOffset,
        limit: 500,
        sort,
        descending,
        dirsFirst,
        showHidden,
        filter: filterValue(),
      });
      if (token !== navToken) return;
      entries = [...entries, ...page.entries];
      nextOffset = page.nextOffset;
      total = page.total;
    } catch {
      // A failed page keeps the already loaded rows; scrolling retries.
    } finally {
      loadingMore = false;
    }
  }

  function goBack() {
    if (!canBack) return;
    historyIndex -= 1;
    void navigate(navHistory[historyIndex], false);
  }

  function goForward() {
    if (!canForward) return;
    historyIndex += 1;
    void navigate(navHistory[historyIndex], false);
  }

  function goUp() {
    if (!canUp) return;
    const parent = parentOf(path);
    if (parent !== path) void navigate(parent);
  }

  async function doRename(entry: Entry) {
    const name = window.prompt('Rename to:', entry.name);
    if (!name || name === entry.name) return;
    try {
      const next = await backend.renamePath(entry.path, name);
      await reload();
      const found = entries.find((e) => e.path === next);
      if (found) {
        selected = new Set([found.id]);
        preview = found;
      }
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  async function doTrash(entriesToTrash: Entry[]) {
    if (entriesToTrash.length === 0) return;
    if (!window.confirm(`Move ${entriesToTrash.length} item(s) to trash?`)) return;
    try {
      await backend.trashItems(entriesToTrash.map((e) => e.path));
      await reload();
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  async function doMove(entriesToMove: Entry[]) {
    if (entriesToMove.length === 0) return;
    const dest = window.prompt('Move to folder (absolute path):');
    if (!dest) return;
    try {
      await backend.moveItems(entriesToMove.map((e) => e.path), dest.trim());
      await reload();
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  function open(entry: Entry) {
    if (entry.kind === 'dir') {
      void navigate(entry.path);
    } else {
      backend.openPath(entry.path).catch((cause) => {
        error = cause instanceof Error ? cause.message : String(cause);
      });
    }
  }

  function reveal(entry: Entry) {
    backend.reveal(entry.path).catch(() => {});
  }

  function select(entry: Entry, event: MouseEvent) {
    if (event.shiftKey && anchor) {
      const from = entries.findIndex((item) => item.id === anchor);
      const to = entries.findIndex((item) => item.id === entry.id);
      if (from >= 0 && to >= 0) {
        const [low, high] = from < to ? [from, to] : [to, from];
        selected = new Set(entries.slice(low, high + 1).map((item) => item.id));
      }
    } else if (event.ctrlKey || event.metaKey) {
      const next = new Set(selected);
      if (next.has(entry.id)) next.delete(entry.id);
      else next.add(entry.id);
      selected = next;
    } else {
      selected = new Set([entry.id]);
    }

    anchor = entry.id;
    preview = entry;
  }

  function moveSelection(delta: number) {
    if (entries.length === 0) return;
    const currentId = preview?.id ?? anchor;
    let index = currentId ? entries.findIndex((entry) => entry.id === currentId) : -1;
    index = Math.max(0, Math.min(entries.length - 1, index + delta));
    const entry = entries[index];
    selected = new Set([entry.id]);
    anchor = entry.id;
    preview = entry;
  }

  const treeCache = $state(new Map<string, TreeStats>());

  async function fetchTree(target: string) {
    if (!target) return;
    const cached = treeCache.get(target);
    if (cached) {
      if (target === path) {
        tree = cached;
        treeLoading = false;
      }
      return;
    }
    const token = ++treeToken;
    treeLoading = true;
    tree = null;
    try {
      const stats = await backend.treeStats(target, 4);
      treeCache.set(target, stats);
      if (token === treeToken && target === path) {
        tree = stats;
        treeLoading = false;
      }
    } catch {
      if (token === treeToken && target === path) {
        tree = null;
        treeLoading = false;
      }
    }
  }

  function loadTree() {
    void fetchTree(path);
  }

  function setTheme(id: string) {
    themeId = id;
    saveThemeId(id);
    applyTheme(getTheme(id));
  }

  function onThemeCreated(id: string) {
    themeId = id;
    saveThemeId(id);
    applyTheme(getTheme(id));
  }

  function setView(next: ViewId) {
    view = next;
    if (next === 'treemap' || next === 'sunburst' || next === 'query' || next === 'clusters' || next === 'palace') void loadTree();
    updateHash(path);
  }

  function updateHash(nextPath: string) {
    if (typeof window === 'undefined' || !nextPath) return;
    window.history.replaceState(
      null,
      '',
      `#view=${view}&path=${encodeURIComponent(nextPath)}`,
    );
  }

  function setSort(next: SortKey) {
    sort = next;
    void reload();
  }

  function toggleDescending() {
    descending = !descending;
    void reload();
  }

  function toggleHidden() {
    showHidden = !showHidden;
    void reload();
  }

  function onFilterChange(value: string) {
    filter = value;
    if (filterTimer) clearTimeout(filterTimer);
    filterTimer = setTimeout(() => {
      filterTimer = null;
      void reload();
    }, 180);
  }

  function onKey(event: KeyboardEvent) {
    const target = event.target as HTMLElement | null;
    const typing =
      !!target &&
      (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable);

    const mod = event.ctrlKey || event.metaKey;

    if (mod && event.key.toLowerCase() === 'l') {
      event.preventDefault();
      locationFocus += 1;
      return;
    }
    if (mod && event.key.toLowerCase() === 'f') {
      event.preventDefault();
      filterFocus += 1;
      return;
    }
    if (mod && event.key.toLowerCase() === 'q') {
      event.preventDefault();
      if (isTauri) {
        import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
          getCurrentWindow().close().catch(() => {});
        });
      }
      return;
    }
     if (mod && ['1', '2', '3', '4', '5', '6', '7'].includes(event.key)) {
      event.preventDefault();
       setView(event.key === '1' ? 'list' : event.key === '2' ? 'treemap' : event.key === '3' ? 'mosaic' : event.key === '4' ? 'sunburst' : event.key === '5' ? 'query' : event.key === '6' ? 'clusters' : 'palace');
      return;
    }
    if (event.altKey && event.key === 'ArrowLeft') {
      event.preventDefault();
      goBack();
      return;
    }
    if (event.altKey && event.key === 'ArrowRight') {
      event.preventDefault();
      goForward();
      return;
    }

    if (typing) {
      if (event.key === 'Escape') target?.blur();
      return;
    }

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        moveSelection(1);
        break;
      case 'ArrowUp':
        event.preventDefault();
        moveSelection(-1);
        break;
      case 'Enter':
        if (preview) {
          event.preventDefault();
          open(preview);
        }
        break;
      case 'Backspace':
        event.preventDefault();
        goUp();
        break;
      case ' ':
        if (preview) {
          event.preventDefault();
          quickLook = preview;
        }
        break;
      case 'Escape':
        quickLook = null;
        break;
      case 'F5':
        event.preventDefault();
        void reload();
        break;
      case 'Delete':
        if (selected.size > 0) {
          event.preventDefault();
          void doTrash(entries.filter((e) => selected.has(e.id)));
        }
        break;
      default:
        break;
    }
  }

  onMount(() => {
    applyTheme(getTheme(themeId));
    let unlisten: (() => void) | null = null;

    void (async () => {
      try {
        const params = new URLSearchParams(window.location.hash.replace(/^#/, ''));
        const requestedView = params.get('view');
        if (requestedView === 'list' || requestedView === 'treemap' || requestedView === 'mosaic' || requestedView === 'sunburst' || requestedView === 'query' || requestedView === 'clusters' || requestedView === 'palace') {
          view = requestedView;
        }
        const requestedPath = params.get('path');

        places = await backend.places();
        home = await backend.homeDir();
        await navigate(requestedPath || home || '/');
        if (isTauri) {
          unlisten = await backend.onChanged((dirs) => {
            if (dirs.includes(path)) void reload();
          });
        }
      } catch (cause) {
        error = cause instanceof Error ? cause.message : String(cause);
        loading = false;
      }
    })();

    window.addEventListener('keydown', onKey);
    return () => {
      window.removeEventListener('keydown', onKey);
      unlisten?.();
    };
  });
</script>

<div class="app">
  <Toolbar
    {path}
    bind:filter
    {view}
    {canBack}
    {canForward}
    {canUp}
    {previewOpen}
    {filterFocus}
    {locationFocus}
    onback={goBack}
    onforward={goForward}
    onup={goUp}
    onreload={() => void reload()}
    onnavigate={(next) => void navigate(next)}
    onfilter={onFilterChange}
    onview={setView}
    ontogglepreview={() => (previewOpen = !previewOpen)}
  />

  <div class="body">
    <Sidebar {places} currentPath={path} mock={!isTauri} onnavigate={(next) => void navigate(next)} />

    <main class="main">
      <ViewHeader
        {total}
        shown={entries.length}
        {sort}
        {descending}
        {showHidden}
        {view}
        {tileSize}
        onsort={setSort}
        ondesc={toggleDescending}
        onhidden={toggleHidden}
        ontile={(size) => (tileSize = size)}
      />

      <div class="view">
        {#if error}
          <div class="state">
            <Icon name="alert" size={24} />
            <p>{error}</p>
            <button onclick={() => void reload()}>Retry</button>
          </div>
        {:else if loading && entries.length === 0}
          <div class="state">
            <span class="spinner"></span>
            <p>Opening {path || 'home'}…</p>
          </div>
        {:else if view === 'treemap'}
          <TreemapView
            root={tree}
            loading={treeLoading}
            {path}
            onnavigate={(next) => void navigate(next)}
            onopen={open}
          />
        {:else if view === 'sunburst'}
          <SunburstView
            root={tree}
            loading={treeLoading}
            {path}
            onnavigate={(next) => void navigate(next)}
            onopen={open}
          />
        {:else if view === 'query'}
          <QueryView
            root={tree}
            loading={treeLoading}
            onopen={open}
          />
        {:else if view === 'clusters'}
          <ClustersView
            root={tree}
            loading={treeLoading}
            onopen={open}
          />
        {:else if view === 'palace'}
          <PalaceView
            root={tree}
            loading={treeLoading}
            onnavigate={(next) => void navigate(next)}
            onopen={open}
          />
        {:else if entries.length === 0}
          <div class="state">
            <Icon name="folder" size={26} />
            <p>This folder is empty.</p>
          </div>
        {:else if view === 'list'}
          <ListView
            {entries}
            {selected}
            onselect={select}
            onopen={open}
            oncontext={(entry, e) => (contextMenu = { x: e.clientX, y: e.clientY, entry })}
            onloadmore={loadMore}
            hasmore={hasMore}
            {loadingMore}
          />
        {:else}
          <MosaicView
            {entries}
            {selected}
            {tileSize}
            onselect={select}
            onopen={open}
            oncontext={(entry, e) => (contextMenu = { x: e.clientX, y: e.clientY, entry })}
            onloadmore={loadMore}
            hasmore={hasMore}
          />
        {/if}
      </div>
    </main>

    {#if previewOpen}
      <PreviewPane entry={preview} onopen={open} onreveal={reveal} />
    {/if}
  </div>

  <div class="theme-float">
    <ThemePicker current={themeId} onchange={setTheme} oncreate={() => (themeEditorOpen = true)} />
  </div>

  <ThemeEditor open={themeEditorOpen} themeId={themeId} onclose={() => (themeEditorOpen = false)} onsaved={onThemeCreated} />

  <StatusBar
    {total}
    shown={entries.length}
    {selectedCount}
    {selectedSize}
    {loading}
    mock={!isTauri}
  />

  {#if contextMenu}
    <ContextMenu
      x={contextMenu.x}
      y={contextMenu.y}
      onopen={() => open(contextMenu!.entry)}
      onrename={() => void doRename(contextMenu!.entry)}
      ontrash={() => void doTrash([contextMenu!.entry])}
      onmove={() => void doMove([contextMenu!.entry])}
      onclose={() => (contextMenu = null)}
    />
  {/if}

  {#if quickLook}
    <QuickLook entry={quickLook} onclose={() => (quickLook = null)} />
  {/if}
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .body {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  .main {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }

  .view {
    position: relative;
    flex: 1;
    min-height: 0;
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
    word-break: break-word;
  }

  .state button {
    height: 30px;
    padding: 0 14px;
    border-radius: 7px;
    border: 1px solid var(--line);
    color: var(--text-dim);
    font-size: 11.5px;
  }

  .theme-float {
    position: fixed;
    top: 10px;
    right: 12px;
    z-index: 100;
  }

  .state button:hover {
    background: var(--bg-3);
    color: var(--text);
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
