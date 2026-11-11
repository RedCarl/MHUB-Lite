<script lang="ts">
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { icons, ICON_SIZE } from '../lib/icons';
  import { device, connect, scan, refresh, disconnect, canEdit } from '../lib/device.svelte';
  import {
    activeDpi,
    actionShortLabel,
    BUTTON_LABELS,
    connectionLabel,
    formatHz,
    hexId,
    timeAgo,
  } from '../lib/format';
  import { go } from '../lib/router.svelte';
  import MouseView from '../components/MouseView.svelte';
  import BatteryGauge from '../components/BatteryGauge.svelte';

  const snap = $derived(device.mouse);
  const cfg = $derived(snap?.config);
  const caps = $derived(snap?.info.capabilities);
  const online = $derived(!!snap?.online);
  const busy = $derived(device.busy);
  const mice = $derived(device.devices.filter((d) => d.kind === 'mouse'));
  const supported = $derived(mice.filter((d) => d.supported));
  const unsupported = $derived(mice.filter((d) => !d.supported));

  function connectionIcon(connection: string) {
    if (connection === 'wireless') return icons.wireless;
    if (connection === 'wired') return icons.wired;
    return icons.info;
  }

  const stats = $derived.by(() => {
    if (!cfg) return [];
    return [
      {
        icon: icons.dpi,
        label: '当前 DPI',
        value: `${activeDpi(cfg) ?? '—'}`,
        unit: 'DPI',
        sub: `第 ${cfg.activeStage} 档，共 ${cfg.stageCount} 档`,
        go: 'dpi' as const,
      },
      {
        icon: icons.performance,
        label: '回报率',
        value: formatHz(cfg.pollingHz),
        unit: '',
        sub: '当前连接模式',
        go: 'performance' as const,
      },
      {
        icon: icons.debounce,
        label: '按键防抖',
        value: `${cfg.debounceMs}`,
        unit: 'ms',
        sub: '时间越短触发越快',
        go: 'performance' as const,
      },
      {
        icon: icons.sleep,
        label: '无线休眠',
        value: cfg.sleepMinutes === 0 ? '永不' : `${cfg.sleepMinutes}`,
        unit: cfg.sleepMinutes === 0 ? '' : '分钟',
        sub: '静止后自动省电',
        go: 'performance' as const,
      },
    ];
  });
</script>

