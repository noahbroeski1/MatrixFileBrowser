<script lang="ts">
  // SPDX-License-Identifier: GPL-3.0-or-later
  import type { Entry } from '../lib/types';
  import { backend, assetUrl } from '../lib/api';
  import { formatBytes, formatDate, iconFor, isImage, isText, isVideo } from '../lib/format';
  import Icon from '../components/Icon.svelte';

  let { entry, onclose }: { entry: Entry; onclose: () => void } = $props();

  let thumb = $state<string | null>(null);
  let text = $state<string | null>(null);
  const AUDIO_EXT = new Set(['mp3', 'wav', 'ogg', 'oga', 'flac', 'm4a', 'aac', 'opus', 'wma']);
  const isAudio = (ext: string | null) => !!ext && AUDIO_EXT.has(ext);

  $effect(() => {
    const current = entry;
    thumb = null;
    text = null;
    let cancelled = false;

    if (isImage(current.ext) || isVideo(current.ext) || current.ext === 'pdf') {
      backend
        .thumbnails([current.path], 1400)
        .then((refs) => {
          if (!cancelled && refs[0]) thumb = assetUrl(refs[0].thumbPath);
        })
        .catch(() => {});
    } else if (isText(current.ext)) {
      backend
        .previewText(current.path, 12_000)
        .then((value) => {
          if (!cancelled) text = value;
        })
        .catch(() => {});
    }

    return () => {
      cancelled = true;
    };
  });
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key === 'Escape') onclose();
  }}
/>

<div class="quicklook" role="dialog" aria-modal="true" aria-label="Quick look">
  <button class="backdrop" onclick={onclose} aria-label="Close quick look"></button>

  <div class="panel">
    <header>
      <div class="titles">
        <span class="title truncate">{entry.name}</span>
        <span class="meta">
          {entry.kind === 'dir' ? 'folder' : formatBytes(entry.size)} · {formatDate(entry.mtimeMs)}
        </span>
      </div>
      <button class="close" onclick={onclose} aria-label="Close">
        <Icon name="x" size={16} />
      </button>
    </header>

    <div class="stage">
      {#if isAudio(entry.ext)}
        <div class="audio-stage">
          <Icon name="music" size={64} />
          <audio src={assetUrl(entry.path)} controls autoplay preload="metadata"></audio>
        </div>
      {:else if isVideo(entry.ext)}
        <video src={assetUrl(entry.path)} controls autoplay preload="metadata" poster={thumb ?? undefined}><track kind="captions" /></video>
      {:else if thumb}
        <img src={thumb} alt={entry.name} />
      {:else if text}
        <pre>{text}</pre>
      {:else}
        <div class="fallback">
          <Icon name={iconFor(entry)} size={56} />
          <span>{entry.kind === 'dir' ? 'folder' : entry.ext ?? 'file'}</span>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .quicklook {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: grid;
    place-items: center;
    padding: 40px;
  }

  .backdrop {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    background: rgba(4, 7, 11, 0.72);
    backdrop-filter: blur(4px);
  }

  .panel {
    position: relative;
    display: flex;
    flex-direction: column;
    width: min(1080px, 92vw);
    height: min(760px, 88vh);
    background: var(--bg-1);
    border: 1px solid var(--line);
    border-radius: 14px;
    box-shadow: var(--shadow);
    overflow: hidden;
  }

  header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--line);
  }

  .titles {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .title {
    font-size: 13.5px;
    font-weight: 600;
  }

  .meta {
    font-size: 10.5px;
    color: var(--text-faint);
  }

  .close {
    margin-left: auto;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border-radius: 7px;
    color: var(--text-dim);
  }

  .close:hover {
    background: var(--bg-3);
    color: var(--text);
  }

  .stage {
    flex: 1;
    min-height: 0;
    display: grid;
    place-items: center;
    padding: 20px;
    background-color: var(--bg);
    background-image: radial-gradient(circle at 50% 0%, rgba(94, 234, 212, 0.05), transparent 60%);
  }

  .stage img,
  .stage video {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: 8px;
  }

  .audio-stage {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 20px;
    width: min(480px, 80%);
    color: var(--text-faint);
  }

  .audio-stage audio {
    width: 100%;
  }

  .stage pre {
    width: 100%;
    height: 100%;
    margin: 0;
    padding: 16px;
    overflow: auto;
    border-radius: 8px;
    background: var(--bg-1);
    border: 1px solid var(--line-soft);
    color: var(--text-dim);
    font-family: var(--mono);
    font-size: 11.5px;
    line-height: 1.6;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .fallback {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    color: var(--text-faint);
    font-family: var(--mono);
    font-size: 10.5px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
</style>
