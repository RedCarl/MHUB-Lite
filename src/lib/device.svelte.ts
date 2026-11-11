import {
  api,
  ApiFailure,
  cloneConfig,
  isTauri,
  type ButtonAction,
  type DeviceSummary,
  type ErrorCode,
  type MouseConfig,
  type Snapshot,
} from './api';
import { prefs, persistPrefs } from './prefs.svelte';
import { showToast } from './toast.svelte';
import { clamp, formatHz, formatMinutes, connectionLabel } from './format';

type BusyKind = 'scanning' | 'connecting' | 'reading' | 'applying' | null;

export const device = $state<{
  devices: DeviceSummary[];
  current: Snapshot | null;
  busy: BusyKind;
  error: string;
  errorCode: ErrorCode | null;
  version: string;
  initialized: boolean;
  browserDemo: boolean;
}>({
  devices: [],
  current: null,
  busy: null,
  error: '',
  errorCode: null,
  version: '',
  initialized: false,
  browserDemo: !isTauri,
});

/** 用户正在编辑的副本；写入前一直和「设备里的样子」分开。 */
export const draft = $state<{
  config: MouseConfig | null;
  buttons: ButtonAction[] | null;
}>({
  config: null,
  buttons: null,
});

function setError(e: unknown, prefix = '') {
  if (e instanceof ApiFailure) {
    device.errorCode = e.code;
    device.error = prefix ? `${prefix}：${e.message}` : e.message;
  } else {
    device.errorCode = 'hid';
    device.error = prefix ? `${prefix}：${String(e)}` : String(e);
  }
}

export function clearError() {
  device.error = '';
  device.errorCode = null;
}

function resetDraftFrom(snap: Snapshot | null) {
  if (snap?.config && snap.buttons) {
    draft.config = cloneConfig(snap.config);
    draft.buttons = [...snap.buttons];
  } else {
    draft.config = null;
    draft.buttons = null;
  }
}

export async function init() {
  if (device.initialized) return;
  try {
    device.version = await api.appVersion();
  } catch {
    device.version = '0.1.0';
  }
  if (isTauri && prefs.demoMode) {
    try {
      await api.setDemoMode(true);
    } catch {
      /* 旧后端没有演示模式时忽略 */
    }
  }
  await scan(false);
  device.initialized = true;
  // 首次进来自动连一台能用的设备，省掉一次点击。
  await autoConnect();
}

async function autoConnect() {
  if (device.current) return;
  const target = device.devices.find((d) => d.supported);
  if (target) await connect(target.key, true);
}

export async function scan(auto = true) {
  if (device.busy === 'applying') return;
  device.busy = 'scanning';
  clearError();
  try {
    const list = await api.scanDevices(prefs.showUnknown);
    device.devices = list;

    const stillThere =
      device.current && list.some((d) => d.key === device.current?.info.key);
    if (stillThere) {
      // 设备还在：不打断当前编辑，只刷新连接状态。
      return;
    }

    if (device.current) {
      device.current = null;
      resetDraftFrom(null);
    }
    if (auto) await autoConnect();
  } catch (e) {
    setError(e, '扫描设备失败');
  } finally {
    device.busy = null;
  }
}

export async function connect(key: string, quiet = false) {
  device.busy = 'connecting';
  clearError();
  try {
    const snap = await api.connect(key);
    device.current = snap;
    if (isTauri && snap.info.isDemo && !prefs.demoMode) {
      // 通过 MHUB_DEMO=1 启动时，让设置里的开关跟着真实状态走
      prefs.demoMode = true;
      persistPrefs();
    }
    resetDraftFrom(snap);
    if (!quiet) {
      showToast(snap.online ? '已读取设备配置' : '接收器已连接，鼠标正在休眠');
    }
  } catch (e) {
    setError(e, '连接设备失败');
    if (!quiet) showToast('连接设备失败');
  } finally {
    device.busy = null;
  }
}

export async function refresh(silent = false) {
  if (!device.current || device.busy === 'applying' || device.busy === 'connecting') return;
  const wasDirty = isDirty();
  if (!silent) device.busy = 'reading';
  try {
    const snap = await api.refresh();
    device.current = snap;
    if (!wasDirty) resetDraftFrom(snap);
    clearError();
    if (!silent) showToast('已从设备重新读取');
  } catch (e) {
    if (!silent) {
      setError(e, '重新读取失败');
      showToast('重新读取失败');
    }
  } finally {
    if (!silent) device.busy = null;
  }
}

