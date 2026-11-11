<script lang="ts">
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { icons, ICON_SIZE } from '../lib/icons';
  import {
    device,
    draft,
    canEdit,
    setButton,
    setScrollReversed,
  } from '../lib/device.svelte';
  import { FACTORY_BUTTONS, type ButtonAction } from '../lib/api';
  import { actionLabel, BUTTON_LABELS } from '../lib/format';
  import { showToast } from '../lib/toast.svelte';
  import { uiState } from '../lib/router.svelte';
  import Segmented from '../components/Segmented.svelte';
  import MouseView from '../components/MouseView.svelte';

  const snap = $derived(device.current);
  const cfg = $derived(draft.config);
  const buttons = $derived(draft.buttons);
  const editable = $derived(canEdit());
  let hovered = $state(uiState.buttonFocus);

  // 从概览的鼠标预览点进来时，先把对应行亮起来，再清掉一次性状态
  $effect(() => {
    if (uiState.buttonFocus >= 0) {
      hovered = uiState.buttonFocus;
      uiState.buttonFocus = -1;
    }
  });

  const rowIcons = [
    icons.buttonLeft,
    icons.buttonRight,
    icons.buttonMiddle,
    icons.buttonForward,
    icons.buttonBackward,
  ];

  const rowDesc = [
    '由固件锁定：无论写什么都保持左键单击',
    '默认右键单击',
    '默认中键单击',
    '默认前进（Mouse 5）',
    '默认后退（Mouse 4）',
  ];

  const actionOptions: { value: ButtonAction; label: string }[] = [
    { value: 'left', label: '左键单击' },
    { value: 'right', label: '右键单击' },
    { value: 'middle', label: '中键单击' },
    { value: 'forward', label: '前进（Mouse 5）' },
    { value: 'backward', label: '后退（Mouse 4）' },
    { value: 'disabled', label: '禁用（不产生输入）' },
  ];

  function optionsFor(current: ButtonAction) {
    if (current.startsWith('custom:')) {
      return [{ value: current, label: '自定义动作（保留原值）' }, ...actionOptions];
    }
    return actionOptions;
  }

  function resetButtons() {
    for (let i = 1; i < 5; i++) setButton(i, FACTORY_BUTTONS[i]);
    setScrollReversed(false);
    showToast('按键已恢复默认映射，写入后生效');
  }

  const scrollOptions = [
    { value: 'standard', label: '标准' },
    { value: 'reverse', label: '反转' },
  ];
</script>

<div class="page">
  <div class="page-head">
    <div class="page-title">
      <h1>按键</h1>
      <p class="page-sub">
        把鼠标上的按键映射成别的动作。映射写在鼠标的板载 Flash 里，不依赖后台程序。
      </p>
    </div>
    <div class="page-actions">
      <button class="btn" onclick={resetButtons} disabled={!editable}>
        <HugeiconsIcon icon={icons.reset} size={ICON_SIZE.inline} />
        恢复默认按键
      </button>
    </div>
  </div>

  {#if !snap}
    <div class="alert info">
      <HugeiconsIcon icon={icons.info} size={ICON_SIZE.control} />
      <div class="alert-body">
        <strong>还没有连接设备</strong>
        <span>回到概览页连接鼠标后，才能读取和改写按键映射。</span>
      </div>
    </div>
  {:else if !snap.online || !cfg || !buttons}
    <div class="alert warn">
      <HugeiconsIcon icon={icons.sleep} size={ICON_SIZE.control} />
      <div class="alert-body">
        <strong>鼠标正在休眠</strong>
        <span>按键映射需要设备在线才能读写。唤醒鼠标后重新读取。</span>
      </div>
    </div>
  {:else}
    <div class="buttons-layout">
      <section class="card button-card">
        <div class="card-head">
          <div>
            <h3>按键映射</h3>
            <p class="card-sub">左键由固件锁定；其余四颗可自由改</p>
          </div>
        </div>

        <div class="button-rows">
          {#each [0, 1, 2, 3, 4] as index (index)}
            <div
              class="button-row"
              class:locked={index === 0}
              class:focus={hovered === index}
              role="group"
              aria-label={BUTTON_LABELS[index]}
              onmouseenter={() => (hovered = index)}
              onmouseleave={() => (hovered = -1)}
              onfocusin={() => (hovered = index)}
              onfocusout={() => (hovered = -1)}
            >
              <span class="button-ico">
                <HugeiconsIcon icon={rowIcons[index]} size={ICON_SIZE.control} />
              </span>
              <span class="button-main">
                <span class="button-name">
                  {BUTTON_LABELS[index]}
                  {#if index === 0}<span class="lock">锁定</span>{/if}
                </span>
                <span class="button-desc">{rowDesc[index]}</span>
              </span>

              {#if index === 0}
                <span class="locked-action">{actionLabel(buttons[0])}</span>
              {:else}
                <select
                  class="action-select"
                  aria-label={`${BUTTON_LABELS[index]}的动作`}
                  value={buttons[index]}
                  disabled={!editable}
                  onchange={(e) => setButton(index, e.currentTarget.value as ButtonAction)}
                >
                  {#each optionsFor(buttons[index]) as option (option.value)}
                    <option value={option.value}>{option.label}</option>
                  {/each}
                </select>
              {/if}
            </div>
          {/each}
        </div>

        <p class="card-note">
          <HugeiconsIcon icon={icons.info} size={ICON_SIZE.inline} />
          写入按键后，系统会短暂重新枚举鼠标/键盘端点（约 0.1–0.3 秒），随后恢复；这是固件刷新描述符的正常行为。
        </p>
      </section>

      <div class="buttons-aside">
        <section class="card pad preview-card">
          <div class="card-head compact">
            <div>
              <h3>预览</h3>
              <p class="card-sub">指针悬停在左侧的行上，这里会亮起对应区域</p>
            </div>
          </div>
          <div class="preview-figure">
            <MouseView buttons={buttons} highlight={hovered} />
          </div>
          <div class="preview-legend">
            {#each [0, 1, 2, 3, 4] as index (index)}
              <span class="legend-item">
                <i class="legend-dot" class:on={hovered === index}></i>
                {BUTTON_LABELS[index]}
              </span>
            {/each}
          </div>
        </section>

        <section class="card pad">
          <div class="card-head compact">
            <div>
              <h3>滚轮方向</h3>
              <p class="card-sub">标准方向之外的另一种习惯</p>
            </div>
          </div>
          <Segmented
            value={cfg.scrollReversed ? 'reverse' : 'standard'}
            options={scrollOptions}
            onchange={(v) => setScrollReversed(v === 'reverse')}
            label="滚轮方向"
          />
          <p class="hint-text">
            {cfg.scrollReversed
              ? '当前为反转：滚轮向上拨，内容向下移动。'
              : '当前为标准：滚轮向上拨，内容向上移动。'}
          </p>
        </section>
      </div>
    </div>
  {/if}
</div>
