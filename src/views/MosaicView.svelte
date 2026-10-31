<script lang="ts">
  // SPDX-License-Identifier: GPL-3.0-or-later
  import type { Entry } from '../lib/types';
  import { backend, assetUrl } from '../lib/api';
  import { formatBytes, iconFor, isImage, isVideo } from '../lib/format';
  import Icon from '../components/Icon.svelte';

  interface Props {
    entries: Entry[];
    selected: Set<string>;
    tileSize: number;
    onselect: (entry: Entry, event: MouseEvent) => void;
    onopen: (entry: Entry) => void;
    oncontext?: (entry: Entry, event: MouseEvent) => void;
    onloadmore: () => void;
    hasmore: boolean;
  }

  let { entries, selected, tileSize, onselect, onopen, oncontext, onloadmore, hasmore }: Props = $props();

  let thumbs = $state<Record<string, string>>({});
  let failed = $state<Set<string>>(new Set());
  const requested = new Set<string>();

  $effect(() => {
    const wanted = entries
      .filter(
        (entry) =>
          !requested.has(entry.path) &&
          !failed.has(entry.path) &&
          (isImage(entry.ext) || isVideo(entry.ext) || entry.ext === 'pdf'),
      )
      .map((entry) => entry.path);

    if (wanted.length === 0) return;
    for (const path of wanted) requested.add(path);
    void load(wanted);
  });

  async function load(paths: string[]) {
    const px = Math.round(tileSize * 1.6);
    for (let i = 0; i < paths.length; i += 32) {
      const batch = paths.slice(i, i + 32);
      try {
        const refs = await backend.thumbnails(batch, px);
        if (refs.length === 0) continue;
        const next = { ...thumbs };
        for (const ref of refs) next[ref.path] = assetUrl(ref.thumbPath);
        thumbs = next;
      } catch {
        const next = new Set(failed);
        for (const path of batch) next.add(path);
        failed = next;
      }
    }
  }

  function onScroll(event: Event) {
    const el = event.currentTarget as HTMLDivElement;
    if (hasmore && el.scrollTop + el.clientHeight > el.scrollHeight - 500) onloadmore();
  }
</script>

<div class="mosaic" onscroll={onScroll}>
  <div class="grid" style="--tile:{tileSize}px">
    {#each entries as entry (entry.id)}
      <button
        class="tile"
        class:selected={selected.has(entry.id)}
        onclick={(event) => onselect(entry, event)}
        ondblclick={() => onopen(entry)}
        oncontextmenu={(event) => { event.preventDefault(); oncontext?.(entry, event); }}
        title={`${entry.name} — ${entry.kind === 'dir' ? 'folder' : formatBytes(entry.size)}`}
      >
        <span class="media">
          {#if thumbs[entry.path]}
            <img src={thumbs[entry.path]} alt="" decoding="async" />
          {:else}
            <span class="fallback">
              <Icon name={iconFor(entry)} size={26} />
              <span class="ext">{entry.kind === 'dir' ? 'folder' : entry.ext ?? 'file'}</span>
            </span>
          {/if}
        </span>
        <span class="caption truncate">{entry.name}</span>
      </button>
    {/each}
  </div>
</div>

<style>
  .mosaic {
    height: 100%;
    overflow-y: auto;
    padding: 12px;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(var(--tile), 1fr));
    gap: 7px;
    align-content: start;
  }

  .tile {
    position: relative;
    display: flex;
    flex-direction: column;
    aspect-ratio: 1;
    padding: 0;
    border-radius: 8px;
    overflow: hidden;
    background: var(--bg-2);
    border: 1px solid var(--line-soft);
    transition: transform 120ms ease, border-color 120ms ease, box-shadow 120ms ease;
    content-visibility: auto;
    contain-intrinsic-size: auto 180px;
  }

  .tile:hover {
    transform: translateY(-1px);
    border-color: var(--bg-4);
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.28);
  }

  .tile.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgba(94, 234, 212, 0.35);
  }

  .media {
    position: relative;
    flex: 1;
    min-height: 0;
    overflow: hidden;
    background-color: var(--bg-2);
  }

  .media img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .fallback {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 7px;
    color: var(--text-faint);
    background-color: var(--bg-2);
    background-image:
      radial-gradient(circle at 30% 20%, rgba(94, 234, 212, 0.09), transparent 55%),
      radial-gradient(circle at 75% 80%, rgba(96, 165, 250, 0.1), transparent 55%);
  }

  .ext {
    font-family: var(--mono);
    font-size: 9.5px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-faint);
  }

  .caption {
    flex: none;
    display: block;
    height: 22px;
    padding: 0 8px;
    line-height: 22px;
    background: rgba(6, 10, 14, 0.55);
    color: #eef2f8;
    font-size: 10.5px;
    text-align: left;
  }

  .tile.selected .caption {
    background: rgba(15, 118, 110, 0.6);
  }
</style>
