<script lang="ts">
  // SPDX-License-Identifier: GPL-3.0-or-later
  import type { Entry, TreeStats } from '../lib/types';
  import { colorFor, squarify, type ColorMode, type Rect } from '../lib/treemap';
  import { formatBytes, formatDate } from '../lib/format';
  import Icon from '../components/Icon.svelte';

  interface Props {
    root: TreeStats | null;
    loading: boolean;
    path: string;
    onnavigate: (path: string) => void;
    onopen: (entry: Entry) => void;
  }

  let { root, loading, path, onnavigate, onopen }: Props = $props();

  const MAX_CELLS = 240;

  let wrap: HTMLDivElement | null = $state(null);
  let canvas: HTMLCanvasElement | null = $state(null);
  let width = $state(0);
  let height = $state(0);
  let mode = $state<ColorMode>('type');
  let stack = $state<TreeStats[]>([]);
  let hover = $state<{ index: number; x: number; y: number } | null>(null);

  interface Cell {
    rect: Rect;
    node: TreeStats | null;
  }

  let current = $derived(stack.length > 0 ? stack[stack.length - 1] : root);

  // Reset drill-down whenever the browsed directory changes.
  $effect(() => {
    void path;
    stack = [];
    hover = null;
  });

  $effect(() => {
    const el = wrap;
    if (!el) return;
    const observer = new ResizeObserver(() => {
      if (!wrap) return;
      width = wrap.clientWidth;
      height = wrap.clientHeight;
    });
    observer.observe(el);
    return () => observer.disconnect();
  });

  let cells = $derived.by((): Cell[] => {
    const node = current;
    if (!node || node.children.length === 0 || width <= 0 || height <= 0) return [];

    const sorted = [...node.children].sort((a, b) => b.totalBytes - a.totalBytes);
    const top = sorted.slice(0, MAX_CELLS);
    const rest = sorted.slice(MAX_CELLS);

    const values = top.map((child) => Math.max(1, child.totalBytes));
    const restBytes = rest.reduce((sum, child) => sum + child.totalBytes, 0);
    if (restBytes > 0) values.push(restBytes);

    const rects = squarify(values, 0, 0, width, height);
    return rects.map((rect, index) => ({ rect, node: top[index] ?? null }));
  });

  let hoverCell = $derived(hover ? cells[hover.index] ?? null : null);

  $effect(() => {
    // Depend on everything draw() reads.
    void cells;
    void mode;
    void hover;
    void width;
    void height;
    draw();
  });

  function draw() {
    if (!canvas) return;
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    canvas.width = Math.max(1, Math.floor(width * dpr));
    canvas.height = Math.max(1, Math.floor(height * dpr));
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, width, height);

    for (let index = 0; index < cells.length; index += 1) {
      const { rect, node } = cells[index];
      if (rect.w < 1 || rect.h < 1) continue;

      if (node) {
        ctx.fillStyle = colorFor(node.node, mode, node.newestMs);
      } else {
        ctx.fillStyle = '#2b3240';
      }
      ctx.fillRect(rect.x, rect.y, rect.w, rect.h);

      if (hover?.index === index) {
        ctx.fillStyle = 'rgba(255, 255, 255, 0.13)';
        ctx.fillRect(rect.x, rect.y, rect.w, rect.h);
      }

      ctx.strokeStyle = 'rgba(6, 10, 14, 0.6)';
      ctx.lineWidth = 1;
      ctx.strokeRect(rect.x + 0.5, rect.y + 0.5, Math.max(0, rect.w - 1), Math.max(0, rect.h - 1));

      if (node && rect.w > 78 && rect.h > 28) {
        const labelH = Math.min(rect.h - 4, 34);
        ctx.fillStyle = 'rgba(6, 10, 14, 0.52)';
        ctx.fillRect(rect.x + 1, rect.y + rect.h - labelH - 1, rect.w - 2, labelH);

        ctx.fillStyle = 'rgba(240, 246, 252, 0.96)';
        ctx.font = '11px ui-sans-serif, system-ui, sans-serif';
        ctx.fillText(fitText(ctx, node.node.name, rect.w - 12), rect.x + 6, rect.y + rect.h - labelH + 13);

        if (labelH > 20) {
          ctx.fillStyle = 'rgba(200, 212, 226, 0.72)';
          ctx.font = '10px ui-monospace, monospace';
          ctx.fillText(formatBytes(node.totalBytes), rect.x + 6, rect.y + rect.h - 7);
        }
      }
    }
  }

  function fitText(ctx: CanvasRenderingContext2D, text: string, maxWidth: number): string {
    if (ctx.measureText(text).width <= maxWidth) return text;
    let low = 0;
    let high = text.length;
    while (low < high) {
      const mid = (low + high + 1) >> 1;
      if (ctx.measureText(`${text.slice(0, mid)}…`).width <= maxWidth) low = mid;
      else high = mid - 1;
    }
    return `${text.slice(0, low)}…`;
  }

  function hitTest(x: number, y: number): number | null {
    for (let index = 0; index < cells.length; index += 1) {
      const { rect } = cells[index];
      if (x >= rect.x && x < rect.x + rect.w && y >= rect.y && y < rect.y + rect.h) {
        return index;
      }
    }
    return null;
  }

  function onMove(event: MouseEvent) {
    if (!canvas) return;
    const box = canvas.getBoundingClientRect();
    const x = event.clientX - box.left;
    const y = event.clientY - box.top;
    const index = hitTest(x, y);
    hover = index === null ? null : { index, x, y };
  }

  function onClick(event: MouseEvent) {
    if (!canvas) return;
    const box = canvas.getBoundingClientRect();
    const index = hitTest(event.clientX - box.left, event.clientY - box.top);
    if (index === null) return;
    const node = cells[index].node;
    if (!node) return;

    if (node.children.length > 0) {
      stack = [...stack, node];
      hover = null;
    } else if (node.node.kind === 'dir') {
      onnavigate(node.node.path);
    } else {
      onopen(node.node);
    }
  }

  function popTo(index: number) {
    stack = index < 0 ? [] : stack.slice(0, index + 1);
    hover = null;
  }