<div class="page">
  <div class="page-head">
    <div class="page-title">
      <h1>概览</h1>
      <p class="page-sub">
        {snap ? `${snap.info.productName} 的实时状态与板载配置。` : '连接一台 MCHOSE 鼠标，直接读写它板载的配置。'}
      </p>
    </div>
    <div class="page-actions">
      {#if snap}
        <button class="btn" onclick={() => void refresh()} disabled={busy !== null}>
          <HugeiconsIcon
            class={busy === 'reading' ? 'spin' : ''}
            icon={busy === 'reading' ? icons.loading : icons.refresh}
            size={ICON_SIZE.inline}
          />
          重新读取
        </button>
        <button class="btn ghost" onclick={() => void disconnect('mouse')} disabled={busy !== null}>断开</button>
      {:else}
        <button class="btn" onclick={() => void scan()} disabled={busy !== null}>
          <HugeiconsIcon
            class={busy === 'scanning' ? 'spin' : ''}
            icon={busy === 'scanning' ? icons.loading : icons.refresh}
            size={ICON_SIZE.inline}
          />
          扫描设备
        </button>
      {/if}
    </div>
  </div>

  {#if !snap}
    <section class="empty-device card">
      <div class="empty-figure">
        <MouseView buttons={null} label={false} />
      </div>
      <div class="empty-copy">
        <span class="eyebrow">{device.browserDemo ? '浏览器预览' : '未连接鼠标'}</span>
        <h2>
          {busy === 'scanning'
            ? '正在寻找 MCHOSE 设备…'
            : supported.length > 0
              ? '找到鼠标了，点一下连接'
              : unsupported.length > 0
                ? '找到了设备，但暂不支持这个型号'
                : '没有找到可用的鼠标'}
        </h2>
        <p>
          {device.browserDemo
            ? '浏览器里只显示预览数据；桌面版会直接连接真实 USB HID 设备。'
            : unsupported.length > 0
              ? 'MHUB Lite 只对已验证协议写入。下面的型号能被识别，但暂时只能显示信息。'
              : '插好 USB 数据线或 2.4G 接收器；无线鼠标休眠时先移动一下再扫描。'}
        </p>
        <div class="empty-actions">
          <button class="btn primary" onclick={() => void scan()} disabled={busy !== null}>
            <HugeiconsIcon icon={icons.refresh} size={ICON_SIZE.inline} />
            重新扫描
          </button>
        </div>
        {#if supported.length > 0}
          <div class="pick-list">
            {#each supported as d (d.key)}
              <button class="pick" onclick={() => void connect(d.key)} disabled={busy !== null}>
                <span class="pick-ico"><HugeiconsIcon icon={icons.mouse} size={ICON_SIZE.control} /></span>
                <span class="pick-body">
                  <span class="pick-name">{d.name}</span>
                  <span class="pick-sub">
                    {connectionLabel(d.connection)} · {hexId(d.vendorId)}:{hexId(d.productId)}
                  </span>
                </span>
                <span class="pick-go"><HugeiconsIcon icon={icons.forward} size={ICON_SIZE.inline} /></span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </section>

    {#if unsupported.length > 0}
      <section class="card pad section-gap">
        <div class="card-head">
          <div>
            <h3>识别到但暂不支持的设备</h3>
            <p class="card-sub">MHUB Lite 只对经过验证的协议写入，避免把未知型号写坏。</p>
          </div>
        </div>
        <ul class="device-list">
          {#each unsupported as d (d.key)}
            <li>
              <span class="device-ico"><HugeiconsIcon icon={icons.alert} size={ICON_SIZE.inline} /></span>
              <span class="device-list-body">
                <span class="device-list-name">{d.name}</span>
                <span class="device-list-sub">
                  {hexId(d.vendorId)}:{hexId(d.productId)} · {connectionLabel(d.connection)}
                </span>
                {#if d.statusNote}<span class="device-list-note">{d.statusNote}</span>{/if}
              </span>
            </li>
          {/each}
        </ul>
      </section>
    {/if}
  {:else}
    {#if !online}
      <div class="alert warn section-gap">
        <HugeiconsIcon icon={icons.sleep} size={ICON_SIZE.control} />
        <div class="alert-body">
          <strong>鼠标正在休眠</strong>
          <span>移动鼠标或按任意键唤醒，然后重新读取。</span>
        </div>
        <button class="btn small" onclick={() => void refresh()} disabled={busy !== null}>重新读取</button>
      </div>
    {/if}

    <section class="device-hero card">
      <div class="device-figure">
        <MouseView
          buttons={snap.buttons}
          label={false}
          onselect={() => {
            if (caps?.buttons) go('buttons');
          }}
        />
      </div>

      <div class="device-summary">
        <div class="identity">
          <div class="identity-main">
            <h2>{snap.info.productName}</h2>
            <div class="badges">
              <span class="badge">
                <HugeiconsIcon icon={connectionIcon(snap.info.connection)} size={13} />
                {connectionLabel(snap.info.connection)}
              </span>
              <span class="badge" class:ok={online} class:off={!online}>
                <i class="dot" aria-hidden="true"></i>
                {online ? '已连接' : '休眠中'}
              </span>
              {#if snap.info.model}<span class="badge">{snap.info.model}</span>{/if}
            </div>
          </div>
          <button class="link-btn" onclick={() => go('dpi')} disabled={!canEdit()}>
            调节
            <HugeiconsIcon icon={icons.forward} size={ICON_SIZE.inline} />
          </button>
        </div>

        <BatteryGauge battery={snap.battery} online={online} />

        <dl class="meta-list">
          <div>
            <dt>固件版本</dt>
            <dd>{snap.info.firmware ?? '—'}</dd>
          </div>
          <div>
            <dt>构建日期</dt>
            <dd>{snap.info.buildDate ?? '—'}</dd>
          </div>
          <div>
            <dt>硬件 ID</dt>
            <dd class="mono">{hexId(snap.info.vendorId)}:{hexId(snap.info.productId)}</dd>
          </div>
          <div>
            <dt>序列号</dt>
            <dd class="mono">{snap.info.serial ?? '—'}</dd>
          </div>
        </dl>

        <div class="hero-foot">
          <span class="muted small">读取于 {timeAgo(snap.readAt)}</span>
          <button class="btn small" onclick={() => go('settings')}>设备与诊断</button>
        </div>
      </div>
    </section>

    {#if cfg}
      <section class="stats">
        {#each stats as s (s.label)}
          <button class="stat" onclick={() => go(s.go)}>
            <span class="stat-head">
              <span class="stat-icon"><HugeiconsIcon icon={s.icon} size={ICON_SIZE.control} /></span>
              <span class="stat-label">{s.label}</span>
              <HugeiconsIcon icon={icons.forward} size={ICON_SIZE.inline} />
            </span>
            <span class="stat-value">
              {s.value}{#if s.unit}<span class="stat-unit">{s.unit}</span>{/if}
            </span>
            <span class="stat-sub">{s.sub}</span>
          </button>
        {/each}
      </section>

      <div class="two-col section-gap">
        <section class="card pad">
          <div class="card-head">
            <div>
              <h3>DPI 档位</h3>
              <p class="card-sub">点任意一档去调节具体数值</p>
            </div>
            <button class="btn small" onclick={() => go('dpi')} disabled={!canEdit()}>编辑</button>
          </div>
          <div class="ladder">
            {#each cfg.stages.slice(0, cfg.stageCount) as dpi, i (i)}
              <button
                class="ladder-row"
                class:on={cfg.activeStage === i + 1}
                onclick={() => go('dpi')}
                title={`第 ${i + 1} 档：${dpi} DPI`}
              >
                <span class="ladder-index">{i + 1}</span>
                <span class="ladder-bar">
                  <span
                    class="ladder-fill"
                    style="width: {Math.max(2, (dpi / Math.max(...cfg.stages.slice(0, cfg.stageCount))) * 100)}%"
                  ></span>
                </span>
                <span class="ladder-value">{dpi}</span>
              </button>
            {/each}
          </div>
        </section>

        {#if caps?.buttons && snap.buttons}
          <section class="card pad">
            <div class="card-head">
              <div>
                <h3>按键映射</h3>
                <p class="card-sub">左键由固件锁定，其余四个可改</p>
              </div>
              <button class="btn small" onclick={() => go('buttons')} disabled={!canEdit()}>编辑</button>
            </div>
            <ul class="mini-map-list">
              {#each [1, 2, 3, 4] as index (index)}
                <li>
                  <span class="mini-map-name">{BUTTON_LABELS[index]}</span>
                  <span class="mini-map-action">{actionShortLabel(snap.buttons[index])}</span>
                </li>
              {/each}
              <li>
                <span class="mini-map-name">滚轮</span>
                <span class="mini-map-action">{cfg.scrollReversed ? '方向反转' : '标准方向'}</span>
              </li>
            </ul>
          </section>
        {/if}
      </div>
    {/if}
  {/if}
</div>
