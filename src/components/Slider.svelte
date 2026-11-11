<script lang="ts">
  let {
    value,
    min,
    max,
    step = 1,
    label,
    disabled = false,
    oninput,
  }: {
    value: number;
    min: number;
    max: number;
    step?: number;
    label: string;
    disabled?: boolean;
    oninput: (value: number) => void;
  } = $props();

  const percent = $derived(max > min ? ((value - min) / (max - min)) * 100 : 0);
</script>

<input
  class="slider"
  type="range"
  {min}
  {max}
  {step}
  {value}
  {disabled}
  aria-label={label}
  style="--p: {percent}%"
  oninput={(e) => oninput(Number(e.currentTarget.value))}
/>
