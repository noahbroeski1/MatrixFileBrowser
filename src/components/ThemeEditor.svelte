<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { type Theme, saveCustomThemes, loadCustomThemes, getTheme } from '../lib/themes';
  import { DEFAULT_PALETTE, type SunburstPalette, savePalette, loadPalette } from '../lib/treemap';
  import { isTauri } from '../lib/api';
  import Icon from './Icon.svelte';

  interface Props {
    open: boolean;
    themeId: string;
    onclose: () => void;
    onsaved: (id: string) => void;
  }

  let { open, themeId, onclose, onsaved }: Props = $props();

  // Load current values when opened
  $effect(() => {
    if (!open) return;
    const theme = getTheme(themeId);
    const v = theme.vars;
    name = theme.name;
    bg = v['--bg'] ?? bg;
    bg1 = v['--bg-1'] ?? bg1;
    bg2 = v['--bg-2'] ?? bg2;
    bg3 = v['--bg-3'] ?? bg3;
    bg4 = v['--bg-4'] ?? bg4;
    line = v['--line'] ?? line;
    lineSoft = v['--line-soft'] ?? lineSoft;
    text = v['--text'] ?? text;
    textDim = v['--text-dim'] ?? textDim;
    textFaint = v['--text-faint'] ?? textFaint;
    accent = v['--accent'] ?? accent;
    accentStrong = v['--accent-strong'] ?? accentStrong;
    accentInk = v['--accent-ink'] ?? accentInk;
    sb = loadPalette();
  });

  // Main theme colors
  let name = $state('My Theme');
  let bg = $state('#0c0f13');
  let bg1 = $state('#11151b');
  let bg2 = $state('#171c24');
  let bg3 = $state('#1e242e');
  let bg4 = $state('#262d39');
  let line = $state('#252c37');
  let lineSoft = $state('#1b212a');
  let text = $state('#e8ecf3');
  let textDim = $state('#9aa4b6');
  let textFaint = $state('#6b7487');
  let accent = $state('#5eead4');
  let accentStrong = $state('#2dd4bf');
  let accentInk = $state('#04201c');

  // Sunburst palette
  let sb = $state<SunburstPalette>({ ...DEFAULT_PALETTE });

  function buildTheme(): Theme {
    const id = 'custom-' + Date.now().toString(36);
    return {
      id,
      name: name || 'Custom Theme',
      swatch: accent,
      vars: {
        'color-scheme': 'dark',
        '--bg': bg,
        '--bg-1': bg1,
        '--bg-2': bg2,
        '--bg-3': bg3,
        '--bg-4': bg4,
        '--line': line,
        '--line-soft': lineSoft,
        '--text': text,
        '--text-dim': textDim,
        '--text-faint': textFaint,
        '--accent': accent,
        '--accent-strong': accentStrong,
        '--accent-ink': accentInk,
        '--blue': '#60a5fa',
        '--danger': '#f87171',
        '--warn': '#fbbf24',
        '--radius': '10px',
        '--radius-sm': '7px',
        '--font': 'ui-sans-serif, system-ui, -apple-system, "Segoe UI", Inter, Cantarell, sans-serif',
        '--mono': 'ui-monospace, "JetBrains Mono", "Cascadia Code", Menlo, Consolas, monospace',
        '--shadow': '0 12px 34px rgba(0, 0, 0, 0.45)',
      },
    };
  }

  async function exportTheme() {
    const theme = buildTheme();
    const json = JSON.stringify({ ...theme, palette: sb }, null, 2);
    const suggestedName = (name || 'theme') + '.json';

    // Tauri native save dialog
    if (isTauri) {
      try {
        const { save } = await import('@tauri-apps/plugin-dialog');
        const { writeTextFile } = await import('@tauri-apps/plugin-fs');
        const path = await save({
          defaultPath: suggestedName,
          filters: [{ name: 'JSON', extensions: ['json'] }],
        });
        if (path) {
          await writeTextFile(path, json);
        }
        return;
      } catch (e) {
        console.error('Tauri save failed', e);
      }
    }

    // Browser File System Access API
    if ('showSaveFilePicker' in window) {
      try {
        const handle = await (window as any).showSaveFilePicker({
          suggestedName,
          types: [{ description: 'JSON', accept: { 'application/json': ['.json'] } }],
        });
        const writable = await handle.createWritable();
        await writable.write(json);
        await writable.close();
        return;
      } catch (e) {
        if ((e as Error).name === 'AbortError') return;
      }
    }

    // Fallback: download
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = suggestedName;
    a.click();
    URL.revokeObjectURL(url);
  }

  function importTheme(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      try {
        const data = JSON.parse(reader.result as string);
        if (data.name) name = data.name;
        const v = data.vars ?? data;
        if (v['--bg']) bg = v['--bg'];
        if (v['--bg-1']) bg1 = v['--bg-1'];
        if (v['--bg-2']) bg2 = v['--bg-2'];
        if (v['--bg-3']) bg3 = v['--bg-3'];
        if (v['--bg-4']) bg4 = v['--bg-4'];
        if (v['--line']) line = v['--line'];
        if (v['--line-soft']) lineSoft = v['--line-soft'];
        if (v['--text']) text = v['--text'];
        if (v['--text-dim']) textDim = v['--text-dim'];
        if (v['--text-faint']) textFaint = v['--text-faint'];
        if (v['--accent']) accent = v['--accent'];
        if (v['--accent-strong']) accentStrong = v['--accent-strong'];
        if (v['--accent-ink']) accentInk = v['--accent-ink'];
        if (data.palette) sb = { ...DEFAULT_PALETTE, ...data.palette };
      } catch (e) {
        alert('Invalid theme file');
      }
    };
    reader.readAsText(file);
    input.value = '';
  }

  function save() {
    const theme = buildTheme();
    const customs = loadCustomThemes();
    customs.push(theme);
    saveCustomThemes(customs);
    savePalette(sb);
    onsaved(theme.id);
    onclose();
  }
