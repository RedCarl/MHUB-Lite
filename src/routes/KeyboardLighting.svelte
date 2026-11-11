<script lang="ts">
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { icons, ICON_SIZE } from '../lib/icons';
  import { device, setKeyboardPerformance, setLightColor, setLightMode, saveLabel, canEditKeyboard } from '../lib/device.svelte';
  import Slider from '../components/Slider.svelte';
  import Toggle from '../components/Toggle.svelte';

  const kb = $derived(device.keyboard);
  const perf = $derived(kb?.performance);
  const modes = $derived(kb?.lightModes ?? []);
  const mode = $derived(modes.find((m) => m.id === perf?.lightMode));
  const editable = $derived(canEditKeyboard());

  const hex = $derived(
    perf ? `#${perf.color.map((b) => b.toString(16).padStart(2, '0')).join('')}` : '#ff0000',
  );

  function setColor(value: string) {
    const clean = value.replace('#', '');
    if (clean.length !== 6) return;
    const rgb: [number, number, number] = [
      parseInt(clean.slice(0, 2), 16),
      parseInt(clean.slice(2, 4), 16),
      parseInt(clean.slice(4, 6), 16),
    ];
    setLightColor(rgb);
  }

  function pickMode(id: number) {
    setLightMode(id);
  }
</script>

<div class="page">
  <div class="page-head">
    <div class="page-title">
      <h1>灯效</h1>
      <p class="page-sub">
        选择灯效、亮度和颜色。键盘会即时保存，不需要额外的写入按钮。
      </p>
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
    <section class="lighting-hero card">
      <div class="light-preview" style="--light-color: {hex}">
        <div class="light-orb"></div>
        <div class="light-meta">
          <span class="eyebrow">当前灯效</span>
          <h2>{mode?.name ?? '未知'}</h2>
          <span class="light-sub">亮度 {perf.brightness}% · 速度 {perf.speed + 1}</span>
        </div>
      </div>

      <div class="lighting-controls">
        <label class="inspector-field">
          <span>灯效模式</span>
          <select
            class="action-select wide"
            value={perf.lightMode}
            disabled={!editable}
            onchange={(e) => pickMode(Number(e.currentTarget.value))}
          >
            {#each modes as m (m.id)}
              <option value={m.id}>{m.name}</option>
            {/each}
          </select>
        </label>

        <div class="lighting-row">
          <span class="lighting-label">亮度</span>
          <Slider
            value={perf.brightness}
            min={0}
            max={100}
            step={1}
            label="亮度"
            disabled={!editable || !mode?.brightness}
            oninput={(v) => setKeyboardPerformance({ brightness: v })}
          />
          <span class="lighting-value">{perf.brightness}%</span>
        </div>

        <div class="lighting-row">
          <span class="lighting-label">速度</span>
          <Slider
            value={perf.speed}
            min={0}
            max={4}
            step={1}
            label="灯效速度"
            disabled={!editable || !mode?.speed}
            oninput={(v) => setKeyboardPerformance({ speed: v })}
          />
          <span class="lighting-value">{perf.speed + 1}</span>
        </div>

        <div class="lighting-row">
          <span class="lighting-label">单色</span>
          <input
            class="color-input"
            type="color"
            value={hex}
            disabled={!editable || !mode?.color}
            oninput={(e) => setColor(e.currentTarget.value)}
            aria-label="灯效颜色"
          />
          <span class="lighting-value mono">{hex.toUpperCase()}</span>
        </div>

        <div class="lighting-row">
          <span class="lighting-label">多彩</span>
          <Toggle
            checked={perf.multiColor}
            disabled={!editable || !mode?.color}
            label="多彩"
            onchange={(v) => setKeyboardPerformance({ multiColor: v })}
          />
          <span class="lighting-value">{perf.multiColor ? '开' : '关'}</span>
        </div>
      </div>
    </section>

    <p class="footnote">
      灯效写入会整块覆盖键盘的灯效配置，所以每次都会先读取当前块再修改。逐键 RGB 调色暂未开放。
    </p>
  {/if}
</div>
