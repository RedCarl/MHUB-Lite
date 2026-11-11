<script lang="ts">
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { icons, ICON_SIZE } from '../lib/icons';
  import { device, refresh, disconnect, saveLabel } from '../lib/device.svelte';
  import { connectionLabel, formatHz, formatMinutes, timeAgo } from '../lib/format';
  import { go } from '../lib/router.svelte';
  import KeyboardBoard from '../components/KeyboardBoard.svelte';
  import BatteryGauge from '../components/BatteryGauge.svelte';

  const kb = $derived(device.keyboard);
  const perf = $derived(kb?.performance);
  const mode = $derived(kb?.lightModes.find((m) => m.id === perf?.lightMode));

  const stats = $derived.by(() => {
    if (!perf) return [];
    return [
      {
        icon: icons.performance,
        label: '回报率',
        value: perf.rateHz ? formatHz(perf.rateHz) : '—',
        sub: '2.4G 无线模式',
        go: 'performance' as const,
      },
      {
        icon: icons.apply,
        label: '当前灯效',
        value: mode?.name ?? `模式 ${perf.lightMode}`,
        sub: `亮度 ${perf.brightness}% · 速度 ${perf.speed + 1}`,
        go: 'lighting' as const,
      },
      {
        icon: icons.sleep,
        label: '无线休眠',
        value: perf.sleepMinutes ? `${perf.sleepMinutes}` : '永不',
        sub: perf.sleepMinutes ? '分钟' : '关闭休眠',
        go: 'performance' as const,
      },
      {
        icon: icons.keyboard,
        label: '按键层',
        value: '3',
        sub: 'Default / Fn / Fn2',
        go: 'buttons' as const,
      },
    ];
  });
</script>

<div class="page">
  <div class="page-head">
    <div class="page-title">
      <h1>概览</h1>
      <p class="page-sub">
        {kb ? `${kb.info.productName} 的实时状态与板载配置。` : '连接一把 MCHOSE 键盘，直接读写它的配置。'}
      </p>
    </div>
    <div class="page-actions">
      {#if kb}
        {#if saveLabel()}<span class="save-chip" class:busy={device.saveState === 'saving'}>{saveLabel()}</span>{/if}
        <button class="btn" onclick={() => void refresh()} disabled={device.busy !== null}>
          <HugeiconsIcon
            class={device.busy === 'reading' ? 'spin' : ''}
            icon={device.busy === 'reading' ? icons.loading : icons.refresh}
            size={ICON_SIZE.inline}
          />
          重新读取
        </button>
        <button class="btn ghost" onclick={() => void disconnect('keyboard')} disabled={device.busy !== null}>
          断开
        </button>
      {/if}
    </div>
  </div>

  {#if !kb}
    <div class="alert info">
      <HugeiconsIcon icon={icons.keyboard} size={ICON_SIZE.control} />
      <div class="alert-body">
        <strong>没有连接键盘</strong>
        <span>把键盘切到 2.4G 模式，插好接收器，然后回到设置页重新扫描。</span>
      </div>
    </div>
  {:else}
    <section class="device-hero keyboard-hero card">
      <div class="keyboard-figure">
        <KeyboardBoard layers={kb.layers} compact />
        <span class="board-caption">K99 V2 · 99 键</span>
      </div>

      <div class="device-summary">
        <div class="identity">
          <div class="identity-main">
            <h2>{kb.info.productName}</h2>
            <div class="badges">
              <span class="badge">
                <HugeiconsIcon icon={icons.wireless} size={13} />
                {connectionLabel(kb.info.connection)}
              </span>
              <span class="badge" class:ok={kb.online} class:off={!kb.online}>
                <i class="dot" aria-hidden="true"></i>
                {kb.online ? '已连接' : '离线'}
              </span>
              <span class="badge">{kb.info.model ?? 'K99 V2'}</span>
            </div>
          </div>
          <button class="link-btn" onclick={() => go('buttons')}>
            改键
            <HugeiconsIcon icon={icons.forward} size={ICON_SIZE.inline} />
          </button>
        </div>

        <BatteryGauge battery={kb.battery} online={kb.online} />

        <dl class="meta-list">
          <div>
            <dt>连接方式</dt>
            <dd>{connectionLabel(kb.info.connection)} · 2.4G 接收器</dd>
          </div>
          <div>
            <dt>当前灯效</dt>
            <dd>{mode?.name ?? '—'}</dd>
          </div>
          <div>
            <dt>回报率</dt>
            <dd>{perf?.rateHz ? formatHz(perf.rateHz) : '—'}</dd>
          </div>
          <div>
            <dt>休眠</dt>
            <dd>{perf ? formatMinutes(perf.sleepMinutes) : '—'}</dd>
          </div>
        </dl>

        <div class="hero-foot">
          <span class="muted small">读取于 {timeAgo(kb.readAt)}</span>
          <button class="btn small" onclick={() => go('settings')}>设备与诊断</button>
        </div>
      </div>
    </section>

    {#if perf}
      <section class="stats">
        {#each stats as s (s.label)}
          <button class="stat" onclick={() => go(s.go)}>
            <span class="stat-head">
              <span class="stat-icon"><HugeiconsIcon icon={s.icon} size={ICON_SIZE.control} /></span>
              <span class="stat-label">{s.label}</span>
              <HugeiconsIcon icon={icons.forward} size={ICON_SIZE.inline} />
            </span>
            <span class="stat-value">{s.value}</span>
            <span class="stat-sub">{s.sub}</span>
          </button>
        {/each}
      </section>
    {/if}

    <p class="footnote">
      键盘改动会即时写入。每次写入需要约 1.2 秒，连续调滑块时会自动合并成一次写入。
    </p>
  {/if}
</div>
