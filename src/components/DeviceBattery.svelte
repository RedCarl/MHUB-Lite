<script lang="ts">
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { icons } from '../lib/icons';
  import type { MonitorReading } from '../lib/api';

  let {
    kind,
    reading,
    threshold = 20,
  }: { kind: 'mouse' | 'keyboard'; reading: MonitorReading | null; threshold?: number } = $props();

  const label = $derived(kind === 'mouse' ? '鼠标' : '键盘');
  const empty = $derived(!reading);
  const asleep = $derived(!!reading && !reading.online);
  const pct = $derived(reading?.battery?.percent ?? null);
  const charging = $derived(!!reading?.battery?.charging);
  const low = $derived(pct !== null && pct <= threshold && !charging);
  const tone = $derived(
    charging ? 'charge' : low ? 'low' : empty || asleep ? 'muted' : 'ok'
  );
  const stateText = $derived(
    empty
      ? '未连接'
      : asleep
        ? '已休眠'
        : pct === null
          ? '电量未知'
          : charging
            ? '充电中'
            : low
              ? '电量偏低'
              : '在线'
  );
  const barWidth = $derived(charging ? 100 : (pct ?? 0));
</script>

<div class="dp-card" data-tone={tone} class:is-muted={empty || asleep}>
  <span class="dp-icon">
    <HugeiconsIcon icon={kind === 'mouse' ? icons.mouse : icons.keyboard} size={26} />
  </span>

  <div class="dp-body">
    <div class="dp-top">
      <span class="dp-kind">{label}</span>
      <span class="dp-name" title={reading?.name ?? ''}>
        {reading?.name ?? (kind === 'mouse' ? '没有检测到鼠标' : '没有检测到键盘')}
      </span>
      <span class="dp-state" data-tone={tone}>{stateText}</span>
    </div>

    <div class="dp-row">
      <span class="dp-pct" data-tone={tone}>
        {pct ?? '—'}<small>%</small>
      </span>
      <span class="dp-bar">
        <i class="dp-fill" style="width: {barWidth}%" data-tone={tone}></i>
      </span>
    </div>
  </div>
</div>

<style>
  .dp-card {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 10px 12px;
    border-radius: 12px;
    border: 1px solid var(--line-soft);
    background: var(--panel);
  }

  .dp-card.is-muted {
    opacity: 0.62;
  }

  .dp-icon {
    display: grid;
    place-items: center;
    width: 46px;
    height: 46px;
    flex: none;
    border-radius: 12px;
    border: 1px solid var(--line-soft);
    background: var(--bg3);
    color: var(--fg2);
  }

  .dp-card[data-tone='ok'] .dp-icon {
    border-color: var(--ok-border);
    background: var(--ok-bg);
    color: var(--success);
  }

  .dp-card[data-tone='charge'] .dp-icon {
    border-color: color-mix(in oklch, var(--brand) 28%, transparent);
    background: color-mix(in oklch, var(--brand) 13%, transparent);
    color: var(--brand);
  }

  .dp-card[data-tone='low'] .dp-icon {
    border-color: var(--warn-border);
    background: var(--warn-bg);
    color: var(--warn);
  }

  .dp-body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .dp-top {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .dp-kind {
    flex: none;
    font-size: 10.5px;
    letter-spacing: 0.06em;
    color: var(--fg3);
  }

  .dp-name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    font-size: 11.5px;
    color: var(--fg2);
  }

  .dp-state {
    flex: none;
    padding: 1px 6px;
    border-radius: 999px;
    background: var(--bg3);
    font-size: 10px;
    color: var(--fg3);
  }

  .dp-state[data-tone='ok'] {
    color: var(--success);
    background: var(--ok-bg);
  }

  .dp-state[data-tone='charge'] {
    color: var(--brand);
    background: color-mix(in oklch, var(--brand) 13%, transparent);
  }

  .dp-state[data-tone='low'] {
    color: var(--warn);
    background: var(--warn-bg);
  }

  .dp-row {
    display: flex;
    align-items: flex-end;
    gap: 10px;
  }

  .dp-pct {
    flex: none;
    font-size: 33px;
    line-height: 1;
    font-weight: 650;
    letter-spacing: -0.035em;
    font-variant-numeric: tabular-nums;
    color: var(--fg);
  }

  .dp-pct small {
    margin-left: 1px;
    font-size: 13px;
    font-weight: 500;
    letter-spacing: 0;
    color: var(--fg3);
  }

  .dp-pct[data-tone='low'] {
    color: var(--warn);
  }

  .dp-pct[data-tone='charge'] {
    color: var(--brand);
  }

  .dp-pct[data-tone='muted'] {
    color: var(--fg3);
  }

  .dp-bar {
    flex: 1;
    height: 6px;
    margin-bottom: 6px;
    border-radius: 999px;
    background: var(--bg3);
    overflow: hidden;
  }

  .dp-fill {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--success);
  }

  .dp-fill[data-tone='charge'] {
    background: var(--brand);
  }

  .dp-fill[data-tone='low'] {
    background: var(--warn);
  }

  .dp-fill[data-tone='muted'] {
    background: var(--fg3);
    opacity: 0.4;
  }
</style>
