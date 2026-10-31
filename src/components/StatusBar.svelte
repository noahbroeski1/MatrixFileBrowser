<script lang="ts">
  // SPDX-License-Identifier: GPL-3.0-or-later
  import { formatBytes } from '../lib/format';

  interface Props {
    total: number;
    shown: number;
    selectedCount: number;
    selectedSize: number;
    loading: boolean;
    mock: boolean;
  }

  let { total, shown, selectedCount, selectedSize, loading, mock }: Props = $props();
</script>

<footer class="status">
  <span>
    {shown}{#if total > shown}<span class="muted"> of {total}</span>{/if}
    <span class="muted">items</span>
  </span>

  {#if selectedCount > 0}
    <span class="sep">·</span>
    <span>{selectedCount} selected</span>
    <span class="sep">·</span>
    <span>{formatBytes(selectedSize)}</span>
  {/if}

  <span class="spacer"></span>

  {#if mock}
    <span class="badge">mock data</span>
  {/if}

  {#if loading}
    <span class="pulse"><span class="dot"></span>loading</span>
  {/if}

  <span class="hint">
    <kbd>Ctrl</kbd>+<kbd>L</kbd> location
    <span class="sep">·</span>
    <kbd>Space</kbd> quick look
  </span>
</footer>

<style>
  .status {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 26px;
    padding: 0 12px;
    flex: none;
    border-top: 1px solid var(--line);
    background: var(--bg-1);
    color: var(--text-faint);
    font-size: 10.5px;
  }

  .muted {
    color: var(--text-faint);
  }

  .sep {
    opacity: 0.5;
  }

  .spacer {
    flex: 1;
  }

  .badge {
    padding: 1px 7px;
    border-radius: 999px;
    background: rgba(251, 191, 36, 0.13);
    color: var(--warn);
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    font-size: 9px;
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

  .hint {
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }
</style>
