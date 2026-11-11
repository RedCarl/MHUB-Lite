<script lang="ts">
  import { onMount } from 'svelte';
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { icons, ICON_SIZE } from './lib/icons';
  import { router, go, type RouteName } from './lib/router.svelte';
  import {
    device,
    init,
    pollLive,
    scan,
    clearError,
  } from './lib/device.svelte';
  import { prefs } from './lib/prefs.svelte';
  import { connectionLabel } from './lib/format';
  import TitleBar from './components/TitleBar.svelte';
  import WindowControls from './components/WindowControls.svelte';
  import DeviceBar from './components/DeviceBar.svelte';
  import Toast from './components/Toast.svelte';
  import Overview from './routes/Overview.svelte';
  import Dpi from './routes/Dpi.svelte';
  import Buttons from './routes/Buttons.svelte';
  import Performance from './routes/Performance.svelte';
  import Settings from './routes/Settings.svelte';

  // 自绘标题栏只出现在 Windows + Tauri（见 main.ts 的 data-chrome）
  const customChrome = document.documentElement.dataset.chrome === 'custom';

  let asideEl = $state<HTMLElement | null>(null);
  let mainEl = $state<HTMLElement | null>(null);
  let indTop = $state(0);
  let indH = $state(0);
  let indShow = $state(false);
  let indReady = $state(false);


  function measureNav() {
    const aside = asideEl;
    if (!aside) return;
    const btn = aside.querySelector<HTMLElement>('button.on');
    if (!btn) {
      indShow = false;
      return;
    }
    const ar = aside.getBoundingClientRect();
    const br = btn.getBoundingClientRect();
    indTop = br.top - ar.top;
    indH = br.height;
    indShow = true;
  }

  $effect(() => {
    // 路由或设备名变化都会影响侧栏排版，重新量一次选中块的位置
    router.route;
    device.current?.info.productName;
    const raf = requestAnimationFrame(() => {
      measureNav();
      if (!indReady) requestAnimationFrame(() => (indReady = true));
    });
    return () => cancelAnimationFrame(raf);
  });

  $effect(() => {
    router.route;
    mainEl?.scrollTo({ top: 0 });
  });

  // 轮询在线状态和电量；间隔来自设置页，关掉时完全不发请求。
  const hasDevice = $derived(!!device.current);
  $effect(() => {
    const seconds = prefs.pollSeconds;
    if (!seconds || !hasDevice) return;
    const id = setInterval(() => void pollLive(), seconds * 1000);
    return () => clearInterval(id);
  });

  onMount(() => {
    void init();
  });

  const navTop: { name: RouteName; label: string; icon: typeof icons.overview }[] = [
    { name: 'overview', label: '概览', icon: icons.overview },
    { name: 'dpi', label: 'DPI', icon: icons.dpi },
  ];
  const navTune: { name: RouteName; label: string; icon: typeof icons.overview }[] = [
    { name: 'buttons', label: '按键', icon: icons.buttons },
    { name: 'performance', label: '性能', icon: icons.performance },
  ];
  const navOther: { name: RouteName; label: string; icon: typeof icons.overview }[] = [
    { name: 'settings', label: '设置', icon: icons.settings },
  ];

  const activeNav = $derived(router.route);
  const snap = $derived(device.current);
  const footName = $derived(snap?.info.productName ?? '未连接设备');
  const footSub = $derived(
    snap
      ? `${connectionLabel(snap.info.connection)} · ${snap.online ? (snap.battery ? `${snap.battery.percent}%` : '在线') : '休眠中'}`
      : device.busy === 'scanning'
        ? '正在扫描…'
        : '插好接收器或数据线',
  );
</script>

<svelte:window onresize={measureNav} />

<div class="app">
  {#if customChrome}
    <TitleBar />
  {/if}

  <aside bind:this={asideEl}>
    <span
      class="nav-ind"
      class:show={indShow}
      class:ready={indReady}
      style="top: {indTop}px; height: {indH}px"
    ></span>

    <!-- macOS 用 Overlay 标题栏：给红绿灯留位置，并让这块区域可以拖动窗口 -->
    <div class="titlebar-drag" data-tauri-drag-region aria-hidden="true"></div>

    <div class="brand">
      <span class="logo"><HugeiconsIcon icon={icons.mouse} size={ICON_SIZE.inline} /></span>
      <span class="brand-text">
        <span class="brand-name">MHUB Lite</span>
        <span class="brand-sub">MCHOSE 设备管理</span>
      </span>
    </div>

    <nav class="side-nav">
      {#each navTop as n (n.name)}
        <button class:on={activeNav === n.name} onclick={() => go(n.name)}>
          <span class="ico"><HugeiconsIcon icon={n.icon} size={ICON_SIZE.control} /></span>
          {n.label}
        </button>
      {/each}
      <div class="nav-sep" aria-hidden="true"></div>
      {#each navTune as n (n.name)}
        <button class:on={activeNav === n.name} onclick={() => go(n.name)}>
          <span class="ico"><HugeiconsIcon icon={n.icon} size={ICON_SIZE.control} /></span>
          {n.label}
        </button>
      {/each}
      <div class="nav-sep" aria-hidden="true"></div>
      {#each navOther as n (n.name)}
        <button class:on={activeNav === n.name} onclick={() => go(n.name)}>
          <span class="ico"><HugeiconsIcon icon={n.icon} size={ICON_SIZE.control} /></span>
          {n.label}
        </button>
      {/each}
    </nav>

    <div class="side-foot">
      <button class="foot" onclick={() => go('overview')} title="查看设备概览">
        <span class="avatar" class:on={!!snap?.online}>
          <HugeiconsIcon icon={icons.mouse} size={ICON_SIZE.inline} />
          <i class="avatar-dot" class:on={!!snap?.online} class:demo={!!snap?.info.isDemo}></i>
        </span>
        <span class="foot-meta">
          <span class="foot-name">{footName}</span>
          <span class="foot-sub">{footSub}</span>
        </span>
      </button>
    </div>
  </aside>

  <main bind:this={mainEl}>
    {#if device.error}
      <div class="app-alert" role="alert">
        <HugeiconsIcon icon={icons.alert} size={ICON_SIZE.control} />
        <div class="app-alert-body">
          <strong>设备操作没有完成</strong>
          <span>{device.error}</span>
        </div>
        <button class="btn small" onclick={clearError}>关闭</button>
      </div>
    {/if}

    {#if router.route === 'overview'}
      <Overview />
    {:else if router.route === 'dpi'}
      <Dpi />
    {:else if router.route === 'buttons'}
      <Buttons />
    {:else if router.route === 'performance'}
      <Performance />
    {:else if router.route === 'settings'}
      <Settings />
    {/if}
  </main>

  <DeviceBar />

  {#if customChrome}
    <WindowControls />
  {/if}

  <Toast />
</div>
