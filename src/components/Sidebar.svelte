<script lang="ts">
  // SPDX-License-Identifier: GPL-3.0-or-later
  import type { Place, PlaceKind } from '../lib/types';
  import Icon from './Icon.svelte';

  interface Props {
    places: Place[];
    currentPath: string;
    mock: boolean;
    onnavigate: (path: string) => void;
  }

  let { places, currentPath, mock, onnavigate }: Props = $props();

  let primary = $derived(places.filter((place) => place.kind === 'home' || place.kind === 'xdg'));
  let devices = $derived(places.filter((place) => place.kind === 'mount' || place.kind === 'root'));
  let trash = $derived(places.filter((place) => place.kind === 'trash'));

  function iconFor(kind: PlaceKind): string {
    switch (kind) {
      case 'home':
        return 'home';
      case 'trash':
        return 'trash';
      case 'mount':
        return 'drive';
      case 'root':
        return 'drive';
      default:
        return 'folder';
    }
  }
</script>

<aside class="sidebar">
  <div class="scroll">
    <section>
      <div class="label">Places</div>
      {#each primary as place (place.id)}
        <button
          class="place"
          class:active={currentPath === place.path}
          title={place.path}
          onclick={() => onnavigate(place.path)}
        >
          <Icon name={iconFor(place.kind)} size={15} />
          <span class="truncate">{place.label}</span>
        </button>
      {/each}
    </section>

    {#if trash.length > 0}
      <section>
        <div class="label">Trash</div>
        {#each trash as place (place.id)}
          <button
            class="place"
            class:active={currentPath === place.path}
            title={place.path}
            onclick={() => onnavigate(place.path)}
          >
            <Icon name={iconFor(place.kind)} size={15} />
            <span class="truncate">{place.label}</span>
          </button>
        {/each}
      </section>
    {/if}

    {#if devices.length > 0}
      <section>
        <div class="label">Devices</div>
        {#each devices as place (place.id)}
          <button
            class="place"
            class:active={currentPath === place.path}
            title={place.path}
            onclick={() => onnavigate(place.path)}
          >
            <Icon name={iconFor(place.kind)} size={15} />
            <span class="truncate">{place.label}</span>
          </button>
        {/each}
      </section>
    {/if}
  </div>

  <div class="foot">
    {#if mock}
      <span class="badge">browser mock</span>
      <span class="note">Run <code>npm run tauri dev</code> for the real filesystem.</span>
    {:else}
      <span class="note"><kbd>Ctrl</kbd>+<kbd>2</kbd> treemap · <kbd>Ctrl</kbd>+<kbd>3</kbd> mosaic</span>
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    width: 212px;
    flex: none;
    background: var(--bg-1);
    border-right: 1px solid var(--line);
  }

  .scroll {
    flex: 1;
    overflow-y: auto;
    padding: 10px 8px;
  }

  section + section {
    margin-top: 16px;
  }

  .label {
    padding: 0 8px 6px;
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--text-faint);
  }

  .place {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    height: 30px;
    padding: 0 9px;
    border-radius: var(--radius-sm);
    color: var(--text-dim);
    font-size: 12.5px;
    text-align: left;
    transition: background 120ms ease, color 120ms ease;
  }

  .place:hover {
    background: var(--bg-2);
    color: var(--text);
  }

  .place.active {
    background: var(--bg-3);
    color: var(--text);
    box-shadow: inset 2px 0 0 var(--accent);
  }

  .place.active :global(svg) {
    color: var(--accent);
  }

  .foot {
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding: 10px 12px;
    border-top: 1px solid var(--line);
  }

  .badge {
    align-self: flex-start;
    padding: 2px 7px;
    border-radius: 999px;
    background: rgba(251, 191, 36, 0.14);
    color: var(--warn);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .note {
    color: var(--text-faint);
    font-size: 10.5px;
    line-height: 1.5;
  }

  code {
    font-family: var(--mono);
    font-size: 10px;
    color: var(--text-dim);
  }
</style>
