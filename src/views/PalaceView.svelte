// SPDX-License-Identifier: GPL-3.0-or-later
<script lang="ts">
  import type { Entry, TreeStats } from '../lib/types';
  import { formatBytes } from '../lib/format';
  import Icon from '../components/Icon.svelte';

  interface Props {
    root: TreeStats | null;
    loading: boolean;
    onnavigate: (path: string) => void;
    onopen: (entry: Entry) => void;
  }

  let { root, loading, onnavigate, onopen }: Props = $props();

  let wrap: HTMLDivElement | null = $state(null);
  let canvas: HTMLCanvasElement | null = $state(null);
  let width = $state(0);
  let height = $state(0);

  let yaw = $state(0.6);
  let pitch = $state(0.35);
  let distance = $state(1);
  let hover = $state<number | null>(null);
  let dragging = $state(false);
  let lastX = 0;
  let lastY = 0;

  interface Item {
    node: TreeStats;
    x: number;
    y: number;
    z: number;
    radius: number;
  }

  const MAX_ITEMS = 60;

  let items = $derived.by((): Item[] => {
    const node = root;
    if (!node) return [];
    const children = [...node.children].sort((a, b) => b.totalBytes - a.totalBytes).slice(0, MAX_ITEMS);
    if (children.length === 0) {
      // single item for the root itself
      return [{ node, x: 0, y: 0, z: 0, radius: 18 }];
    }
    const n = children.length;
    const out: Item[] = [];
    // Fibonacci sphere
    const golden = Math.PI * (3 - Math.sqrt(5));
    for (let i = 0; i < n; i += 1) {
      const y = 1 - (i / (n - 1 || 1)) * 2;
      const r = Math.sqrt(1 - y * y);
      const theta = golden * i;
      const x = Math.cos(theta) * r;
      const z = Math.sin(theta) * r;
      const node = children[i];
      const radius = 6 + Math.sqrt(Math.max(1, node.totalBytes)) / 4000;
      out.push({ node, x: x * 2.2, y: y * 2.2, z: z * 2.2, radius: Math.min(26, radius) });
    }
    return out;
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

  $effect(() => {
    void items;
    void yaw;
    void pitch;
    void distance;
    void hover;
    void width;
    void height;
    draw();
  });

  function project(p: { x: number; y: number; z: number }): { x: number; y: number; scale: number } {
    // rotate around Y (yaw) then X (pitch)
    const cy = Math.cos(yaw);
    const sy = Math.sin(yaw);
    const cp = Math.cos(pitch);
    const sp = Math.sin(pitch);

    const x1 = p.x * cy + p.z * sy;
    const z1 = -p.x * sy + p.z * cy;
    const y1 = p.y;

    const y2 = y1 * cp - z1 * sp;
    const z2 = y1 * sp + z1 * cp;
    const x2 = x1;

    const focal = 6;
    const scale = focal / (focal + z2 + distance * 3);
    const cx = width / 2;
    const cy2 = height / 2;
    return { x: cx + x2 * scale * 120, y: cy2 + y2 * scale * 120, scale };
  }

  function draw() {
    if (!canvas) return;
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    canvas.width = Math.max(1, Math.floor(width * dpr));
    canvas.height = Math.max(1, Math.floor(height * dpr));
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, width, height);

    // background gradient
    const grad = ctx.createRadialGradient(width / 2, height / 2, 0, width / 2, height / 2, Math.max(width, height) / 1.2);
    grad.addColorStop(0, 'rgba(20, 30, 45, 0.6)');
    grad.addColorStop(1, 'rgba(6, 10, 14, 1)');
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, width, height);

    // ground grid
    ctx.strokeStyle = 'rgba(94, 234, 212, 0.06)';
    ctx.lineWidth = 1;
    for (let i = -4; i <= 4; i += 1) {
      const a = project({ x: i, y: -1.6, z: -4 });
      const b = project({ x: i, y: -1.6, z: 4 });
      ctx.beginPath();
      ctx.moveTo(a.x, a.y);
      ctx.lineTo(b.x, b.y);
      ctx.stroke();
      const c = project({ x: -4, y: -1.6, z: i });
      const d = project({ x: 4, y: -1.6, z: i });
      ctx.beginPath();
      ctx.moveTo(c.x, c.y);
      ctx.lineTo(d.x, d.y);
      ctx.stroke();
    }

    // sort items by projected z (depth) for painter's algorithm
    const projected = items.map((item, index) => {
      const p = project(item);
      return { item, index, p };
    });
    projected.sort((a, b) => a.p.scale - b.p.scale);

    for (const { item, index, p } of projected) {
      const r = item.radius * p.scale * 1.5;
      if (r < 1) continue;

      // glow
      const glow = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, r * 2.2);
      glow.addColorStop(0, `hsla(${hueFor(item.node.node)}, 60%, 60%, 0.35)`);
      glow.addColorStop(1, 'rgba(0,0,0,0)');
      ctx.fillStyle = glow;
      ctx.beginPath();
      ctx.arc(p.x, p.y, r * 2.2, 0, Math.PI * 2);
      ctx.fill();

      // sphere
      const sphere = ctx.createRadialGradient(p.x - r * 0.3, p.y - r * 0.3, r * 0.1, p.x, p.y, r);
      sphere.addColorStop(0, `hsla(${hueFor(item.node.node)}, 55%, 62%, 0.95)`);
      sphere.addColorStop(1, `hsla(${hueFor(item.node.node)}, 55%, 32%, 0.95)`);
      ctx.fillStyle = sphere;
      ctx.beginPath();
      ctx.arc(p.x, p.y, r, 0, Math.PI * 2);
      ctx.fill();

      ctx.strokeStyle = 'rgba(6, 10, 14, 0.6)';
      ctx.lineWidth = 1;
      ctx.stroke();

      if (hover === index) {
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.8)';
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.arc(p.x, p.y, r + 3, 0, Math.PI * 2);
        ctx.stroke();
      }

      // label for large items
      if (r > 9) {
        ctx.fillStyle = 'rgba(240, 246, 252, 0.9)';
        ctx.font = '10px ui-sans-serif, system-ui, sans-serif';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'top';
        ctx.fillText(fitText(ctx, item.node.node.name, r * 3), p.x, p.y + r + 3);
      }
    }
  }

  function hueFor(node: Entry): number {
    if (node.kind === 'dir') return 168;
    const ext = node.ext ?? 'file';
    const map: Record<string, number> = {
      rs: 24, ts: 210, js: 48, json: 48, md: 200, txt: 200, pdf: 0,
      jpg: 280, jpeg: 280, png: 280, mp4: 330, mkv: 330, mp3: 120, flac: 120,
      zip: 38, rpm: 38,
    };
    return map[ext] ?? 200;
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

  function hitTest(px: number, py: number): number | null {
    // find closest projected item within radius
    let best: { index: number; dist: number } | null = null;
    for (let i = 0; i < items.length; i += 1) {
      const p = project(items[i]);
      const r = items[i].radius * p.scale * 1.5;
      const d = Math.hypot(px - p.x, py - p.y);
      if (d <= Math.max(r, 8) && (!best || d < best.dist)) {
        best = { index: i, dist: d };
      }
    }
    return best ? best.index : null;
  }

  function onDown(event: MouseEvent) {
    dragging = true;
    lastX = event.clientX;
    lastY = event.clientY;
  }

  function onMove(event: MouseEvent) {
    if (dragging) {
      const dx = event.clientX - lastX;
      const dy = event.clientY - lastY;
      yaw += dx * 0.008;
      pitch = Math.max(-1.2, Math.min(1.2, pitch + dy * 0.008));
      lastX = event.clientX;
      lastY = event.clientY;
      return;
    }
    if (!canvas) return;
    const box = canvas.getBoundingClientRect();
    const idx = hitTest(event.clientX - box.left, event.clientY - box.top);
    hover = idx;
  }

  function onUp() {
    dragging = false;
  }

  function onLeave() {
    dragging = false;
    hover = null;
  }

  function onWheel(event: WheelEvent) {
    event.preventDefault();
    distance = Math.max(0.4, Math.min(3, distance + event.deltaY * 0.001));
  }

  function onClick(event: MouseEvent) {
    if (!canvas) return;
    const box = canvas.getBoundingClientRect();
    const idx = hitTest(event.clientX - box.left, event.clientY - box.top);
    if (idx === null) return;
    const item = items[idx];
    if (item.node.node.kind === 'dir') {
      onnavigate(item.node.node.path);
    } else {
      onopen(item.node.node);
    }
  }

  function resetView() {
    yaw = 0.6;
    pitch = 0.35;
    distance = 1;
  }
</script>

<div class="palace">
  <div class="bar">
    <span class="hint">drag to orbit · scroll to zoom · click a sphere to open</span>
    <span class="spacer"></span>
    <button class="chip" onclick={resetView}>Reset view</button>
  </div>

  <div class="canvas-wrap" bind:this={wrap}>
    <canvas
      bind:this={canvas}
      onmousedown={onDown}
      onmousemove={onMove}
      onmouseup={onUp}
      onmouseleave={onLeave}
      onwheel={onWheel}
      onclick={onClick}
    ></canvas>

    {#if loading}
      <div class="overlay">
        <span class="spinner"></span>
        Aggregating sizes…
      </div>
    {:else if items.length === 0}
      <div class="overlay">
        <Icon name="cube" size={24} />
        <p>This folder is empty.</p>
      </div>
    {/if}

    {#if hover !== null && items[hover]}
      {@const item = items[hover]}
      <div class="tooltip" style="left:16px; top:16px">
        <div class="tip-name truncate">{item.node.node.name}</div>
        <div class="tip-row">
          <span>{formatBytes(item.node.totalBytes)}</span>
          <span>{item.node.fileCount.toLocaleString()} files</span>
        </div>
        <div class="tip-row muted">
          <span>{item.node.node.kind === 'dir' ? 'folder' : (item.node.node.ext ?? 'file')}</span>
          <span>{item.node.children.length > 0 ? 'click to enter' : 'click to open'}</span>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .palace {
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

  .hint {
    font-size: 11px;
    color: var(--text-faint);
  }

  .spacer {
    flex: 1;
  }

  .chip {
    height: 26px;
    padding: 0 10px;
    border-radius: 999px;
    border: 1px solid var(--line);
    color: var(--text-dim);
    font-size: 11px;
  }

  .chip:hover {
    background: var(--bg-3);
    color: var(--text);
  }

  .canvas-wrap {
    position: relative;
    flex: 1;
    min-height: 0;
    overflow: hidden;
    cursor: grab;
  }

  .canvas-wrap:active {
    cursor: grabbing;
  }

  canvas {
    display: block;
    width: 100%;
    height: 100%;
  }

  .overlay {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
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