/** 后台轮询只碰「在线状态 + 电量」，不覆盖用户正在改的草稿。 */
export async function pollLive() {
  if (!device.current || device.busy) return;
  try {
    const live = await api.liveStatus();
    const wasOnline = device.current.online;
    device.current.online = live.online;
    device.current.battery = live.battery;
    if (!wasOnline && live.online) {
      showToast('设备已唤醒，正在同步配置');
      await refresh(true);
    } else if (wasOnline && !live.online) {
      showToast('设备进入休眠：配置暂不可写');
    }
  } catch {
    /* 轮询失败保持安静，用户点「重新读取」时再报错 */
  }
}

export async function apply() {
  if (!device.current || !draft.config || !draft.buttons) return;
  if (!device.current.online) {
    setError(new ApiFailure('offline', '鼠标正在休眠，先移动鼠标唤醒再写入'), '无法写入');
    return;
  }
  if (!isDirty()) {
    showToast('没有需要写入的更改');
    return;
  }

  device.busy = 'applying';
  clearError();
  try {
    const snap = await api.applyConfig(cloneConfig(draft.config), [...draft.buttons]);
    device.current = snap;
    resetDraftFrom(snap);
    showToast('已写入鼠标，并保存到板载 Flash');
  } catch (e) {
    setError(e, '写入失败');
    showToast('写入失败');
  } finally {
    device.busy = null;
  }
}

export function revertDraft() {
  resetDraftFrom(device.current);
  showToast('已还原为设备当前配置');
}

export function restoreFactory() {
  const snap = device.current;
  if (!snap) return;
  draft.config = cloneConfig(snap.factoryConfig);
  draft.buttons = [...snap.factoryButtons];
  showToast('已填入原厂默认值，写入后生效');
}

export async function disconnect() {
  try {
    await api.disconnect();
  } catch {
    /* 关连接失败也把界面复位 */
  }
  device.current = null;
  resetDraftFrom(null);
  await scan(false);
  showToast('已断开设备');
}

export async function setDemoMode(enabled: boolean) {
  prefs.demoMode = enabled;
  persistPrefs();
  device.busy = 'scanning';
  clearError();
  try {
    await api.setDemoMode(enabled);
    device.current = null;
    resetDraftFrom(null);
    await scan(false);
    await autoConnect();
    showToast(enabled ? '已进入演示模式（不碰真实硬件）' : '已退出演示模式');
  } catch (e) {
    setError(e, '切换演示模式失败');
  } finally {
    device.busy = null;
  }
}

export async function setShowUnknown(show: boolean) {
  prefs.showUnknown = show;
  persistPrefs();
  await scan(false);
}

// ---------------------------------------------------------------------------
// 编辑动作
// ---------------------------------------------------------------------------

export function caps() {
  return device.current?.info.capabilities ?? null;
}

function editableConfig(): MouseConfig | null {
  if (!device.current?.online || device.busy === 'applying') return null;
  return draft.config;
}

export function setStageCount(n: number) {
  const cfg = editableConfig();
  const c = caps();
  if (!cfg || !c?.dpi) return;
  cfg.stageCount = clamp(n, 1, c.maxStages);
  if (cfg.activeStage > cfg.stageCount) cfg.activeStage = cfg.stageCount;
}

export function setActiveStage(n: number) {
  const cfg = editableConfig();
  const c = caps();
  if (!cfg || !c?.dpi) return;
  cfg.activeStage = clamp(n, 1, cfg.stageCount);
}

export function setDpi(index: number, value: number) {
  const cfg = editableConfig();
  const c = caps();
  if (!cfg || !c?.dpi) return;
  const { min, max, step } = c.dpiRange;
  const snapped = Math.round(value / step) * step;
  cfg.stages[index] = clamp(snapped, min, max);
}

export function nudgeDpi(index: number, delta: number) {
  const cfg = editableConfig();
  const c = caps();
  if (!cfg || !c?.dpi) return;
  setDpi(index, cfg.stages[index] + delta);
}

export function setPolling(hz: number) {
  const cfg = editableConfig();
  if (!cfg || !caps()?.polling) return;
  cfg.pollingHz = hz;
}

export function setDebounce(ms: number) {
  const cfg = editableConfig();
  if (!cfg || !caps()?.debounce) return;
  cfg.debounceMs = clamp(Math.round(ms), 0, 20);
}