</script>

{#if open}
  <div class="backdrop" onclick={onclose} role="presentation"></div>
  <div class="modal" role="dialog" aria-modal="true">
    <header>
      <h2>Create Custom Theme</h2>
      <button class="close" onclick={onclose} title="Close">
        <Icon name="x" size={16} />
      </button>
    </header>

    <div class="body">
      <label class="field">
        <span>Name</span>
        <input bind:value={name} />
      </label>

      <h3>Colors</h3>
      <div class="grid">
        <label class="color-field"><span>Background</span><input type="color" bind:value={bg} /></label>
        <label class="color-field"><span>Bg 1</span><input type="color" bind:value={bg1} /></label>
        <label class="color-field"><span>Bg 2</span><input type="color" bind:value={bg2} /></label>
        <label class="color-field"><span>Bg 3</span><input type="color" bind:value={bg3} /></label>
        <label class="color-field"><span>Bg 4</span><input type="color" bind:value={bg4} /></label>
        <label class="color-field"><span>Line</span><input type="color" bind:value={line} /></label>
        <label class="color-field"><span>Line Soft</span><input type="color" bind:value={lineSoft} /></label>
        <label class="color-field"><span>Text</span><input type="color" bind:value={text} /></label>
        <label class="color-field"><span>Text Dim</span><input type="color" bind:value={textDim} /></label>
        <label class="color-field"><span>Text Faint</span><input type="color" bind:value={textFaint} /></label>
        <label class="color-field"><span>Accent</span><input type="color" bind:value={accent} /></label>
        <label class="color-field"><span>Accent Strong</span><input type="color" bind:value={accentStrong} /></label>
        <label class="color-field"><span>Accent Ink</span><input type="color" bind:value={accentInk} /></label>
      </div>

      <h3>Sunburst Colors</h3>
      <div class="sb-grid">
        <label><span>Dir Hue Start</span><input type="number" bind:value={sb.dirHueStart} min="0" max="360" /></label>
        <label><span>Dir Hue End</span><input type="number" bind:value={sb.dirHueEnd} min="0" max="360" /></label>
        <label><span>Dir Saturation</span><input type="number" bind:value={sb.dirSat} min="0" max="100" /></label>
        <label><span>Dir Lightness</span><input type="number" bind:value={sb.dirLight} min="0" max="100" /></label>
        <label><span>File Saturation</span><input type="number" bind:value={sb.fileSat} min="0" max="100" /></label>
        <label><span>File Lightness</span><input type="number" bind:value={sb.fileLight} min="0" max="100" /></label>
        <label><span>Age Hue Start</span><input type="number" bind:value={sb.ageHueStart} min="0" max="360" /></label>
        <label><span>Age Hue End</span><input type="number" bind:value={sb.ageHueEnd} min="0" max="360" /></label>
        <label><span>Size Hue Start</span><input type="number" bind:value={sb.sizeHueStart} min="0" max="360" /></label>
        <label><span>Size Hue End</span><input type="number" bind:value={sb.sizeHueEnd} min="0" max="360" /></label>
      </div>
    </div>

    <footer>
      <button class="btn secondary" onclick={onclose}>Cancel</button>
      <button class="btn secondary" onclick={exportTheme}>Export</button>
      <label class="btn secondary import-label">
        Import
        <input type="file" accept=".json" onchange={importTheme} hidden />
      </label>
      <button class="btn primary" onclick={save}>Save Theme</button>
    </footer>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    z-index: 200;
  }
  .modal {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 201;
    width: 520px;
    max-height: 80vh;
    overflow-y: auto;
    background: var(--bg-1);
    border: 1px solid var(--line);
    border-radius: var(--radius);
    box-shadow: var(--shadow);
    padding: 0;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 18px;
    border-bottom: 1px solid var(--line-soft);
  }
  header h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 650;
  }
  .close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 6px;
    color: var(--text-dim);
  }
  .close:hover { background: var(--bg-3); color: var(--text); }
  .body { padding: 16px 18px; }
  .body h3 {
    margin: 16px 0 8px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-faint);
  }
  .body h3:first-of-type { margin-top: 0; }
  .field {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 12px;
  }
  .field span { font-size: 12px; color: var(--text-dim); width: 60px; }
  .field input {
    flex: 1;
    height: 30px;
    padding: 0 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--line-soft);
    background: var(--bg-2);
    color: var(--text);
    font-size: 12.5px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
  }
  .color-field {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--text-dim);
  }
  .color-field span { flex: 1; }
  .color-field input[type="color"] {
    width: 32px;
    height: 24px;
    border: 1px solid var(--line);
    border-radius: 4px;
    background: var(--bg-2);
    cursor: pointer;
    padding: 0;
  }
  .sb-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }
  .sb-grid label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    color: var(--text-dim);
  }
  .sb-grid label span { flex: 1; }
  .sb-grid input[type="number"] {
    width: 60px;
    height: 26px;
    padding: 0 6px;
    border-radius: 4px;
    border: 1px solid var(--line-soft);
    background: var(--bg-2);
    color: var(--text);
    font-size: 11.5px;
  }
  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 18px;
    border-top: 1px solid var(--line-soft);
  }
  .btn {
    height: 30px;
    padding: 0 14px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    border: 1px solid var(--line);
  }
  .btn.secondary { color: var(--text-dim); }
  .btn.secondary:hover { background: var(--bg-3); color: var(--text); }
  .btn.primary { background: var(--accent); color: var(--accent-ink); border-color: var(--accent); }
  .import-label {
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 30px;
    padding: 0 14px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    border: 1px solid var(--line);
    color: var(--text-dim);
  }
  .import-label:hover { background: var(--bg-3); color: var(--text); }
</style>
