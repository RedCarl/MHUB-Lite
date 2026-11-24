<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import Logo from './components/Logo.svelte';
  import DeviceBattery from './components/DeviceBattery.svelte';
  import { api, type MonitorStatus } from './lib/api';

  let status = $state<MonitorStatus | null>(null);
  let threshold = $state(20);
  let notifyLow = $state(true);
  let now = $state(Date.now());

  const updated = $derived(updatedText(status?.readAt ?? 0));

  function updatedText(seconds: number) {
    if (!seconds) return '等待读取';
    const diff = Math.max(0, Math.floor(now / 1000) - seconds);
    if (diff < 10) return '刚刚更新';
    if (diff < 60) return diff + ' 秒前更新';
    const minutes = Math.floor(diff / 60);
    if (minutes < 60) return minutes + ' 分钟前更新';
    return Math.floor(minutes / 60) + ' 小时前更新';
  }

  async function refresh() {
    try {
      status = await api.monitorStatus();
    } catch {
      // 面板不展示错误，保留上一次读数即可。
    }
  }

  onMount(() => {
    void refresh();
    void api
      .monitorConfig()
      .then((config) => {
        threshold = config.settings.lowThreshold;
        notifyLow = config.settings.notifyLow;
      })
      .catch(() => {});

    const subscriptions: Promise<UnlistenFn>[] = [
      listen<MonitorStatus>('monitor-reading', (event) => {
        status = event.payload;
        now = Date.now();
      }),
      listen('panel-shown', () => {
        now = Date.now();
        void refresh();
      }),
    ];

    const tick = setInterval(() => (now = Date.now()), 10_000);
    const poll = setInterval(() => void refresh(), 30_000);

    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') void invoke('hide_status_panel');
    };
    window.addEventListener('keydown', onKey);

    return () => {
      subscriptions.forEach((pending) => void pending.then((off) => off()));
      clearInterval(tick);
      clearInterval(poll);
      window.removeEventListener('keydown', onKey);
    };
  });
</script>

<div class="dp-panel">
  <div class="dp-head">
    <span class="dp-mark"><Logo size={17} /></span>
    <span class="dp-title">MHUB Lite</span>
    <span class="dp-updated">{updated}</span>
  </div>

  <div class="dp-devices">
    <DeviceBattery kind="mouse" reading={status?.mouse ?? null} {threshold} />
    <DeviceBattery kind="keyboard" reading={status?.keyboard ?? null} {threshold} />
  </div>

  <div class="dp-foot">
    <span class="dp-note">{notifyLow ? '低于 ' + threshold + '% 时提醒' : '低电量提醒已关闭'}</span>
    <button class="dp-quit" onclick={() => void invoke('quit_app')}>退出</button>
  </div>
</div>

<style>
  .dp-panel {
    box-sizing: border-box;
    height: 100vh;
    display: flex;
    flex-direction: column;
    gap: 9px;
    padding: 11px 12px;
    border-radius: 16px;
    border: 1px solid var(--line);
    background:
      radial-gradient(120% 90% at 18% 0%, color-mix(in oklch, var(--fg) 7%, transparent), transparent 62%),
      color-mix(in oklch, var(--popover) 95%, transparent);
    backdrop-filter: blur(30px) saturate(170%);
    -webkit-backdrop-filter: blur(30px) saturate(170%);
    box-shadow: inset 0 1px 0 var(--glass-rim);
    color: var(--fg);
    font-family: var(--font-sans);
    overflow: hidden;
    user-select: none;
    -webkit-user-select: none;
  }

  .dp-head {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 0 3px;
    height: 18px;
  }

  .dp-mark {
    display: flex;
    color: var(--fg2);
  }

  .dp-title {
    font-size: 12px;
    font-weight: 600;
    letter-spacing: -0.005em;
    color: var(--fg2);
  }

  .dp-updated {
    margin-left: auto;
    font-size: 10.5px;
    color: var(--fg3);
    font-variant-numeric: tabular-nums;
  }

  .dp-devices {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .dp-foot {
    margin-top: auto;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 3px;
  }

  .dp-note {
    font-size: 10.5px;
    color: var(--fg3);
  }

  .dp-quit {
    border: 0;
    background: transparent;
    padding: 3px 7px;
    border-radius: 6px;
    font: inherit;
    font-size: 11px;
    color: var(--fg3);
    cursor: pointer;
    transition: background 0.15s ease, color 0.15s ease;
  }

  .dp-quit:hover {
    background: var(--bg3);
    color: var(--fg);
  }
</style>
