<script lang="ts">
  let {
    checked,
    onchange,
    label,
    disabled = false,
  }: {
    checked: boolean;
    onchange: (value: boolean) => void;
    label: string;
    disabled?: boolean;
  } = $props();
</script>

<label class="switch">
  <input
    type="checkbox"
    {checked}
    {disabled}
    aria-label={label}
    onchange={(e) => onchange(e.currentTarget.checked)}
  />
  <span class="switch-track" aria-hidden="true"></span>
</label>
