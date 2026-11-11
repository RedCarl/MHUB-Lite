<script lang="ts">
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { icons, ICON_SIZE } from '../lib/icons';
  import { device, draft, disconnect, scan, setDemoMode, setShowUnknown, diagnosticsText } from '../lib/device.svelte';
  import { prefs, persistPrefs } from '../lib/prefs.svelte';
  import { theme, setTheme, type ThemeMode } from '../lib/theme.svelte';
  import { connectionLabel, hexId, timeAgo } from '../lib/format';
  import { copyText } from '../lib/toast.svelte';
  import Segmented from '../components/Segmented.svelte';
  import Toggle from '../components/Toggle.svelte';

  const snap = $derived(device.current);
  const cfg = $derived(draft.config);

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

  async function toggleUnknown(value: boolean) {
    await setShowUnknown(value);
  }

  function rawConfig(): string {
    if (!cfg) return '—';
    return JSON.stringify(
      {
        pollingHz: cfg.pollingHz,
        stageCount: cfg.stageCount,
        activeStage: cfg.activeStage,
        stages: cfg.stages,
        lod: cfg.lod,
        debounceMs: cfg.debounceMs,
        sleepMinutes: cfg.sleepMinutes,
        scrollReversed: cfg.scrollReversed,
        sensorFlags: cfg.sensorFlags,
        highspeedMode: cfg.highspeedMode,
        sensorAngle: cfg.sensorAngle,
      },
      null,
      2,
    );
  }
</script>

<div class="page settings-page">
  <div class="page-head">
    <div class="page-title">
      <h1>设置</h1>
      <p class="page-sub">外观、运行方式和诊断信息。</p>
    </div>
  </div>

  <section class="st-section">
    <div class="st-section-title">外观</div>
    <div class="st-card">
      <div class="st-row">
        <div class="st-label">
          主题
          <span class="st-desc">
            当前解析为{theme.resolved === 'dark' ? '深色' : '浅色'}；跟随系统时会随系统外观切换
          </span>
        </div>
        <Segmented
          value={theme.mode}
          options={themeOptions}
          onchange={(v) => setTheme(v)}
          label="主题"
        />
      </div>
    </div>
  </section>

  <section class="st-section">
    <div class="st-section-title">设备</div>
    <div class="st-card">
      <div class="st-row">
        <div class="st-label">
          自动刷新
          <span class="st-desc">定期读取在线状态与电量；不会覆盖正在编辑的草稿</span>
        </div>
        <Segmented value={prefs.pollSeconds} options={pollOptions} onchange={setPoll} label="自动刷新" />
      </div>

      <div class="st-row">
        <div class="st-label">
          显示未验证型号
          <span class="st-desc">
            列出其他 MCHOSE 设备，但不会对未经验证的协议执行写入
          </span>
        </div>
        <Toggle
          checked={prefs.showUnknown}
          onchange={(v) => void toggleUnknown(v)}
          label="显示未验证型号"
        />
      </div>

      <div class="st-row">
        <div class="st-label">
          演示模式
          <span class="st-desc">
            {device.browserDemo
              ? '浏览器预览固定使用模拟设备；在 Tauri 窗口里可以关掉'
              : '使用内存里的模拟 G3 V2，不访问 USB HID；适合没有设备时体验界面'}
          </span>
        </div>
        <Toggle
          checked={prefs.demoMode || device.browserDemo}
          onchange={(v) => void setDemoMode(v)}
          label="演示模式"
          disabled={device.browserDemo}
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
    <div class="st-section-title">当前设备</div>
    <div class="st-card">
      {#if snap}
        <div class="st-row">
          <div class="st-label">设备</div>
          <span class="st-value">
            {snap.info.productName}
            {#if snap.info.isDemo || device.browserDemo}<span class="badge demo">演示</span>{/if}
          </span>
        </div>
        <div class="st-row">
          <div class="st-label">连接</div>
          <span class="st-value">
            {connectionLabel(snap.info.connection)} · {snap.online ? '在线' : '休眠'}
          </span>
        </div>
        <div class="st-row">
          <div class="st-label">固件</div>
          <span class="st-value">{snap.info.firmware ?? '—'} · {snap.info.buildDate ?? '—'}</span>
        </div>
        <div class="st-row">
          <div class="st-label">硬件 ID</div>
          <span class="st-value mono">{hexId(snap.info.vendorId)}:{hexId(snap.info.productId)}</span>
        </div>
        <div class="st-row">
          <div class="st-label">序列号</div>
          <span class="st-value mono">{snap.info.serial ?? '—'}</span>
        </div>
        <div class="st-row">
          <div class="st-label">读取时间</div>
          <span class="st-value">{timeAgo(snap.readAt)}</span>
        </div>
        <div class="st-row">
          <div class="st-label">
            诊断
            <span class="st-desc">把设备、配置和固件信息复制成一段文本，方便排查问题</span>
          </div>
          <div class="st-control">
            <button class="st-btn" onclick={() => void copyText(diagnosticsText())}>
              <HugeiconsIcon icon={icons.copy} size={ICON_SIZE.inline} />
              复制诊断信息
            </button>
            <button
              class="st-btn danger"
              onclick={() => void disconnect()}
              disabled={device.busy !== null}
            >断开设备</button>
          </div>
        </div>

        {#if cfg}
          <details class="st-details">
            <summary>原始配置字段</summary>
            <pre>{rawConfig()}</pre>
            <p class="st-desc">
              其中 sensorFlags / highspeedMode / sensorAngle 目前没有在界面上开放，
              但会原样读出来、原样写回去。
            </p>
          </details>
        {/if}
      {:else}
        <div class="st-row">
          <div class="st-label">
            没有连接设备
            <span class="st-desc">
              插好 USB 数据线或 2.4G 接收器后点扫描；无线鼠标休眠时先移动一下
            </span>
          </div>
          <button class="st-btn" onclick={() => void scan()} disabled={device.busy !== null}>
            扫描设备
          </button>
        </div>
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
        <div class="st-label">已验证的型号</div>
        <span class="st-value">MCHOSE G3 V2（有线 0x418C / 2.4G 0x4245）</span>
      </div>
      <div class="st-row">
        <div class="st-label">协议</div>
        <span class="st-value">MCHOSE 厂商 HID · Usage Page 0xFF01</span>
      </div>
      <div class="st-row">
        <div class="st-label">许可</div>
        <span class="st-value">MIT</span>
      </div>
      <div class="st-row st-row-note">
        <p class="st-desc">
          非官方工具，与 MCHOSE 没有隶属关系。只通过公开逆向的 HID 协议读写设备，
          不包含官方软件的任何代码。Linux 下需要 hidraw 权限（见 README 的 udev 规则）。
          写入前请确认设备型号；未验证型号不会被写入。
        </p>
      </div>
    </div>
  </section>
</div>
