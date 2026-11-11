<script lang="ts">
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { icons, ICON_SIZE } from '../lib/icons';
  import {
    device,
    draft,
    canEdit,
    restoreFactory,
    setPolling,
    setDebounce,
    setSleep,
    setLod,
  } from '../lib/device.svelte';
  import { formatHz } from '../lib/format';
  import Segmented from '../components/Segmented.svelte';
  import Slider from '../components/Slider.svelte';

  const snap = $derived(device.current);
  const cfg = $derived(draft.config);
  const caps = $derived(snap?.info.capabilities ?? null);
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
      <p class="page-sub">
        回报率、防抖、休眠和静默高度。它们都写在板载 Flash 里，换一台电脑同样生效。
      </p>
    </div>
    <div class="page-actions">
      <button class="btn" onclick={restoreFactory} disabled={!editable}>
        <HugeiconsIcon icon={icons.reset} size={ICON_SIZE.inline} />
        恢复默认值
      </button>
    </div>
  </div>

  {#if !snap}
    <div class="alert info">
      <HugeiconsIcon icon={icons.info} size={ICON_SIZE.control} />
      <div class="alert-body">
        <strong>还没有连接设备</strong>
        <span>回到概览页连接鼠标后，才能调节性能参数。</span>
      </div>
    </div>
  {:else if !snap.online || !cfg || !caps}
    <div class="alert warn">
      <HugeiconsIcon icon={icons.sleep} size={ICON_SIZE.control} />
      <div class="alert-body">
        <strong>鼠标正在休眠</strong>
        <span>性能参数需要设备在线才能读写。唤醒鼠标后重新读取。</span>
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

        <Segmented
          value={cfg.pollingHz}
          options={rateOptions}
          onchange={setPolling}
          label="回报率"
        />

        <p class="hint-text">
          {#if snap.info.connection === 'wired'}
            当前用 USB 线连接：有线模式固定以 1000 Hz 工作，这里保存的是拔线后 2.4G 模式使用的值。
          {:else}
            当前用 2.4G 接收器连接，所选值立即作用于无线模式。
          {/if}
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
        <div class="slider-scale">
          <span>0 ms</span>
          <span>20 ms</span>
        </div>
        <div class="chips">
          {#each debouncePresets as ms (ms)}
            <button
              class="chip"
              class:on={cfg.debounceMs === ms}
              onclick={() => setDebounce(ms)}
              disabled={!editable}
            >{ms} ms</button>
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
          <span class="current-pill">
            {cfg.sleepMinutes === 0 ? '永不休眠' : `${cfg.sleepMinutes} 分钟`}
          </span>
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
        <div class="slider-scale">
          <span>永不休眠</span>
          <span>100 分钟</span>
        </div>
        <div class="chips">
          {#each sleepPresets as m (m)}
            <button
              class="chip"
              class:on={cfg.sleepMinutes === m}
              onclick={() => setSleep(m)}
              disabled={!editable}
            >{m === 0 ? '永不' : `${m} 分钟`}</button>
          {/each}
        </div>
        <p class="hint-text">仅无线模式生效；有线连接不受影响。</p>
      </section>

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

        <p class="hint-text">
          低静默高度适合抬鼠操作；如果桌面不平导致光标抖动，改成 2 mm 更稳。
        </p>
      </section>
    </div>

    <p class="footnote">
      这些参数会同时写入鼠标的板载存储，和当前这台电脑无关。恢复默认值只会改动草稿，点底栏「写入设备」后才会真正生效。
    </p>
  {/if}
</div>