export function setSleep(minutes: number) {
  const cfg = editableConfig();
  if (!cfg || !caps()?.sleep) return;
  cfg.sleepMinutes = clamp(Math.round(minutes), 0, 100);
}

export function setLod(lod: 'low' | 'high') {
  const cfg = editableConfig();
  if (!cfg || !caps()?.lod) return;
  cfg.lod = lod;
}

export function setScrollReversed(reversed: boolean) {
  const cfg = editableConfig();
  if (!cfg || !caps()?.scrollDirection) return;
  cfg.scrollReversed = reversed;
}

export function setButton(index: number, action: ButtonAction) {
  if (index === 0) return; // 左键由固件锁死
  const buttons = draft.buttons;
  if (!buttons || !device.current?.online || !caps()?.buttons) return;
  buttons[index] = action;
}

// ---------------------------------------------------------------------------
// 草稿差异
// ---------------------------------------------------------------------------

function sameArray<T>(a: readonly T[], b: readonly T[]): boolean {
  return a.length === b.length && a.every((v, i) => v === b[i]);
}

/** 每一组改动单独计数，底栏用它显示「N 项未写入」。 */
export function diffItems(): string[] {
  const saved = device.current?.config;
  const cfg = draft.config;
  const savedButtons = device.current?.buttons;
  const buttons = draft.buttons;
  if (!saved || !cfg) return [];

  const items: string[] = [];
  if (
    saved.stageCount !== cfg.stageCount ||
    saved.activeStage !== cfg.activeStage ||
    !sameArray(saved.stages, cfg.stages)
  ) {
    items.push('DPI');
  }
  if (saved.pollingHz !== cfg.pollingHz) items.push('回报率');
  if (saved.debounceMs !== cfg.debounceMs) items.push('防抖');
  if (saved.sleepMinutes !== cfg.sleepMinutes) items.push('休眠');
  if (saved.lod !== cfg.lod) items.push('静默高度');
  if (saved.scrollReversed !== cfg.scrollReversed) items.push('滚轮方向');
  if (savedButtons && buttons && !sameArray(savedButtons, buttons)) items.push('按键');
  return items;
}

export function isDirty(): boolean {
  return diffItems().length > 0;
}

export function isOnline(): boolean {
  return !!device.current?.online;
}

export function canEdit(): boolean {
  return !!device.current?.online && !!draft.config && device.busy !== 'applying';
}

export function canApply(): boolean {
  return canEdit() && isDirty();
}

// ---------------------------------------------------------------------------
// 诊断
// ---------------------------------------------------------------------------

export function diagnosticsText(): string {
  const snap = device.current;
  if (!snap) return 'MHUB Lite — 未连接设备';
  const c = snap.config;
  const lines = [
    'MHUB Lite 诊断信息',
    `版本: ${device.version}`,
    `平台: ${navigator.userAgent}`,
    `设备: ${snap.info.productName} (${snap.info.model ?? '未知型号'})`,
    `VID:PID: ${snap.info.vendorId.toString(16).toUpperCase()}:${snap.info.productId.toString(16).toUpperCase()}`,
    `连接: ${connectionLabel(snap.info.connection)}`,
    `固件: ${snap.info.firmware ?? '未知'} / ${snap.info.buildDate ?? '未知'}`,
    `序列号: ${snap.info.serial ?? '无'}`,
    `在线: ${snap.online ? '是' : '否'}`,
    `电量: ${snap.battery ? `${snap.battery.percent}%${snap.battery.charging ? '（充电中）' : ''}` : '未知'}`,
  ];
  if (c) {
    lines.push(
      `回报率: ${formatHz(c.pollingHz)}`,
      `DPI 档位: ${c.stageCount} / 当前第 ${c.activeStage} 档`,
      `各档 DPI: ${c.stages.slice(0, c.stageCount).join(' / ')}`,
      `防抖: ${c.debounceMs} ms`,
      `休眠: ${formatMinutes(c.sleepMinutes)}`,
      `静默高度: ${c.lod === 'low' ? '1 mm' : '2 mm'}`,
      `滚轮方向: ${c.scrollReversed ? '反转' : '标准'}`,
    );
  }
  if (snap.buttons) lines.push(`按键: ${snap.buttons.join(' / ')}`);
  const diff = diffItems();
  if (diff.length) lines.push(`未写入: ${diff.join('、')}`);
  return lines.join('\n');
}
