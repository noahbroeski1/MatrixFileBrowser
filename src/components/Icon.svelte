<script lang="ts">
  // SPDX-License-Identifier: GPL-3.0-or-later
  let {
    name,
    size = 16,
    stroke = 1.7,
  }: { name: string; size?: number; stroke?: number } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#if name === 'folder'}
    <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
  {:else if name === 'file'}
    <path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z" />
    <path d="M14 3v5h5" />
  {:else if name === 'image'}
    <rect x="3" y="4" width="18" height="16" rx="2" />
    <circle cx="8.5" cy="9.5" r="1.5" />
    <path d="m21 15-4.5-4.5L7 20" />
  {:else if name === 'video'}
    <rect x="3" y="5" width="18" height="14" rx="2" />
    <path d="m10 9 5 3-5 3z" />
  {:else if name === 'music'}
    <path d="M9 18V5l12-2v13" />
    <circle cx="6" cy="18" r="3" />
    <circle cx="18" cy="16" r="3" />
  {:else if name === 'archive'}
    <rect x="3" y="4" width="18" height="16" rx="2" />
    <path d="M3 9h18M10 4v5M14 9v3" />
  {:else if name === 'pdf'}
    <path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z" />
    <path d="M14 3v5h5M8.5 17v-4h1.5a1.5 1.5 0 0 1 0 3H8.5" />
  {:else if name === 'code'}
    <path d="m8 8-4 4 4 4M16 8l4 4-4 4M13 6l-2 12" />
  {:else if name === 'home'}
    <path d="m3 10 9-7 9 7v10a1 1 0 0 1-1 1h-5v-6H9v6H4a1 1 0 0 1-1-1z" />
  {:else if name === 'download'}
    <path d="M12 3v12m0 0 4-4m-4 4-4-4M4 19h16" />
  {:else if name === 'trash'}
    <path d="M4 7h16M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2M6 7l1 13a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1l1-13" />
  {:else if name === 'drive'}
    <rect x="3" y="5" width="18" height="6" rx="1.5" />
    <rect x="3" y="13" width="18" height="6" rx="1.5" />
    <path d="M7 8h.01M7 16h.01" />
  {:else if name === 'chevron-left'}
    <path d="m15 18-6-6 6-6" />
  {:else if name === 'chevron-right'}
    <path d="m9 18 6-6-6-6" />
  {:else if name === 'arrow-up'}
    <path d="M12 20V4m0 0-6 6m6-6 6 6" />
  {:else if name === 'search'}
    <circle cx="11" cy="11" r="7" />
    <path d="m20 20-3.5-3.5" />
  {:else if name === 'list'}
    <path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01" />
  {:else if name === 'grid'}
    <rect x="3" y="3" width="7" height="7" rx="1.5" />
    <rect x="14" y="3" width="7" height="7" rx="1.5" />
    <rect x="3" y="14" width="7" height="7" rx="1.5" />
    <rect x="14" y="14" width="7" height="7" rx="1.5" />
  {:else if name === 'treemap'}
    <rect x="3" y="3" width="18" height="18" rx="2" />
    <path d="M3 12h11M14 3v18M14 9h7M14 15h7" />
  {:else if name === 'eye'}
    <path d="M2 12s3.5-6 10-6 10 6 10 6-3.5 6-10 6-10-6-10-6z" />
    <circle cx="12" cy="12" r="2.5" />
  {:else if name === 'eye-off'}
    <path d="M3 3l18 18M10.6 6.2A10.9 10.9 0 0 1 12 6c6.5 0 10 6 10 6a17 17 0 0 1-3.2 3.8M6.2 6.6A17 17 0 0 0 2 12s3.5 6 10 6a10.7 10.7 0 0 0 4.2-.8" />
  {:else if name === 'refresh'}
    <path d="M21 12a9 9 0 1 1-2.6-6.4M21 4v5h-5" />
  {:else if name === 'sort'}
    <path d="M8 4v16m0 0-3-3m3 3 3-3M16 20V4m0 0-3 3m3-3 3 3" />
  {:else if name === 'x'}
    <path d="M6 6l12 12M18 6 6 18" />
  {:else if name === 'external'}
    <path d="M14 4h6v6M20 4l-8 8M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5" />
  {:else if name === 'clock'}
    <circle cx="12" cy="12" r="9" />
    <path d="M12 7v5l3 2" />
  {:else if name === 'sidebar'}
    <rect x="3" y="4" width="18" height="16" rx="2" />
    <path d="M9 4v16" />
  {:else if name === 'panel-right'}
    <rect x="3" y="4" width="18" height="16" rx="2" />
    <path d="M15 4v16" />
  {:else if name === 'check'}
    <path d="m4 12 5 5L20 6" />
  {:else if name === 'info'}
    <circle cx="12" cy="12" r="9" />
    <path d="M12 11v5M12 8h.01" />
  {:else if name === 'alert'}
    <path d="M12 3 2 20h20z" />
    <path d="M12 9v5M12 17h.01" />
  {:else if name === 'cube'}
    <path d="M12 2 2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
  {:else if name === 'sparkles'}
    <path d="m12 3 1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8z" />
    <path d="M19 16l.8 2.2L22 19l-2.2.8L19 22l-.8-2.2L16 19l2.2-.8z" />
  {:else if name === 'sunburst'}
    <circle cx="12" cy="12" r="9" fill="none" stroke="currentColor" />
    <circle cx="12" cy="12" r="6" fill="none" stroke="currentColor" />
    <circle cx="12" cy="12" r="3" fill="none" stroke="currentColor" />
  {:else}
    <path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z" />
    <path d="M14 3v5h5" />
  {/if}
</svg>
