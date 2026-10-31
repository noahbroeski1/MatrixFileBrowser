<script lang="ts">
  // SPDX-License-Identifier: GPL-3.0-or-later
  import Icon from './Icon.svelte';

  let { path, onnavigate }: { path: string; onnavigate: (path: string) => void } = $props();

  let parts = $derived.by(() => {
    const segments = path.split('/').filter(Boolean);
    const out: { label: string; path: string }[] = [{ label: '/', path: '/' }];
    let acc = '';
    for (const segment of segments) {
      acc += `/${segment}`;
      out.push({ label: segment, path: acc });
    }
    return out;
  });
</script>

<nav class="crumbs" aria-label="Breadcrumb">
  {#each parts as part, index (part.path)}
    {#if index > 0}
      <Icon name="chevron-right" size={12} />
    {/if}
    <button
      class="crumb truncate"
      class:current={index === parts.length - 1}
      title={part.path}
      onclick={() => onnavigate(part.path)}
    >
      {#if index === 0}
        <Icon name="drive" size={13} />
      {:else}
        {part.label}
      {/if}
    </button>
  {/each}
</nav>

<style>
  .crumbs {
    display: flex;
    align-items: center;
    gap: 1px;
    min-width: 0;
    overflow: hidden;
  }

  .crumb {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    max-width: 190px;
    padding: 5px 8px;
    border-radius: var(--radius-sm);
    color: var(--text-dim);
    font-size: 12.5px;
    transition: background 120ms ease, color 120ms ease;
  }

  .crumb:hover {
    background: var(--bg-3);
    color: var(--text);
  }

  .crumb.current {
    color: var(--text);
    font-weight: 600;
  }

  :global(.crumbs > svg) {
    color: var(--text-faint);
    flex: none;
  }
</style>
