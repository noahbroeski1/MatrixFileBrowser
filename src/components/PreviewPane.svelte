<script lang="ts">
  // SPDX-License-Identifier: GPL-3.0-or-later
  import type { Entry } from '../lib/types';
  import { backend, assetUrl } from '../lib/api';
  import { formatBytes, formatDate, formatMode, iconFor, isImage, isText, isVideo } from '../lib/format';
  import Icon from '../components/Icon.svelte';

  interface Props {
    entry: Entry | null;
    onopen: (entry: Entry) => void;
    onreveal: (entry: Entry) => void;
  }

  let { entry, onopen, onreveal }: Props = $props();

  let thumb = $state<string | null>(null);
  let text = $state<string | null>(null);
  let busy = $state(false);

  $effect(() => {
    const current = entry;
    thumb = null;
    text = null;
    busy = false;
    if (!current || current.kind === 'dir') return;

    let cancelled = false;
    if (isImage(current.ext) || isVideo(current.ext) || current.ext === 'pdf') {
      busy = true;
      backend
        .thumbnails([current.path], 720)
        .then((refs) => {
          if (!cancelled && refs[0]) thumb = assetUrl(refs[0].thumbPath);
        })
        .catch(() => {})
        .finally(() => {
          if (!cancelled) busy = false;
        });
    } else if (isText(current.ext) && current.size < 512 * 1024) {
      busy = true;
      backend
        .previewText(current.path, 6000)
        .then((value) => {
          if (!cancelled) text = value;
        })
        .catch(() => {})
        .finally(() => {
          if (!cancelled) busy = false;
        });
    }

    return () => {
      cancelled = true;
    };
  });
</script>

<aside class="preview">
  {#if !entry}
    <div class="empty">
      <Icon name="info" size={22} />
      <p>Select a file to preview it.</p>
    </div>
  {:else}
    <div class="hero">
      {#if thumb}
        <img src={thumb} alt={entry.name} />
      {:else}
        <div class="hero-fallback">
          <Icon name={iconFor(entry)} size={42} />
          <span class="ext">{entry.kind === 'dir' ? 'folder' : entry.ext ?? 'file'}</span>
        </div>
      {/if}
      {#if busy}
        <span class="busy">loading…</span>
      {/if}
    </div>

    <div class="body">
      <h2 class="truncate" title={entry.name}>{entry.name}</h2>
      <p class="path mono truncate" title={entry.path}>{entry.path}</p>

      <dl>
        <dt>Kind</dt>
        <dd>{entry.kind === 'dir' ? 'Folder' : entry.mime ?? 'File'}</dd>
        <dt>Size</dt>
        <dd>{entry.kind === 'dir' ? '—' : formatBytes(entry.size)}</dd>
        <dt>Modified</dt>
        <dd>{formatDate(entry.mtimeMs)}</dd>
        <dt>Permissions</dt>
        <dd class="mono">{formatMode(entry.mode)}</dd>
      </dl>

      {#if text}
        <pre class="snippet">{text}</pre>
      {/if}

      <div class="actions">
        <button onclick={() => onopen(entry)}>
          <Icon name="external" size={14} /> Open
        </button>
        <button onclick={() => onreveal(entry)}>
          <Icon name="folder" size={14} /> Reveal
        </button>
      </div>
    </div>
  {/if}
</aside>

<style>
  .preview {
    display: flex;
    flex-direction: column;
    width: 288px;
    flex: none;
    background: var(--bg-1);
    border-left: 1px solid var(--line);
    overflow-y: auto;
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 48px 24px;
    color: var(--text-faint);
    text-align: center;
    font-size: 12px;
  }

  .hero {
    position: relative;
    aspect-ratio: 4 / 3;
    margin: 12px;
    border-radius: 9px;
    overflow: hidden;
    background: var(--bg-2);
    border: 1px solid var(--line-soft);
    flex: none;
  }

  .hero img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .hero-fallback {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    height: 100%;
    color: var(--text-faint);
    background:
      radial-gradient(circle at 30% 20%, rgba(94, 234, 212, 0.1), transparent 55%),
      radial-gradient(circle at 75% 80%, rgba(96, 165, 250, 0.1), transparent 55%);
  }

  .ext {
    font-family: var(--mono);
    font-size: 9.5px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .busy {
    position: absolute;
    right: 8px;
    bottom: 8px;
    padding: 2px 7px;
    border-radius: 999px;
    background: rgba(6, 10, 14, 0.7);
    color: var(--text-dim);
    font-size: 9.5px;
  }

  .body {
    padding: 0 14px 16px;
    min-width: 0;
  }

  h2 {
    margin: 0 0 3px;
    font-size: 14px;
    font-weight: 650;
    letter-spacing: -0.01em;
  }

  .path {
    margin: 0 0 14px;
    font-size: 10.5px;
    color: var(--text-faint);
  }

  dl {
    display: grid;
    grid-template-columns: 86px 1fr;
    gap: 6px 10px;
    margin: 0 0 14px;
    font-size: 11.5px;
  }

  dt {
    color: var(--text-faint);
  }

  dd {
    margin: 0;
    color: var(--text-dim);
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .snippet {
    max-height: 220px;
    margin: 0 0 14px;
    padding: 10px;
    overflow: auto;
    border-radius: 7px;
    background: var(--bg);
    border: 1px solid var(--line-soft);
    color: var(--text-dim);
    font-family: var(--mono);
    font-size: 10.5px;
    line-height: 1.55;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .actions {
    display: flex;
    gap: 8px;
  }

  .actions button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 12px;
    border-radius: 7px;
    border: 1px solid var(--line);
    color: var(--text-dim);
    font-size: 11.5px;
    transition: background 120ms ease, color 120ms ease;
  }

  .actions button:hover {
    background: var(--bg-3);
    color: var(--text);
  }
</style>
