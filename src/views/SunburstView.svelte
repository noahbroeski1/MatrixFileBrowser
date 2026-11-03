<script lang="ts">
  // SPDX-License-Identifier: GPL-3.0-or-later
  import type { Entry, TreeStats } from '../lib/types';
  import { colorFor, type ColorMode } from '../lib/treemap';
  import { formatBytes, formatDate } from '../lib/format';
  import EntryIcon from '../components/EntryIcon.svelte';
  import Icon from '../components/Icon.svelte';

  interface Props {
    root: TreeStats | null;
    loading: boolean;
    path: string;
    onnavigate: (path: string) => void;
    onopen: (entry: Entry) => void;
  }

  let { root, loading, path, onnavigate, onopen }: Props = $props();

  let wrap: HTMLDivElement | null = $state(null);
  let canvas: HTMLCanvasElement | null = $state(null);
  let width = $state(0);
  let height = $state(0);
  let mode = $state<ColorMode>('type');
  let zoom = $state<TreeStats | null>(null);
  let hover = $state<{ node: TreeStats; x: number; y: number } | null>(null);
  let hoverNode = $derived(hover?.node ?? null);

  interface Arc {
    node: TreeStats;
    depth: number;
    start: number;
    end: number;
  }

  let current = $derived(zoom ?? root);
  let displayNode = $derived(hover ? hover.node : current);

  // Reset zoom whenever the browsed directory changes.
  $effect(() => {
    void path;
    zoom = null;
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
    width = el.clientWidth;
    height = el.clientHeight;
    return () => observer.disconnect();
  });

  const MAX_DEPTH = 5;

  let arcs = $derived.by((): Arc[] => {
    const node = current;
    if (!node || width <= 0 || height <= 0) return [];
    const out: Arc[] = [];
    const maxDepth = Math.min(MAX_DEPTH, depthOf(node));
    layout(node, 0, -Math.PI / 2, Math.PI * 1.5, maxDepth, out);
    return out;
  });

  function depthOf(node: TreeStats): number {
    if (!node.children.length) return 0;
    return 1 + Math.max(...node.children.map(depthOf));
  }

  function layout(
    node: TreeStats,
    depth: number,
    start: number,
    end: number,
    maxDepth: number,
    out: Arc[],
  ): void {
    out.push({ node, depth, start, end });
    if (depth >= maxDepth || node.children.length === 0) return;

    const total = node.children.reduce((sum, child) => sum + Math.max(1, child.totalBytes), 0);
    let cursor = start;
    const full = end - start;
    // Snap the last child so the ring closes exactly (avoids a radial seam
    // when one child takes ~100% of the angle).
    const spans = node.children.map((child) => (Math.max(1, child.totalBytes) / total) * full);
    const drift = full - spans.reduce((a, b) => a + b, 0);
    if (spans.length > 0) spans[spans.length - 1] += drift;
    for (let i = 0; i < node.children.length; i += 1) {
      const child = node.children[i];
      let span = spans[i];
      if (span > full) span = full;
      if (full - span < 1e-9) span = full;
      const childStart = cursor;
      // Clamp to full circle.
      const cs = childStart;
      const ce = childStart + Math.min(span, full - (childStart - start));
      layout(child, depth + 1, cs, Math.min(ce, start + full), maxDepth, out);
      cursor += span;
      if (cursor > end) cursor = end;
    }
  }

  let maxDepth = $derived(arcs.reduce((m, a) => Math.max(m, a.depth), 0));
  let ringWidth = $derived(Math.min(width, height) / 2 / (maxDepth + 1.2));

  $effect(() => {
    void arcs;
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

    const cx = width / 2;
    const cy = height / 2;

    for (const arc of arcs) {
      if (arc.depth === 0) continue; // center handled by shadow disc below
      const inner = arc.depth * ringWidth;
      const outer = (arc.depth + 1) * ringWidth;
      if (outer <= 0 || arc.end - arc.start < 0.001) continue;

      let s = arc.start;
      let e = arc.end;
      if (e - s > Math.PI * 2 - 1e-9) e = s + Math.PI * 2;
      if (e - s < 0) e = s;
      ctx.beginPath();
      ctx.arc(cx, cy, outer, s, e);
      if (inner > 0) ctx.arc(cx, cy, inner, e, s, true);
      ctx.closePath();

      ctx.fillStyle = colorFor(arc.node.node, mode, arc.node.newestMs);
      ctx.fill();

      if (hover && hover.node === arc.node) {
        ctx.fillStyle = 'rgba(255, 255, 255, 0.16)';
        ctx.fill();
      }

      ctx.strokeStyle = 'rgba(6, 10, 14, 0.55)';
      ctx.lineWidth = 1;
      ctx.stroke();
    }

// Center disc (label rendered as HTML overlay below)
    const rootNode = displayNode ?? current;
    if (rootNode) {
      ctx.fillStyle = 'rgba(6, 10, 14, 0.6)';
      ctx.beginPath();
      ctx.arc(cx, cy, ringWidth, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  function hitTest(px: number, py: number): TreeStats | null {
    if (!current || width <= 0 || height <= 0) return null;
    const cx = width / 2;
    const cy = height / 2;
    const dx = px - cx;
    const dy = py - cy;
    const r = Math.hypot(dx, dy);
    if (r > (maxDepth + 1) * ringWidth) return null;
    const depth = Math.floor(r / ringWidth);
    if (depth > maxDepth) return null;
    let angle = Math.atan2(dy, dx);
    if (angle < -Math.PI / 2) angle += Math.PI * 2;
    // normalize to [start0, start0+2π)
    const offset = -Math.PI / 2;
    let a = angle - offset;
    if (a < 0) a += Math.PI * 2;

    for (const arc of arcs) {
      if (arc.depth !== depth) continue;
      // arc angles are in absolute radians; convert to same offset space
      const s = arc.start - offset;
      const e = arc.end - offset;
      const ss = s < 0 ? s + Math.PI * 2 : s;
      const ee = e < 0 ? e + Math.PI * 2 : e;
      let inRange = false;
      if (ss <= ee) {
        inRange = a >= ss && a < ee;
      } else {
        inRange = a >= ss || a < ee;
      }
      if (inRange) return arc.node;
    }
    // center
    if (r < ringWidth) return current;
    return null;
  }

  function onMove(event: MouseEvent) {
    if (!canvas) return;
    const box = canvas.getBoundingClientRect();
    const node = hitTest(event.clientX - box.left, event.clientY - box.top);
    hover = node ? { node, x: event.clientX - box.left, y: event.clientY - box.top } : null;
  }

  function onClick(event: MouseEvent) {
    if (!canvas) return;
    const box = canvas.getBoundingClientRect();
    const node = hitTest(event.clientX - box.left, event.clientY - box.top);
    if (!node) return;

    if (node === current) {
      // center click: zoom out if we drilled in
      if (zoom) zoomOut();
      return;
    }
    if (node.children.length > 0) {
      zoom = node;
      hover = null;
    } else if (node.node.kind === 'dir') {
      onnavigate(node.node.path);
    } else {
      onopen(node.node);
    }
  }

  function zoomOut() {
    zoom = null;
    hover = null;
  }
</script>

<div class="sunburst">
  <div class="bar">
    <div class="trail">
      {#if zoom}
        <button onclick={zoomOut} title="Back to root">
          {root?.node.name ?? path}
        </button>
        <Icon name="chevron-right" size={11} />
        <span class="current">{zoom.node.name}</span>
      {:else}
        <span class="current">{root?.node.name ?? path}</span>
      {/if}
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
    <canvas
      bind:this={canvas}
      onmousemove={onMove}
      onmouseleave={() => (hover = null)}
      onclick={onClick}
    ></canvas>

    {#if (displayNode ?? current)}
      {@const node = (displayNode ?? current)!}
      <div class="center-label" style="width:{ringWidth * 2}px; height:{ringWidth * 2}px">
        <EntryIcon entry={node.node} size={20} />
        <span class="center-name truncate">{node.node.name}</span>
        <span class="center-size">{formatBytes(node.totalBytes)}</span>
      </div>
    {/if}

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
          This folder has no subfolders to show.
        {:else}
          This is a file.
        {/if}
      </div>
    {/if}

    {#if hover && hoverNode}
      {#key hoverNode.node.id}
      <div
        class="tooltip"
        style="left:{Math.min(hover.x + 14, width - 240)}px; top:{Math.min(hover.y + 14, height - 96)}px"
      >
        <div class="tip-name truncate">{hoverNode.node.name}</div>
        <div class="tip-row">
          <span>{formatBytes(hoverNode.totalBytes)}</span>
          <span>{hoverNode.fileCount.toLocaleString()} files</span>
        </div>
        <div class="tip-row muted">
          <span>{formatDate(hoverNode.newestMs)}</span>
          <span>{hoverNode.children.length > 0 ? 'click to zoom' : 'click to open'}</span>
        </div>
      </div>
      {/key}
    {/if}
  </div>
</div>

<style>
  .sunburst {
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
    gap: 4px;
    min-width: 0;
    overflow: hidden;
    font-size: 11.5px;
  }

  .trail button {
    padding: 3px 6px;
    border-radius: 5px;
    color: var(--text-dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 180px;
  }

  .trail button:hover {
    background: var(--bg-3);
    color: var(--text);
  }

  .trail :global(svg) {
    color: var(--text-faint);
    flex: none;
  }

  .trail .current {
    color: var(--text);
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 180px;
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

  .center-label {
    position: absolute;
    border-radius: 50%;
    justify-content: center;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    max-width: 140px;
    pointer-events: none;
    text-align: center;
    background: transparent;
  }

  .center-label :global(svg) {
    color: var(--accent);
  }

  .center-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--text);
    max-width: 100%;
  }

  .center-size {
    font-size: 10px;
    color: var(--text-faint);
    font-family: var(--mono);
  }
</style>
