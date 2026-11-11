<script lang="ts" generics="T extends string | number">
  type Option = { value: T; label: string; disabled?: boolean; title?: string };

  let {
    value,
    options,
    onchange,
    label = '',
    compact = false,
  }: {
    value: T;
    options: Option[];
    onchange: (value: T) => void;
    label?: string;
    compact?: boolean;
  } = $props();
</script>

<div class="seg" class:compact role="group" aria-label={label}>
  {#each options as option (option.value)}
    <button
      type="button"
      class:on={value === option.value}
      aria-pressed={value === option.value}
      disabled={option.disabled}
      title={option.title}
      onclick={() => onchange(option.value)}
    >
      {option.label}
    </button>
  {/each}
</div>
