<script lang="ts">
  import { codeLabel, keyLabel, layoutKeys } from '../lib/keyboard';

  let {
    layers,
    layer = 0,
    activeIndex = -1,
    onselect,
    compact = false,
  }: {
    layers: [number, number, number, number][][];
    layer?: number;
    activeIndex?: number;
    onselect?: (index: number) => void;
    compact?: boolean;
  } = $props();

  const layout = layoutKeys();
  const codes = $derived(layers[layer] ?? []);
</script>

<div
  class="kb-board"
  class:compact
  style="aspect-ratio: {layout.width} / {layout.height}"
>
  {#each layout.items as p (p.key)}
    {#if p.index >= 0}
      <button
        class="kb-key"
        class:sel={activeIndex === p.index}
        style="left: {((p.x / layout.width) * 100).toFixed(3)}%; top: {((p.y / layout.height) * 100).toFixed(3)}%; width: {((p.w / layout.width) * 100).toFixed(3)}%; height: {((p.h / layout.height) * 100).toFixed(3)}%"
        onclick={() => onselect?.(p.index)}
        title={p.label}
        aria-label={p.label}
      >
        <span class="kb-label">{keyLabel(p.label)}</span>
        {#if !compact}
          {@const code = (codes[p.index] ?? [0, 0, 0, 0]) as [number, number, number, number]}
          {@const assigned = codeLabel(code)}
          {#if assigned !== keyLabel(p.label) && assigned !== '默认'}
            <span class="kb-code">{assigned}</span>
          {/if}
        {/if}
      </button>
    {/if}
  {/each}
</div>
