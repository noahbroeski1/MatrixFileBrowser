<script lang="ts">
  // SPDX-License-Identifier: GPL-3.0-or-later
  import type { Entry } from '../lib/types';
  import { iconFor } from '../lib/format';
  import Icon from './Icon.svelte';

  let { entry, size = 16 }: { entry: Entry; size?: number } = $props();
</script>

<Icon name={iconFor(entry)} {size} />
