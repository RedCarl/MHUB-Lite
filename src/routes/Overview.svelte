<script lang="ts">
  import { device } from '../lib/device.svelte';
  import KeyboardOverview from './KeyboardOverview.svelte';
  import MouseOverview from './MouseOverview.svelte';
</script>

{#if device.active === 'keyboard'}
  <KeyboardOverview />
{:else}
  <MouseOverview />
{/if}
