<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { THEMES } from '../lib/themes';

  interface Props {
    current: string;
    onchange: (id: string) => void;
  }

  let { current, onchange }: Props = $props();

  let open = $state(false);
</script>

<div class="theme-picker">
  <button class="icon-btn" title="Theme" onclick={() => (open = !open)}>
    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="12" r="10" />
      <path d="M12 2a10 10 0 0 0 0 20" fill="currentColor" opacity="0.3" />
    </svg>
  </button>
  {#if open}
    <div class="menu">
      {#each THEMES as theme (theme.id)}
        <button
          class="item"
          class:active={theme.id === current}
          onclick={() => { onchange(theme.id); open = false; }}
        >
          <span class="swatch" style="background:{theme.swatch}"></span>
          <span>{theme.name}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .theme-picker {
    position: relative;
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
  .icon-btn:hover {
    background: var(--bg-3);
    color: var(--text);
  }
  .menu {
    position: absolute;
    top: 36px;
    right: 0;
    z-index: 20;
    min-width: 160px;
    padding: 4px;
    border-radius: var(--radius-sm);
    background: var(--bg-2);
    border: 1px solid var(--line);
    box-shadow: var(--shadow);
  }
  .item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 8px;
    border-radius: 5px;
    font-size: 12px;
    color: var(--text-dim);
    text-align: left;
  }
  .item:hover {
    background: var(--bg-3);
    color: var(--text);
  }
  .item.active {
    color: var(--accent);
    background: var(--bg-3);
  }
  .swatch {
    width: 14px;
    height: 14px;
    border-radius: 4px;
    border: 1px solid var(--line);
    flex: none;
  }
</style>
