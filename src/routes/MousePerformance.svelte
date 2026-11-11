<script lang="ts">
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { icons, ICON_SIZE } from '../lib/icons';
  import { device, canEdit, restoreFactory, setPolling, setDebounce, setSleep, setLod, saveLabel } from '../lib/device.svelte';
  import { formatHz } from '../lib/format';
  import Segmented from '../components/Segmented.svelte';
  import Slider from '../components/Slider.svelte';

  const snap = $derived(device.mouse);
  const cfg = $derived(snap?.config);
  const caps = $derived(snap?.info.capabilities);
  const editable = $derived(canEdit());

  const rateOptions = $derived(
    (caps?.pollingRates ?? []).map((hz) => ({ value: hz, label: formatHz(hz) })),
  );

  const debouncePresets = [0, 4, 8, 12];
  const sleepPresets = [0, 3, 10, 30];
</script>

<div class="page">
  <div class="page-head">
    <div class="page-title">
      <h1>性能</h1>
      <p class="page-sub">回报率、防抖、休眠和静默高度。修改后立即写入设备。</p>
    </div>
    <div class="page-actions">
      {#if saveLabel()}<span class="save-chip" class:busy={device.saveState === 'saving'}>{saveLabel()}</span>{/if}
      <button class="btn" onclick={restoreFactory} disabled={!editable}>
        <HugeiconsIcon icon={icons.reset} size={ICON_SIZE.inline} />
        恢复默认值
      </button>
    </div>
  </div>

  {#if !snap || !cfg || !caps}
    <div class="alert info">
      <HugeiconsIcon icon={icons.info} size={ICON_SIZE.control} />
      <div class="alert-body">
        <strong>还没有连接鼠标</strong>
        <span>回到概览页连接设备后，才能调节性能参数。</span>
      </div>
    </div>
  {:else}
    <div class="perf-grid">
      <section class="card pad perf-wide">
        <div class="card-head">
          <div class="head-with-icon">
            <span class="section-ico"><HugeiconsIcon icon={icons.performance} size={ICON_SIZE.control} /></span>
            <div>
              <h3>回报率</h3>
              <p class="card-sub">每秒向电脑报告多少次位置，越高越跟手，也越费电</p>
            </div>
          </div>
          <span class="current-pill">{formatHz(cfg.pollingHz)}</span>
        </div>
        <Segmented value={cfg.pollingHz} options={rateOptions} onchange={setPolling} label="回报率" />
        <p class="hint-text">
          {snap.info.model === 'A7 V3 Ultra+'
            ? 'A7 V3 Ultra+ 使用 0x4D 协议，USB 有线支持 125–8000 Hz；修改后立即写入。'
            : snap.info.connection === 'wired'
              ? 'USB 有线模式固定 1000 Hz；这里保存的是拔线后 2.4G 模式使用的值。'
              : '当前用 2.4G 接收器连接，所选值立即作用于无线模式。'}
        </p>
      </section>

      <section class="card pad">
        <div class="card-head">
          <div class="head-with-icon">
            <span class="section-ico"><HugeiconsIcon icon={icons.debounce} size={ICON_SIZE.control} /></span>
            <div>
              <h3>按键防抖</h3>
              <p class="card-sub">过滤机械抖动的等待时间</p>
            </div>
          </div>
          <span class="current-pill">{cfg.debounceMs} ms</span>
        </div>
        <Slider
          value={cfg.debounceMs}
          min={0}
          max={20}
          step={1}
          label="按键防抖"
          disabled={!editable}
          oninput={setDebounce}
        />
        <div class="slider-scale"><span>0 ms</span><span>20 ms</span></div>
        <div class="chips">
          {#each debouncePresets as ms (ms)}
            <button class="chip" class:on={cfg.debounceMs === ms} onclick={() => setDebounce(ms)} disabled={!editable}>
              {ms} ms
            </button>
          {/each}
        </div>
        <p class="hint-text">数值越小点击越直接；出现双击误触时可适当调高。</p>
      </section>

      <section class="card pad">
        <div class="card-head">
          <div class="head-with-icon">
            <span class="section-ico"><HugeiconsIcon icon={icons.sleep} size={ICON_SIZE.control} /></span>
            <div>
              <h3>无线休眠</h3>
              <p class="card-sub">静止多久后进入省电待机</p>
            </div>
          </div>
          <span class="current-pill">{cfg.sleepMinutes === 0 ? '永不休眠' : `${cfg.sleepMinutes} 分钟`}</span>
        </div>
        <Slider
          value={cfg.sleepMinutes}
          min={0}
          max={100}
          step={1}
          label="无线休眠时间"
          disabled={!editable}
          oninput={setSleep}
        />
        <div class="slider-scale"><span>永不休眠</span><span>100 分钟</span></div>
        <div class="chips">
          {#each sleepPresets as m (m)}
            <button class="chip" class:on={cfg.sleepMinutes === m} onclick={() => setSleep(m)} disabled={!editable}>
              {m === 0 ? '永不' : `${m} 分钟`}
            </button>
          {/each}
        </div>
        <p class="hint-text">仅无线模式生效；有线连接不受影响。</p>
      </section>

      {#if caps.lod}
        <section class="card pad">
          <div class="card-head">
            <div class="head-with-icon">
              <span class="section-ico"><HugeiconsIcon icon={icons.dpi} size={ICON_SIZE.control} /></span>
              <div>
                <h3>静默高度（LOD）</h3>
                <p class="card-sub">鼠标抬离桌面多高后停止响应</p>
              </div>
            </div>
            <span class="current-pill">{cfg.lod === 'low' ? '1 mm' : '2 mm'}</span>
          </div>
          <Segmented
            value={cfg.lod}
            options={[
              { value: 'low', label: '1 mm（低）' },
              { value: 'high', label: '2 mm（高）' },
            ]}
            onchange={setLod}
            label="静默高度"
          />
          <p class="hint-text">低静默高度适合抬鼠操作；桌面不平时改成 2 mm 更稳。</p>
        </section>
      {:else}
        <section class="card pad">
          <div class="card-head">
            <div class="head-with-icon">
              <span class="section-ico"><HugeiconsIcon icon={icons.info} size={ICON_SIZE.control} /></span>
              <div>
                <h3>传感器</h3>
                <p class="card-sub">该型号的 LOD 使用 0.7–1.7 mm 五档，尚未开放</p>
              </div>
            </div>
          </div>
          <p class="hint-text">
            0x4D 协议可以读取 LOD 索引，但为了避免把 1/2 mm 的模型写进其他档位，
            这里暂时只保留读取。传感器标志：<span class="mono">{cfg.sensorFlags.toString(16).padStart(2, '0')}</span>
          </p>
        </section>
      {/if}
    </div>
  {/if}
</div>
