<script lang="ts">
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { icons, ICON_SIZE } from '../lib/icons';
  import {
    device,
    disconnect,
    scan,
    setShowUnknown,
    diagnosticsText,
    switchTo,
    saveLabel,
  } from '../lib/device.svelte';
  import { prefs, persistPrefs } from '../lib/prefs.svelte';
  import { theme, setTheme, type ThemeMode } from '../lib/theme.svelte';
  import { connectionLabel, hexId, timeAgo } from '../lib/format';
  import { copyText } from '../lib/toast.svelte';
  import Segmented from '../components/Segmented.svelte';
  import Toggle from '../components/Toggle.svelte';

  const mouse = $derived(device.mouse);
  const kb = $derived(device.keyboard);
  const active = $derived(device.active);
  const cfg = $derived(mouse?.config);

  const themeOptions: { value: ThemeMode; label: string }[] = [
    { value: 'auto', label: '跟随系统' },
    { value: 'light', label: '浅色' },
    { value: 'dark', label: '深色' },
  ];
  const pollOptions = [
    { value: 2, label: '2 秒' },
    { value: 5, label: '5 秒' },
    { value: 10, label: '10 秒' },
    { value: 0, label: '关闭' },
  ];

  function setPoll(v: number) {
    prefs.pollSeconds = v;
    persistPrefs();
  }

  function rawConfig(): string {
    if (!cfg) return '—';
    return JSON.stringify(cfg, null, 2);
  }
</script>

<div class="page settings-page">
  <div class="page-head">
    <div class="page-title">
      <h1>设置</h1>
      <p class="page-sub">外观、设备连接和诊断信息。</p>
    </div>
    <div class="page-actions">
      {#if saveLabel()}<span class="save-chip" class:busy={device.saveState === 'saving'}>{saveLabel()}</span>{/if}
    </div>
  </div>

  <section class="st-section">
    <div class="st-section-title">外观</div>
    <div class="st-card">
      <div class="st-row">
        <div class="st-label">
          主题
          <span class="st-desc">当前解析为{theme.resolved === 'dark' ? '深色' : '浅色'}；跟随系统时会随系统外观切换</span>
        </div>
        <Segmented value={theme.mode} options={themeOptions} onchange={(v) => setTheme(v)} label="主题" />
      </div>
    </div>
  </section>

  <section class="st-section">
    <div class="st-section-title">设备</div>
    <div class="st-card">
      <div class="st-row">
        <div class="st-label">
          自动刷新
          <span class="st-desc">定期读取当前设备的在线状态与电量</span>
        </div>
        <Segmented value={prefs.pollSeconds} options={pollOptions} onchange={setPoll} label="自动刷新" />
      </div>
      <div class="st-row">
        <div class="st-label">
          显示未验证型号
          <span class="st-desc">列出其他 MCHOSE 设备，但不会对未经验证的协议执行写入</span>
        </div>
        <Toggle
          checked={prefs.showUnknown}
          onchange={(v) => void setShowUnknown(v)}
          label="显示未验证型号"
        />
      </div>
      <div class="st-row">
        <div class="st-label">扫描</div>
        <button class="st-btn" onclick={() => void scan()} disabled={device.busy !== null}>
          <HugeiconsIcon
            class={device.busy === 'scanning' ? 'spin' : ''}
            icon={device.busy === 'scanning' ? icons.loading : icons.refresh}
            size={ICON_SIZE.inline}
          />
          重新扫描设备
        </button>
      </div>
    </div>
  </section>

  <section class="st-section">
    <div class="st-section-title">已连接设备</div>
    <div class="st-card">
      {#if mouse}
        <div class="st-row st-device-row" class:active={active === 'mouse'}>
          <div class="st-label">
            <span class="st-device-name">
              <HugeiconsIcon icon={icons.mouse} size={ICON_SIZE.inline} />
              {mouse.info.productName}
              {#if active === 'mouse'}<span class="badge on">当前</span>{/if}
            </span>
            <span class="st-desc">
              {connectionLabel(mouse.info.connection)} · {mouse.battery ? `${mouse.battery.percent}%` : '电量未知'}
              {#if mouse.info.model} · {mouse.info.model}{/if}
            </span>
          </div>
          <div class="st-control">
            <button class="st-btn" onclick={() => switchTo('mouse')} disabled={active === 'mouse'}>
              切换
            </button>
            <button class="st-btn danger" onclick={() => void disconnect('mouse')}>断开</button>
          </div>
        </div>
      {/if}

      {#if kb}
        <div class="st-row st-device-row" class:active={active === 'keyboard'}>
          <div class="st-label">
            <span class="st-device-name">
              <HugeiconsIcon icon={icons.keyboard} size={ICON_SIZE.inline} />
              {kb.info.productName}
              {#if active === 'keyboard'}<span class="badge on">当前</span>{/if}
            </span>
            <span class="st-desc">
              {connectionLabel(kb.info.connection)} · {kb.battery ? `${kb.battery.percent}%` : '电量未知'}
              {#if kb.info.model} · {kb.info.model}{/if}
            </span>
          </div>
          <div class="st-control">
            <button class="st-btn" onclick={() => switchTo('keyboard')} disabled={active === 'keyboard'}>
              切换
            </button>
            <button class="st-btn danger" onclick={() => void disconnect('keyboard')}>断开</button>
          </div>
        </div>
      {/if}

      {#if !mouse && !kb}
        <div class="st-row">
          <div class="st-label">
            没有连接设备
            <span class="st-desc">插好 USB 数据线或接收器后点扫描</span>
          </div>
          <button class="st-btn" onclick={() => void scan()} disabled={device.busy !== null}>扫描设备</button>
        </div>
      {/if}

      <div class="st-row">
        <div class="st-label">
          诊断
          <span class="st-desc">把当前设备、配置和固件信息复制成一段文本</span>
        </div>
        <div class="st-control">
          <button class="st-btn" onclick={() => void copyText(diagnosticsText())}>
            <HugeiconsIcon icon={icons.copy} size={ICON_SIZE.inline} />
            复制诊断信息
          </button>
        </div>
      </div>

      {#if mouse}
        <div class="st-row">
          <div class="st-label">鼠标读取时间</div>
          <span class="st-value">{timeAgo(mouse.readAt)}</span>
        </div>
        {#if cfg}
          <details class="st-details">
            <summary>鼠标原始配置字段</summary>
            <pre>{rawConfig()}</pre>
          </details>
        {/if}
      {/if}
    </div>
  </section>

  <section class="st-section">
    <div class="st-section-title">关于</div>
    <div class="st-card">
      <div class="st-row">
        <div class="st-label">版本</div>
        <span class="st-value">v{device.version || '—'}</span>
      </div>
      <div class="st-row">
        <div class="st-label">支持的设备</div>
        <span class="st-value">G3 V2 · A7 V3 Ultra+ · K99 V2（2.4G）</span>
      </div>
      <div class="st-row st-row-note">
        <p class="st-desc">
          非官方工具，与 MCHOSE 没有隶属关系。修改会立即写入设备的板载存储；
          未验证型号不会被写入。写入前请确认设备型号，使用风险自负。
        </p>
      </div>
    </div>
  </section>
</div>
