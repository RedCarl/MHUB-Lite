<script lang="ts">
  import { device } from '../lib/device.svelte';
  import KeyboardPerformance from './KeyboardPerformance.svelte';
  import MousePerformance from './MousePerformance.svelte';
</script>

{#if device.active === 'keyboard'}
  <KeyboardPerformance />
{:else}
  <MousePerformance />
{/if}
