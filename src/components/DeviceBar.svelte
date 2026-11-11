<script lang="ts">
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { icons, ICON_SIZE } from '../lib/icons';
  import {
    device,
    diffItems,
    isDirty,
    canApply,
    scan,
    refresh,
    apply,
    revertDraft,
  } from '../lib/device.svelte';
  import { connectionLabel } from '../lib/format';
  import { go } from '../lib/router.svelte';

  const snap = $derived(device.current);
  const online = $derived(!!snap?.online);
  const busy = $derived(device.busy);
  const applying = $derived(device.busy === 'applying');
  const dirtyItems = $derived(diffItems());
  const dirty = $derived(dirtyItems.length > 0);
  const demo = $derived(!!snap?.info.isDemo || device.browserDemo);

  const name = $derived(
    snap?.info.productName ?? (busy === 'scanning' ? '正在扫描设备…' : '未连接设备'),
  );

  const sub = $derived.by(() => {
    if (!snap) return device.browserDemo ? '浏览器演示模式' : '插好接收器或数据线';
    const parts = [connectionLabel(snap.info.connection)];
    if (snap.battery) parts.push(`${snap.battery.percent}%`);
    if (demo) parts.push('演示');
    return parts.join(' · ');
  });

  const syncTitle = $derived.by(() => {
    if (!snap) return '没有可写入的设备';
    if (!snap.online) return '设备休眠中';
    if (dirtyItems.length) return `${dirtyItems.join('、')} 未写入`;
    return '已与设备同步';
  });

  const syncHint = $derived.by(() => {
    if (!snap) return '点右侧按钮扫描 MCHOSE 接收器';
    if (!snap.online) return '移动鼠标或按任意键唤醒后再写入';
    if (dirtyItems.length) return '写入后立即生效，并保存在鼠标的板载 Flash';
    return '设备里的配置与当前页面一致';
  });
</script>

<footer>
  <div class="bar" class:no-device={!snap}>
    <button class="bar-device" onclick={() => go('overview')} title="回到概览">
      <span class="bar-glyph" class:on={online}>
        <HugeiconsIcon icon={icons.mouse} size={20} />
        <i class="bar-dot" aria-hidden="true"></i>
      </span>
      <span class="bar-meta">
        <span class="bar-name">{name}</span>
        <span class="bar-sub">{sub}</span>
      </span>
    </button>

    <div class="bar-sync" aria-live="polite">
      <span class="sync-icon" class:ok={!dirty && online} class:warn={dirty} class:off={!online}>
        {#if busy === 'applying'}
          <HugeiconsIcon class="spin" icon={icons.loading} size={ICON_SIZE.inline} />
        {:else if !snap}
          <HugeiconsIcon icon={icons.info} size={ICON_SIZE.inline} />
        {:else if dirty}
          <HugeiconsIcon icon={icons.alert} size={ICON_SIZE.inline} />
        {:else if online}
          <HugeiconsIcon icon={icons.check} size={ICON_SIZE.inline} />
        {:else}
          <HugeiconsIcon icon={icons.sleep} size={ICON_SIZE.inline} />
        {/if}
      </span>
      <span class="sync-body">
        <span class="sync-title">{syncTitle}</span>
        <span class="sync-hint">{syncHint}</span>
      </span>
    </div>

    <div class="bar-actions">
      {#if snap}
        <button
          class="bar-icon"
          onclick={() => void refresh()}
          disabled={busy !== null}
          title="从设备重新读取"
          aria-label="从设备重新读取"
        >
          <HugeiconsIcon
            class={busy === 'reading' ? 'spin' : ''}
            icon={busy === 'reading' ? icons.loading : icons.refresh}
            size={ICON_SIZE.control}
          />
        </button>
        <button
          class="bar-btn"
          onclick={revertDraft}
          disabled={!isDirty() || busy !== null}
          title="丢弃未写入的更改"
        >
          还原
        </button>
        <button
          class="bar-apply"
          onclick={() => void apply()}
          disabled={!canApply() || busy !== null}
          title="把当前配置写入鼠标板载 Flash"
        >
          {#if applying}
            <HugeiconsIcon class="spin" icon={icons.loading} size={ICON_SIZE.inline} />
            写入中…
          {:else}
            <HugeiconsIcon icon={icons.apply} size={ICON_SIZE.inline} />
            写入设备
          {/if}
        </button>
      {:else}
        <button
          class="bar-apply"
          onclick={() => void scan()}
          disabled={busy === 'scanning'}
        >
          <HugeiconsIcon
            class={busy === 'scanning' ? 'spin' : ''}
            icon={busy === 'scanning' ? icons.loading : icons.refresh}
            size={ICON_SIZE.inline}
          />
          {busy === 'scanning' ? '扫描中…' : '扫描设备'}
        </button>
      {/if}
    </div>
  </div>
</footer>
