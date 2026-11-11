<script lang="ts">
  import type { ButtonAction } from '../lib/api';
  import { actionShortLabel, BUTTON_LABELS } from '../lib/format';

  let {
    buttons = null,
    highlight = -1,
    onselect,
    label = true,
  }: {
    buttons?: ButtonAction[] | null;
    highlight?: number;
    onselect?: (index: number) => void;
    label?: boolean;
  } = $props();

  const uid = $props.id();

  function pick(index: number) {
    onselect?.(index);
  }

  function keydown(e: KeyboardEvent, index: number) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      pick(index);
    }
  }

  function actionName(index: number): string {
    if (!buttons || !buttons[index]) return BUTTON_LABELS[index];
    return `${BUTTON_LABELS[index]} · ${actionShortLabel(buttons[index])}`;
  }
</script>

<!--
  俯视的鼠标轮廓：不是装饰图，而是按键页的「预览」。
  每个热区都可点（或 Tab + 回车），把注意力带到对应的按键行。
-->
<svg class="mouseview" viewBox="0 0 240 392" role="img" aria-label="鼠标按键预览">
  <defs>
    <clipPath id="mv-clip-{uid}">
      <rect x="46" y="18" width="148" height="344" rx="74" />
    </clipPath>
    <linearGradient id="mv-body-{uid}" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="var(--mv-a)" />
      <stop offset="100%" stop-color="var(--mv-b)" />
    </linearGradient>
    <linearGradient id="mv-side-{uid}" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="var(--mv-side-a)" />
      <stop offset="100%" stop-color="var(--mv-side-b)" />
    </linearGradient>
  </defs>

  <!-- 影子 -->
  <ellipse cx="124" cy="356" rx="82" ry="18" fill="var(--mv-shadow)" />

  <!-- 主体 -->
  <rect
    x="46"
    y="18"
    width="148"
    height="344"
    rx="74"
    fill="url(#mv-body-{uid})"
    stroke="var(--mv-edge)"
    stroke-width="1.5"
  />

  <g clip-path="url(#mv-clip-{uid})">
    <!-- 上盖分缝：横向分按键区，纵向分左右键 -->
    <line x1="46" y1="148" x2="194" y2="148" stroke="var(--mv-seam)" stroke-width="1.5" />
    <line x1="120" y1="18" x2="120" y2="148" stroke="var(--mv-seam)" stroke-width="1.2" opacity="0.7" />
    <!-- 右侧键槽 -->
    <rect x="176" y="176" width="22" height="44" rx="10" fill="url(#mv-side-{uid})" stroke="var(--mv-edge)" />
    <rect x="176" y="228" width="22" height="44" rx="10" fill="url(#mv-side-{uid})" stroke="var(--mv-edge)" />
    <!-- 滚轮 -->
    <rect x="108" y="52" width="24" height="58" rx="12" fill="var(--mv-wheel)" stroke="var(--mv-edge)" />
    <rect x="114" y="58" width="12" height="26" rx="6" fill="var(--mv-accent)" opacity="0.95" />
    <!-- DPI 键 -->
    <circle cx="120" cy="160" r="7" fill="var(--mv-wheel)" stroke="var(--mv-edge)" />
    <!-- 底部灯带 -->
    <rect x="90" y="330" width="60" height="7" rx="3.5" fill="var(--mv-accent)" opacity="0.75" />

    <!-- 热区：左键 / 右键 / 中键 / 前进 / 后退 -->
    <g
      class="mv-hot"
      class:on={highlight === 0}
      role="button"
      tabindex="0"
      aria-label={actionName(0)}
      onclick={() => pick(0)}
      onkeydown={(e) => keydown(e, 0)}
    >
      <title>{actionName(0)}</title>
      <path d="M46 132 V92 A74 74 0 0 1 120 18 V132 Z" />
    </g>
    <g
      class="mv-hot"
      class:on={highlight === 1}
      role="button"
      tabindex="0"
      aria-label={actionName(1)}
      onclick={() => pick(1)}
      onkeydown={(e) => keydown(e, 1)}
    >
      <title>{actionName(1)}</title>
      <path d="M194 132 V92 A74 74 0 0 0 120 18 V132 Z" />
    </g>
    <g
      class="mv-hot"
      class:on={highlight === 2}
      role="button"
      tabindex="0"
      aria-label={actionName(2)}
      onclick={() => pick(2)}
      onkeydown={(e) => keydown(e, 2)}
    >
      <title>{actionName(2)}</title>
      <rect x="100" y="44" width="40" height="74" rx="16" />
    </g>
    <g
      class="mv-hot"
      class:on={highlight === 3}
      role="button"
      tabindex="0"
      aria-label={actionName(3)}
      onclick={() => pick(3)}
      onkeydown={(e) => keydown(e, 3)}
    >
      <title>{actionName(3)}</title>
      <rect x="168" y="170" width="38" height="56" rx="14" />
    </g>
    <g
      class="mv-hot"
      class:on={highlight === 4}
      role="button"
      tabindex="0"
      aria-label={actionName(4)}
      onclick={() => pick(4)}
      onkeydown={(e) => keydown(e, 4)}
    >
      <title>{actionName(4)}</title>
      <rect x="168" y="222" width="38" height="56" rx="14" />
    </g>
  </g>

  {#if label}
    <text x="120" y="382" text-anchor="middle" class="mv-caption">MCHOSE G3 V2</text>
  {/if}
</svg>
