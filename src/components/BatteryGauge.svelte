<script lang="ts">
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { icons, ICON_SIZE } from '../lib/icons';
  import type { Battery } from '../lib/api';
  import { batteryTone } from '../lib/format';

  let { battery, online }: { battery: Battery | null; online: boolean } = $props();

  const percent = $derived(battery?.percent ?? 0);
  const tone = $derived(battery ? batteryTone(battery.percent, battery.charging) : 'ok');
  const icon = $derived(
    battery?.charging
      ? icons.batteryCharging
      : percent <= 30
        ? icons.batteryLow
        : icons.battery,
  );
</script>

<div class="gauge" class:offline={!online}>
  <div class="gauge-head">
    <span class="gauge-icon">
      <HugeiconsIcon icon={icon} size={ICON_SIZE.control} />
    </span>
    <span class="gauge-value">
      {online && battery ? battery.percent : '—'}<span class="gauge-unit">%</span>
    </span>
    {#if battery?.charging}
      <span class="gauge-chip">充电中</span>
    {/if}
  </div>

  <div
    class="gauge-track"
    role="meter"
    aria-label="设备电量"
    aria-valuemin={0}
    aria-valuemax={100}
    aria-valuenow={online ? percent : 0}
  >
    <span class="gauge-fill {tone}" style="width: {online ? percent : 0}%"></span>
  </div>

  <div class="gauge-foot">
    <span>{online ? '板载电量' : '设备休眠中'}</span>
    <span>
      {online
        ? battery?.charging
          ? '已接通电源'
          : percent <= 10
            ? '建议尽快充电'
            : '状态正常'
        : '移动鼠标唤醒'}
    </span>
  </div>
</div>
