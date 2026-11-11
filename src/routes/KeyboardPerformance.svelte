<script lang="ts">
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { icons, ICON_SIZE } from '../lib/icons';
  import { device, setKeyboardPerformance, saveLabel, canEditKeyboard } from '../lib/device.svelte';
  import Segmented from '../components/Segmented.svelte';
  import Toggle from '../components/Toggle.svelte';

  const kb = $derived(device.keyboard);
  const perf = $derived(kb?.performance);
  const editable = $derived(canEditKeyboard());

  const rateOptions = [
    { value: 250, label: '250 Hz' },
    { value: 500, label: '500 Hz' },
    { value: 1000, label: '1 kHz' },
  ];
  const latencyOptions = [
    { value: 'speed', label: '速度模式' },
    { value: 'standard', label: '标准模式' },
  ];
  const sleepOptions = [0, 1, 2, 3, 5, 10, 15, 30];
</script>

<div class="page">
  <div class="page-head">
    <div class="page-title">
      <h1>性能</h1>
      <p class="page-sub">回报率、延迟、休眠和系统键行为。修改后立即写入键盘。</p>
    </div>
    <div class="page-actions">
      {#if saveLabel()}<span class="save-chip" class:busy={device.saveState === 'saving'}>{saveLabel()}</span>{/if}
    </div>
  </div>

  {#if !kb || !perf}
    <div class="alert info">
      <HugeiconsIcon icon={icons.keyboard} size={ICON_SIZE.control} />
      <div class="alert-body">
        <strong>没有连接键盘</strong>
        <span>请先连接 2.4G 接收器。</span>
      </div>
    </div>
  {:else}
    <div class="perf-grid keyboard-perf">
      <section class="card pad perf-wide">
        <div class="card-head">
          <div class="head-with-icon">
            <span class="section-ico"><HugeiconsIcon icon={icons.performance} size={ICON_SIZE.control} /></span>
            <div>
              <h3>回报率</h3>
              <p class="card-sub">每秒向电脑报告多少次按键状态</p>
            </div>
          </div>
          <span class="current-pill">{perf.rateHz} Hz</span>
        </div>
        <Segmented
          value={perf.rateHz}
          options={rateOptions}
          label="回报率"
          onchange={(v) => setKeyboardPerformance({ rateHz: Number(v) })}
        />
        <p class="hint-text">K99 V2 支持 250 / 500 / 1000 Hz；有线直连时同样以这个值工作。</p>
      </section>

      <section class="card pad">
        <div class="card-head">
          <div class="head-with-icon">
            <span class="section-ico"><HugeiconsIcon icon={icons.debounce} size={ICON_SIZE.control} /></span>
            <div>
              <h3>按键延迟</h3>
              <p class="card-sub">速度模式优先响应，标准模式更稳</p>
            </div>
          </div>
        </div>
        <Segmented
          value={perf.latency}
          options={latencyOptions}
          label="按键延迟"
          onchange={(v) => setKeyboardPerformance({ latency: String(v) })}
        />
      </section>

      <section class="card pad">
        <div class="card-head">
          <div class="head-with-icon">
            <span class="section-ico"><HugeiconsIcon icon={icons.sleep} size={ICON_SIZE.control} /></span>
            <div>
              <h3>无线休眠</h3>
              <p class="card-sub">静止多久后进入省电</p>
            </div>
          </div>
          <span class="current-pill">{perf.sleepMinutes ? `${perf.sleepMinutes} 分钟` : '永不'}</span>
        </div>
        <div class="chips">
          {#each sleepOptions as m (m)}
            <button
              class="chip"
              class:on={perf.sleepMinutes === m}
              disabled={!editable}
              onclick={() => setKeyboardPerformance({ sleepMinutes: m })}
            >
              {m === 0 ? '永不' : `${m} 分钟`}
            </button>
          {/each}
        </div>
      </section>

      <section class="card pad">
        <div class="card-head">
          <div class="head-with-icon">
            <span class="section-ico"><HugeiconsIcon icon={icons.keyboard} size={ICON_SIZE.control} /></span>
            <div>
              <h3>系统键</h3>
              <p class="card-sub">Win 键锁定与 Mac 模式</p>
            </div>
          </div>
        </div>
        <div class="setting-row">
          <div class="setting-label">
            <span>锁定 Win 键</span>
            <span class="setting-desc">防止游戏中误触回到桌面</span>
          </div>
          <Toggle
            checked={perf.winKeyLocked}
            disabled={!editable}
            label="锁定 Win 键"
            onchange={(v) => setKeyboardPerformance({ winKeyLocked: v })}
          />
        </div>
        <div class="setting-row">
          <div class="setting-label">
            <span>Mac 模式</span>
            <span class="setting-desc">交换 Command 与 Option 的位置</span>
          </div>
          <Toggle
            checked={perf.macMode}
            disabled={!editable}
            label="Mac 模式"
            onchange={(v) => setKeyboardPerformance({ macMode: v })}
          />
        </div>
      </section>
    </div>
  {/if}
</div>