</script>

<div class="treemap">
  <div class="bar">
    <div class="trail">
      <button onclick={() => popTo(-1)} title={root?.node.path ?? path}>
        {root?.node.name ?? path}
      </button>
      {#each stack as node, index (node.node.id)}
        <Icon name="chevron-right" size={11} />
        <button onclick={() => popTo(index)} title={node.node.path}>{node.node.name}</button>
      {/each}
    </div>

    <div class="modes" role="group" aria-label="Color by">
      {#each ['type', 'age', 'size'] as option (option)}
        <button
          class:active={mode === option}
          onclick={() => (mode = option as ColorMode)}
        >
          {option}
        </button>
      {/each}
    </div>
  </div>

  <div class="canvas-wrap" bind:this={wrap}>
    <canvas bind:this={canvas} onmousemove={onMove} onmouseleave={() => (hover = null)} onclick={onClick}></canvas>

    {#if loading}
      <div class="overlay">
        <span class="spinner"></span>
        Indexing this folder…
      </div>
    {:else if !current}
      <div class="overlay">No data for this folder.</div>
    {:else if current.children.length === 0}
      <div class="overlay">
        {#if current.node.kind === 'dir'}
          This folder is empty or only contains files. Open it in list view to inspect.
        {:else}
          This is a file.
        {/if}
      </div>
    {/if}

    {#if hover && hoverCell}
      <div
        class="tooltip"
        style="left:{Math.min(hover.x + 14, width - 240)}px; top:{Math.min(hover.y + 14, height - 96)}px"
      >
        {#if hoverCell.node}
          <div class="tip-name truncate">{hoverCell.node.node.name}</div>
          <div class="tip-row">
            <span>{formatBytes(hoverCell.node.totalBytes)}</span>
            <span>{hoverCell.node.fileCount.toLocaleString()} files</span>
          </div>
          <div class="tip-row muted">
            <span>{formatDate(hoverCell.node.newestMs)}</span>
            <span>{hoverCell.node.children.length > 0 ? 'click to drill in' : 'click to open'}</span>
          </div>
        {:else}
          <div class="tip-name">Other items</div>
          <div class="tip-row muted">Aggregated to keep the view fast</div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .treemap {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .bar {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 34px;
    padding: 0 12px;
    flex: none;
    border-bottom: 1px solid var(--line-soft);
    background: var(--bg);
  }

  .trail {
    display: flex;
    align-items: center;
    gap: 2px;
    min-width: 0;
    overflow: hidden;
  }

  .trail button {
    max-width: 180px;
    padding: 3px 6px;
    border-radius: 5px;
    color: var(--text-dim);
    font-size: 11.5px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .trail button:hover {
    background: var(--bg-3);
    color: var(--text);
  }

  .trail :global(svg) {
    color: var(--text-faint);
    flex: none;
  }

  .modes {
    display: flex;
    gap: 2px;
    margin-left: auto;
    padding: 2px;
    background: var(--bg-2);
    border: 1px solid var(--line-soft);
    border-radius: 6px;
  }

  .modes button {
    padding: 3px 9px;
    border-radius: 4px;
    color: var(--text-faint);
    font-size: 10.5px;
    text-transform: capitalize;
  }

  .modes button:hover {
    color: var(--text-dim);
  }

  .modes button.active {
    background: var(--bg-4);
    color: var(--accent);
  }

  .canvas-wrap {
    position: relative;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  canvas {
    display: block;
    width: 100%;
    height: 100%;
    cursor: pointer;
  }

  .overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 24px;
    text-align: center;
    color: var(--text-faint);
    font-size: 12.5px;
    background: color-mix(in srgb, var(--bg) 72%, transparent);
  }

  .spinner {
    width: 14px;
    height: 14px;
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

  .tooltip {
    position: absolute;
    z-index: 5;
    width: 226px;
    padding: 9px 11px;
    border-radius: 8px;
    background: var(--bg-2);
    border: 1px solid var(--line);
    box-shadow: var(--shadow);
    pointer-events: none;
    font-size: 11.5px;
  }

  .tip-name {
    font-weight: 600;
    margin-bottom: 5px;
  }

  .tip-row {
    display: flex;
    justify-content: space-between;
    gap: 10px;
    color: var(--text-dim);
  }

  .tip-row.muted {
    margin-top: 3px;
    color: var(--text-faint);
    font-size: 10.5px;
  }
</style>
