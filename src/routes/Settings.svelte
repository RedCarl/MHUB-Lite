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
  import {
    monitor,
    monitorStatusText,
    quitApplication,
    testMonitorNotification,
    updateMonitorSettings,
  } from '../lib/monitor.svelte';
  import { theme, setTheme, type ThemeMode } from '../lib/theme.svelte';
  import { connectionLabel, hexId, timeAgo } from '../lib/format';
  import { copyText } from '../lib/toast.svelte';
  import { check } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { pendingUpdate, setPendingUpdate, updateState } from '../lib/update.svelte';
  import Segmented from '../components/Segmented.svelte';
  import Toggle from '../components/Toggle.svelte';

  const mouse = $derived(device.mouse);
  const kb = $derived(device.keyboard);
  const active = $derived(device.active);
  const cfg = $derived(mouse?.config);

  let updBusy = $state(false);
  let updMsg = $state('');
  let updErr = $state('');
  let progress = $state(0);
  const pending = $derived(updateState.version ? pendingUpdate() : null);

  async function checkUpdate() {
    updBusy = true;
    updErr = '';
    updMsg = '';
    try {
      if (!('__TAURI_INTERNALS__' in window)) {
        updMsg = '浏览器预览不支持更新，请使用桌面版。';
        return;
      }
      const found = await check();
      setPendingUpdate(found);
      updMsg = found ? `发现新版本 ${found.version}` : '已经是最新版本';
    } catch (e) {
      updErr = `检查失败：${e}`;
    } finally {
      updBusy = false;
    }
  }

  async function installUpdate() {
    const update = pending;
    if (!update) return;
    updBusy = true;
    updErr = '';
    progress = 0;
    let total = 0;
    let got = 0;
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          total = event.data.contentLength ?? 0;
        } else if (event.event === 'Progress') {
          got += event.data.chunkLength;
          progress = total > 0 ? Math.round((got / total) * 100) : 0;
        } else if (event.event === 'Finished') {
          progress = 100;
        }
      });
      updMsg = '安装完成，正在重启…';
      await relaunch();
    } catch (e) {
      updErr = `更新失败：${e}`;
      updBusy = false;
    }
  }

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
  const thresholdOptions = [
    { value: 10, label: '10%' },
    { value: 20, label: '20%' },
    { value: 30, label: '30%' },
  ];
  const monitorIntervalOptions = [
    { value: 30, label: '30 秒' },
    { value: 60, label: '1 分钟' },
    { value: 300, label: '5 分钟' },
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
      <p class="page-sub">外观、设备、后台提醒和诊断信息。</p>
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
    <div class="st-section-title">后台与电量</div>
    <div class="st-card">
      <div class="st-row">
        <div class="st-label">
          后台常驻
          <span class="st-desc">关闭窗口时隐藏到托盘 / 菜单栏，电量监测继续运行；需要真正退出时用托盘菜单或下面的退出按钮</span>
        </div>
        <Toggle
          checked={monitor.settings.closeToTray}
          onchange={(v) => void updateMonitorSettings({ closeToTray: v })}
          label="后台常驻"
        />
      </div>
      <div class="st-row">
        <div class="st-label">
          开机自动启动
          <span class="st-desc">登录系统后自动在后台启动；启动后窗口默认隐藏</span>
        </div>
        <Toggle
          checked={monitor.autostart}
          onchange={(v) => void updateMonitorSettings({ startAtLogin: v })}
          label="开机自动启动"
        />
      </div>
      <div class="st-row">
        <div class="st-label">
          电量监测
          <span class="st-desc">后台定期读取鼠标和键盘的电量，窗口隐藏后仍然生效</span>
        </div>
        <Toggle
          checked={monitor.settings.enabled}
          onchange={(v) => void updateMonitorSettings({ enabled: v })}
          label="电量监测"
        />
      </div>
      <div class="st-row">
        <div class="st-label">
          低电量提醒
          <span class="st-desc">电量低于阈值时发送系统通知；同一轮放电只提醒一次，充电后重新计数</span>
        </div>
        <Toggle
          checked={monitor.settings.notifyLow}
          onchange={(v) => void updateMonitorSettings({ notifyLow: v })}
          label="低电量提醒"
        />
      </div>
      <div class="st-row">
        <div class="st-label">
          提醒阈值
          <span class="st-desc">电量低于等于这个值时提醒；充电时不会打扰</span>
        </div>
        <Segmented
          value={monitor.settings.lowThreshold}
          options={thresholdOptions}
          onchange={(v) => void updateMonitorSettings({ lowThreshold: v })}
          label="提醒阈值"
        />
      </div>
      <div class="st-row">
        <div class="st-label">
          检查间隔
          <span class="st-desc">后台读取电量的频率；读取只在设备在线时产生一次短暂 HID 访问</span>
        </div>
        <Segmented
          value={monitor.settings.intervalSecs}
          options={monitorIntervalOptions}
          onchange={(v) => void updateMonitorSettings({ intervalSecs: v })}
          label="检查间隔"
        />
      </div>
      <div class="st-row">
        <div class="st-label">
          当前状态
          <span class="st-desc">
            {monitor.status.readAt ? `上次读取于 ${timeAgo(monitor.status.readAt)}` : '还没有读取过电量'}
          </span>
        </div>
        <div class="st-control">
          <span class="st-value">{monitorStatusText()}</span>
          <button class="st-btn" onclick={() => void testMonitorNotification()}>
            发送测试提醒
          </button>
        </div>
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
        <div class="st-label">当前版本</div>
        <span class="st-value">v{device.version || '—'}</span>
        {#if pending}<span class="st-value new">可更新到 v{pending.version}</span>{/if}
      </div>
      <div class="st-row">
        <div class="st-label">
          软件更新
          {#if pending}
            <span class="st-desc">发现新版本 v{pending.version}，安装后会自动重启</span>
          {:else if updMsg}
            <span class="st-desc">{updMsg}</span>
          {/if}
        </div>
        <div class="st-control">
          {#if pending}
            <button class="st-btn primary" onclick={installUpdate} disabled={updBusy}>下载并安装</button>
          {/if}
          <button class="st-btn" onclick={checkUpdate} disabled={updBusy}>
            {updBusy && !pending ? '检查中…' : '检查更新'}
          </button>
        </div>
      </div>
      {#if updBusy && pending}
        <div class="update-row">
          <div class="update-bar"><i style="width: {progress}%"></i></div>
          <span class="st-value">{progress}%</span>
        </div>
      {/if}
      {#if updErr}
        <div class="update-error">{updErr}</div>
      {/if}
      <div class="st-row">
        <div class="st-label">支持的设备</div>
        <span class="st-value">G3 V2 · A7 V3 Ultra+ · K99 V2（2.4G）</span>
      </div>
      <div class="st-row">
        <div class="st-label">
          退出应用
          <span class="st-desc">退出后不再读取电量，也不会再发提醒</span>
        </div>
        <button class="st-btn danger" onclick={() => void quitApplication()}>退出 MHUB Lite</button>
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
