<script lang="ts">
  import { onMount } from 'svelte';
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { icons, ICON_SIZE } from './lib/icons';
  import { router, go, type RouteName } from './lib/router.svelte';
  import {
    device,
    init,
    pollLive,
    clearError,
    switchTo,
    saveLabel,
    activeInfo,
    activeBattery,
    isOnline,
  } from './lib/device.svelte';
  import { prefs } from './lib/prefs.svelte';
  import { connectionLabel } from './lib/format';
  import Logo from './components/Logo.svelte';
  import TitleBar from './components/TitleBar.svelte';
  import WindowControls from './components/WindowControls.svelte';
  import Toast from './components/Toast.svelte';
  import Overview from './routes/Overview.svelte';
  import Dpi from './routes/Dpi.svelte';
  import Buttons from './routes/Buttons.svelte';
  import KeyboardKeys from './routes/KeyboardKeys.svelte';
  import KeyboardLighting from './routes/KeyboardLighting.svelte';
  import Performance from './routes/Performance.svelte';
  import Settings from './routes/Settings.svelte';

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
    const btn = aside.querySelector<HTMLElement>('.side-nav button.on');
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
    router.route;
    device.active;
    device.mouse?.info.productName;
    device.keyboard?.info.productName;
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

  const hasDevice = $derived(!!device.mouse || !!device.keyboard);
  $effect(() => {
    const seconds = prefs.pollSeconds;
    if (!seconds || !hasDevice) return;
    const id = setInterval(() => void pollLive(), seconds * 1000);
    return () => clearInterval(id);
  });

  // 设备切换后，当前页面如果不适用就回概览。
  $effect(() => {
    const active = device.active;
    const route = router.route;
    if (!active) return;
    if (active === 'keyboard' && route === 'dpi') {
      go('overview');
    }
    if (active === 'mouse' && route === 'lighting') {
      go('overview');
    }
    if (active === 'mouse' && route === 'buttons' && !device.mouse?.info.capabilities.buttons) {
      go('overview');
    }
  });

  onMount(() => {
    void init();
  });

  type NavItem = { name: RouteName; label: string; icon: typeof icons.overview };

  const navTop = $derived<NavItem[]>([
    { name: 'overview', label: '概览', icon: icons.overview },
    ...(device.active === 'keyboard'
      ? [{ name: 'buttons', label: '按键', icon: icons.keyboard } as NavItem]
      : device.active === 'mouse'
        ? [
            { name: 'dpi', label: 'DPI', icon: icons.dpi } as NavItem,
            ...(device.mouse?.info.capabilities.buttons
              ? [{ name: 'buttons', label: '按键', icon: icons.buttons } as NavItem]
              : []),
          ]
        : []),
  ]);

  const navTune = $derived<NavItem[]>([
    ...(device.active === 'keyboard'
      ? [{ name: 'lighting', label: '灯效', icon: icons.apply } as NavItem]
      : []),
    ...(device.active
      ? [{ name: 'performance', label: '性能', icon: icons.performance } as NavItem]
      : []),
  ]);

  const activeNav = $derived(router.route);
  const info = $derived(activeInfo());
  const battery = $derived(activeBattery());
  const online = $derived(isOnline());
  const bothDevices = $derived(!!device.mouse && !!device.keyboard);
  const footName = $derived(info?.productName ?? '未连接设备');
  const footSub = $derived.by(() => {
    if (!info) return device.busy === 'scanning' ? '正在扫描…' : '插好设备或接收器';
    const parts = [connectionLabel(info.connection)];
    if (battery) parts.push(`${battery.percent}%`);
    if (saveLabel()) parts.push(saveLabel());
    return parts.join(' · ');
  });
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

    <div class="titlebar-drag" data-tauri-drag-region aria-hidden="true"></div>

    <div class="brand">
      <span class="logo"><Logo size={20} /></span>
      <span class="brand-text">
        <span class="brand-name">MHUB Lite</span>
        <span class="brand-sub">MCHOSE 设备管理</span>
      </span>
    </div>

    {#if bothDevices}
      <div class="device-switch" role="group" aria-label="切换设备">
        <button
          class:on={device.active === 'mouse'}
          onclick={() => switchTo('mouse')}
          title="切换到鼠标"
        >
          <HugeiconsIcon icon={icons.mouse} size={ICON_SIZE.inline} />
          鼠标
        </button>
        <button
          class:on={device.active === 'keyboard'}
          onclick={() => switchTo('keyboard')}
          title="切换到键盘"
        >
          <HugeiconsIcon icon={icons.keyboard} size={ICON_SIZE.inline} />
          键盘
        </button>
      </div>
    {/if}

    <nav class="side-nav">
      {#each navTop as n (n.name)}
        <button class:on={activeNav === n.name} onclick={() => go(n.name)}>
          <span class="ico"><HugeiconsIcon icon={n.icon} size={ICON_SIZE.control} /></span>
          {n.label}
        </button>
      {/each}
      {#if navTune.length}
        <div class="nav-sep" aria-hidden="true"></div>
        {#each navTune as n (n.name)}
          <button class:on={activeNav === n.name} onclick={() => go(n.name)}>
            <span class="ico"><HugeiconsIcon icon={n.icon} size={ICON_SIZE.control} /></span>
            {n.label}
          </button>
        {/each}
      {/if}
    </nav>

    <div class="side-foot">
      <button
        class="settings-btn"
        class:is-active={activeNav === 'settings'}
        onclick={() => go('settings')}
        aria-current={activeNav === 'settings' ? 'page' : undefined}
        title="打开设置"
      >
        <span class="ico"><HugeiconsIcon icon={icons.settings} size={ICON_SIZE.control} /></span>
        <span>设置</span>
      </button>

      <button class="foot" onclick={() => go('overview')} title="查看设备概览">
        <span class="avatar" class:on={online}>
          {#if device.active === 'keyboard'}
            <HugeiconsIcon icon={icons.keyboard} size={ICON_SIZE.inline} />
          {:else}
            <HugeiconsIcon icon={icons.mouse} size={ICON_SIZE.inline} />
          {/if}
          <i class="avatar-dot" class:on={online} class:saving={device.saveState === 'saving'}></i>
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
    {:else if router.route === 'dpi' && device.active === 'mouse'}
      <Dpi />
    {:else if router.route === 'buttons' && device.active === 'keyboard'}
      <KeyboardKeys />
    {:else if router.route === 'buttons' && device.active === 'mouse'}
      <Buttons />
    {:else if router.route === 'lighting' && device.active === 'keyboard'}
      <KeyboardLighting />
    {:else if router.route === 'performance'}
      <Performance />
    {:else if router.route === 'settings'}
      <Settings />
    {:else}
      <Overview />
    {/if}
  </main>

  {#if customChrome}
    <WindowControls />
  {/if}

  <Toast />
</div>
