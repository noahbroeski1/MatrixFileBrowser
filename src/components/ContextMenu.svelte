// SPDX-License-Identifier: GPL-3.0-or-later
<script lang="ts">
  import Icon from './Icon.svelte';

  interface Props {
    x: number;
    y: number;
    onopen: () => void;
    onrename: () => void;
    ontrash: () => void;
    onmove: () => void;
    onclose: () => void;
  }

  let { x, y, onopen, onrename, ontrash, onmove, onclose }: Props = $props();

  // Clamp to viewport
  let left = $derived(Math.min(x, (typeof window !== 'undefined' ? window.innerWidth : 800) - 200));
  let top = $derived(Math.min(y, (typeof window !== 'undefined' ? window.innerHeight : 600) - 180));
</script>

<svelte:window onclick={onclose} onkeydown={(e) => { if (e.key === 'Escape') onclose(); }} />

<div class="menu" style="left:{left}px; top:{top}px" role="menu">
  <button role="menuitem" onclick={() => { onopen(); onclose(); }}>
    <Icon name="external" size={14} /> Open
  </button>
  <button role="menuitem" onclick={() => { onrename(); onclose(); }}>
    <Icon name="code" size={14} /> Rename
  </button>
  <button role="menuitem" onclick={() => { onmove(); onclose(); }}>
    <Icon name="folder" size={14} /> Move to…
  </button>
  <div class="sep"></div>
  <button role="menuitem" class="danger" onclick={() => { ontrash(); onclose(); }}>
    <Icon name="trash" size={14} /> Move to trash
  </button>
</div>

<style>
  .menu {
    position: fixed;
    z-index: 60;
    min-width: 170px;
    padding: 4px;
    background: var(--bg-2);
    border: 1px solid var(--line);
    border-radius: 8px;
    box-shadow: var(--shadow);
  }

  .menu button {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    height: 28px;
    padding: 0 10px;
    border-radius: 5px;
    color: var(--text-dim);
    font-size: 12px;
    text-align: left;
  }

  .menu button:hover {
    background: var(--bg-3);
    color: var(--text);
  }

  .menu button.danger:hover {
    color: var(--danger);
    background: rgba(248, 113, 113, 0.1);
  }

  .sep {
    height: 1px;
    margin: 4px 6px;
    background: var(--line-soft);
  }
</style>
