<script lang="ts">
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { icons, ICON_SIZE } from '../lib/icons';
  import {
    device,
    setKey,
    restoreKeyboardLayer,
    canEditKeyboard,
    saveLabel,
  } from '../lib/device.svelte';
  import {
    ACTIONS,
    codeLabel,
    codeHex,
    defaultLayer,
    layoutKeys,
    parseCode,
    type KeyCode,
  } from '../lib/keyboard';
  import { showToast } from '../lib/toast.svelte';
  import KeyboardBoard from '../components/KeyboardBoard.svelte';

  const kb = $derived(device.keyboard);
  const layers = $derived(kb?.layers ?? []);
  const layout = layoutKeys();
  let layer = $state(0);
  let selected = $state(-1);
  let custom = $state('');

  const selectedPlacement = $derived(layout.items.find((p) => p.index === selected) ?? null);
  const currentCode = $derived<KeyCode>(
    (layers[layer]?.[selected] ?? [0, 0, 0, 0]) as KeyCode,
  );
  const resetCode = $derived<KeyCode>(
    selected >= 0 ? (defaultLayer(layer)[selected] ?? [0, 0, 0, 0]) : [0, 0, 0, 0],
  );
  const editable = $derived(canEditKeyboard());

  const layerNames = ['Default', 'Fn', 'Fn2'];

  function pick(index: number) {
    selected = index;
    custom = codeHex(currentCode);
  }

  function applyCode(code: KeyCode) {
    if (selected < 0) return;
    setKey(layer, selected, code);
    custom = codeHex(code);
  }

  function onSelect(e: Event) {
    const hex = (e.currentTarget as HTMLSelectElement).value;
    const code = parseCode(hex);
    if (code) applyCode(code);
  }

  function onCustom() {
    const code = parseCode(custom);
    if (!code) {
      showToast('自定义代码必须是 8 位十六进制，例如 00000004');
      return;
    }
    applyCode(code);
  }

  function resetLayer() {
    restoreKeyboardLayer(layer, defaultLayer(layer));
    selected = -1;
  }

  // 换层时保留选中键，但刷新自定义输入框
  $effect(() => {
    layer;
    selected;
    if (selected >= 0) custom = codeHex(currentCode);
  });
</script>

<div class="page">
  <div class="page-head">
    <div class="page-title">
      <h1>按键</h1>
      <p class="page-sub">
        三层按键映射写在键盘的板载存储里。点任意键查看和修改，改动即时写入。
      </p>
    </div>
    <div class="page-actions">
      {#if saveLabel()}<span class="save-chip" class:busy={device.saveState === 'saving'}>{saveLabel()}</span>{/if}
      <button class="btn" onclick={resetLayer} disabled={!editable}>
        <HugeiconsIcon icon={icons.reset} size={ICON_SIZE.inline} />
        恢复本层默认
      </button>
    </div>
  </div>

  {#if !kb}
    <div class="alert info">
      <HugeiconsIcon icon={icons.keyboard} size={ICON_SIZE.control} />
      <div class="alert-body">
        <strong>没有连接键盘</strong>
        <span>请先连接 2.4G 接收器。</span>
      </div>
    </div>
  {:else}
    <div class="layer-tabs" role="group" aria-label="按键层">
      {#each layerNames as name, i (name)}
        <button
          class:on={layer === i}
          role="tab"
          aria-selected={layer === i}
          onclick={() => (layer = i)}
        >
          <span class="layer-index">{i + 1}</span>
          {name}
        </button>
      {/each}
      <span class="layer-hint">当前显示：{layerNames[layer]} 层</span>
    </div>

    <section class="card pad kb-card">
      <div class="kb-wrap">
        <KeyboardBoard
          {layers}
          {layer}
          activeIndex={selected}
          onselect={pick}
        />
      </div>
    </section>

    <section class="card pad key-inspector">
      {#if selected >= 0 && selectedPlacement}
        <div class="inspector-head">
          <div>
            <span class="eyebrow">选中按键</span>
            <h3>{selectedPlacement.label}</h3>
            <span class="inspector-sub">
              槽位 {selected} · 当前：{codeLabel(currentCode)}
            </span>
          </div>
          <span class="code-chip">{codeHex(currentCode)}</span>
        </div>

        <div class="inspector-controls">
          <label class="inspector-field">
            <span>映射为</span>
            <select
              class="action-select wide"
              value={codeHex(currentCode)}
              disabled={!editable}
              onchange={onSelect}
            >
              {#if !ACTIONS.some((a) => codeHex(a.code) === codeHex(currentCode))}
                <option value={codeHex(currentCode)}>当前自定义（{codeLabel(currentCode)}）</option>
              {/if}
              {#each ACTIONS as a (codeHex(a.code))}
                <option value={codeHex(a.code)}>{a.label}</option>
              {/each}
            </select>
          </label>

          <label class="inspector-field">
            <span>自定义 4 字节代码</span>
            <div class="custom-code">
              <input
                class="st-input mono"
                value={custom}
                spellcheck="false"
                maxlength="11"
                disabled={!editable}
                oninput={(e) => (custom = e.currentTarget.value)}
                onkeydown={(e) => e.key === 'Enter' && onCustom()}
                placeholder="00000004"
              />
              <button class="btn small" onclick={onCustom} disabled={!editable}>应用</button>
            </div>
          </label>

          <button
            class="btn small"
            onclick={() => applyCode(resetCode)}
            disabled={!editable}
          >
            恢复此键默认
          </button>
        </div>
      {:else}
        <div class="inspector-empty">
          <HugeiconsIcon icon={icons.keyboard} size={ICON_SIZE.empty} />
          <div>
            <strong>点键盘上的任意一键</strong>
            <span>选中后可以把它映射成其他按键、修饰键、媒体键或自定义代码。</span>
          </div>
        </div>
      {/if}
    </section>

    <p class="footnote">
      共 113 个可映射槽位，其中 99 个出现在键盘上。Fn 与 Fn2 层只显示键盘上的实际按键；
      写按键后系统会短暂重新枚举键盘端点（约 0.1–0.3 秒）。
    </p>
  {/if}
</div>
