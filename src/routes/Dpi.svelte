<script lang="ts">
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { icons, ICON_SIZE } from '../lib/icons';
  import {
    device,
    draft,
    canEdit,
    restoreFactory,
    setActiveStage,
    setDpi,
    setStageCount,
    nudgeDpi,
  } from '../lib/device.svelte';
  import Segmented from '../components/Segmented.svelte';
  import Slider from '../components/Slider.svelte';

  const snap = $derived(device.current);
  const cfg = $derived(draft.config);
  const caps = $derived(snap?.info.capabilities ?? null);
  const editable = $derived(canEdit());

  const stageCountOptions = $derived(
    Array.from({ length: caps?.maxStages ?? 0 }, (_, i) => ({ value: i + 1, label: `${i + 1}` })),
  );
  const activeOptions = $derived(
    Array.from({ length: cfg?.stageCount ?? 0 }, (_, i) => ({ value: i + 1, label: `${i + 1}` })),
  );

  function setStageCountSafe(n: number) {
    setStageCount(n);
  }

  function onNumber(index: number, value: string) {
    const n = Number(value);
    if (!Number.isFinite(n)) return;
    setDpi(index, n);
  }
</script>

<div class="page">
  <div class="page-head">
    <div class="page-title">
      <h1>DPI</h1>
      <p class="page-sub">
        设置启用的档位数、当前档位，以及每一档的具体 DPI；改动先留在草稿，写入后才生效。
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
        <span>回到概览页扫描并连接 MCHOSE 设备后，这里才会出现可调的档位。</span>
      </div>
    </div>
  {:else if !snap.online || !cfg || !caps}
    <div class="alert warn">
      <HugeiconsIcon icon={icons.sleep} size={ICON_SIZE.control} />
      <div class="alert-body">
        <strong>鼠标正在休眠</strong>
        <span>配置在休眠时不可读也不可写。移动鼠标唤醒后，到概览页点「重新读取」。</span>
      </div>
    </div>
  {:else}
    <section class="card pad dpi-settings">
      <div class="setting-row">
        <div class="setting-label">
          <span>启用的档位数量</span>
          <span class="setting-desc">
            按鼠标上的 DPI 键会在这些档位之间循环；最多 {caps.maxStages} 档。
          </span>
        </div>
        <Segmented
          value={cfg.stageCount}
          options={stageCountOptions}
          onchange={setStageCountSafe}
          label="启用的档位数量"
        />
      </div>

      <div class="setting-row">
        <div class="setting-label">
          <span>当前使用的档位</span>
          <span class="setting-desc">决定鼠标此刻以哪一档 DPI 工作，也可以直接点下面的档位号码。</span>
        </div>
        <Segmented
          value={cfg.activeStage}
          options={activeOptions}
          onchange={setActiveStage}
          label="当前使用的档位"
        />
      </div>
    </section>

    <section class="stage-list">
      {#each cfg.stages.slice(0, cfg.stageCount) as dpi, index (index)}
        <article class="stage-card" class:on={cfg.activeStage === index + 1}>
          <button
            class="stage-badge"
            class:on={cfg.activeStage === index + 1}
            onclick={() => setActiveStage(index + 1)}
            disabled={!editable}
            title={`把第 ${index + 1} 档设为当前档位`}
            aria-label={`把第 ${index + 1} 档设为当前档位`}
          >
            {index + 1}
          </button>

          <div class="stage-body">
            <div class="stage-head">
              <div class="stage-heading">
                <span class="stage-title">第 {index + 1} 档</span>
                {#if cfg.activeStage === index + 1}
                  <span class="badge on">正在使用</span>
                {/if}
              </div>
              <label class="dpi-number">
                <input
                  type="number"
                  min={caps.dpiRange.min}
                  max={caps.dpiRange.max}
                  step={caps.dpiRange.step}
                  value={dpi}
                  disabled={!editable}
                  aria-label={`第 ${index + 1} 档 DPI`}
                  onchange={(e) => onNumber(index, e.currentTarget.value)}
                />
                <span>DPI</span>
              </label>
            </div>

            <div class="stage-controls">
              <button
                class="step"
                onclick={() => nudgeDpi(index, -caps.dpiRange.step)}
                disabled={!editable || dpi <= caps.dpiRange.min}
                aria-label="减少 100 DPI"
              >−</button>
              <Slider
                value={dpi}
                min={caps.dpiRange.min}
                max={caps.dpiRange.max}
                step={caps.dpiRange.step}
                label={`第 ${index + 1} 档 DPI 滑块`}
                disabled={!editable}
                oninput={(v) => setDpi(index, v)}
              />
              <button
                class="step"
                onclick={() => nudgeDpi(index, caps.dpiRange.step)}
                disabled={!editable || dpi >= caps.dpiRange.max}
                aria-label="增加 100 DPI"
              >+</button>
            </div>

            <div class="stage-scale">
              <span>{caps.dpiRange.min}</span>
              <span>步进 {caps.dpiRange.step}</span>
              <span>{caps.dpiRange.max}</span>
            </div>
          </div>
        </article>
      {/each}
    </section>

    <p class="footnote">
      第 1 档到第 {cfg.stageCount} 档会随鼠标指针移动方向循环切换；未启用的档位不会出现在鼠标上。
      写入后配置保存在板载 Flash，换电脑也保留。
    </p>
  {/if}
</div>
