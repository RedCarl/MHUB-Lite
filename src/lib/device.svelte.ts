import { api, ApiFailure, cloneConfig, isTauri } from './api';
import type {
  Battery,
  ButtonAction,
  DeviceKind,
  DeviceSummary,
  ErrorCode,
  KeyboardInput,
  KeyboardPerformance,
  KeyboardSnapshot,
  MouseConfig,
  Snapshot,
} from './api';
import { prefs, persistPrefs } from './prefs.svelte';
import { showToast } from './toast.svelte';
import { clamp, formatHz, formatMinutes, connectionLabel } from './format';

type BusyKind = 'scanning' | 'connecting' | 'reading' | null;
export type SaveState = 'idle' | 'saving' | 'saved' | 'error';

export const device = $state<{
  devices: DeviceSummary[];
  mouse: Snapshot | null;
  /** 兼容旧页面：指向当前鼠标快照 */
  current: Snapshot | null;
  keyboard: KeyboardSnapshot | null;
  active: DeviceKind | null;
  busy: BusyKind;
  saveState: SaveState;
  error: string;
  errorCode: ErrorCode | null;
  version: string;
  initialized: boolean;
  browserDemo: boolean;
}>({
  devices: [],
  mouse: null,
  current: null,
  keyboard: null,
  active: null,
  busy: null,
  saveState: 'idle',
  error: '',
  errorCode: null,
  version: '',
  initialized: false,
  browserDemo: !isTauri,
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

let saveTimer: ReturnType<typeof setTimeout> | null = null;
let saveIdleTimer: ReturnType<typeof setTimeout> | null = null;
let savesInFlight = 0;

function beginSave() {
  savesInFlight += 1;
  if (saveIdleTimer) clearTimeout(saveIdleTimer);
  device.saveState = 'saving';
}

function endSave() {
  savesInFlight = Math.max(0, savesInFlight - 1);
  if (savesInFlight > 0) return;
  device.saveState = 'saved';
  if (saveIdleTimer) clearTimeout(saveIdleTimer);
  saveIdleTimer = setTimeout(() => {
    if (savesInFlight === 0 && device.saveState === 'saved') device.saveState = 'idle';
  }, 1600);
}

export function saveLabel(): string {
  switch (device.saveState) {
    case 'saving':
      return '保存中…';
    case 'saved':
      return '已保存';
    case 'error':
      return '保存失败';
    default:
      return '';
  }
}

export function activeInfo() {
  return device.active === 'keyboard' ? device.keyboard?.info ?? null : device.mouse?.info ?? null;
}

export function activeBattery(): Battery | null {
  return device.active === 'keyboard'
    ? device.keyboard?.battery ?? null
    : device.mouse?.battery ?? null;
}

export function isOnline(): boolean {
  return device.active === 'keyboard'
    ? !!device.keyboard?.online
    : !!device.mouse?.online;
}

export function switchTo(kind: DeviceKind) {
  if (kind === 'mouse' && !device.mouse) return;
  if (kind === 'keyboard' && !device.keyboard) return;
  device.active = kind;
}

export async function init() {
  if (device.initialized) return;
  try {
    device.version = await api.appVersion();
  } catch {
    device.version = '0.1.0';
  }
  await scan();
  device.initialized = true;
}

let scanning = false;

export async function scan(auto = true) {
  if (scanning) return;
  scanning = true;
  device.busy = 'scanning';
  clearError();
  try {
    const list = await api.scanDevices(prefs.showUnknown);
    device.devices = list;

    if (!auto) {
      if (device.mouse && !list.some((d) => d.key === device.mouse?.info.key)) {
        device.mouse = null;
        device.current = null;
      }
      if (device.keyboard && !list.some((d) => d.key === device.keyboard?.info.key)) {
        device.keyboard = null;
      }
      syncActive();
      return;
    }

    const mouseTarget = list.find((d) => d.kind === 'mouse' && d.supported) ?? null;
    const keyboardTarget = list.find((d) => d.kind === 'keyboard' && d.supported) ?? null;
    const keepMouse = device.mouse && list.some((d) => d.key === device.mouse?.info.key);
    const keepKeyboard =
      device.keyboard && list.some((d) => d.key === device.keyboard?.info.key);

    const jobs: Promise<unknown>[] = [];
    if (!keepMouse) {
      if (device.mouse) {
        device.mouse = null;
        device.current = null;
      }
      if (mouseTarget) jobs.push(connectMouse(mouseTarget.key, true));
    }
    if (!keepKeyboard) {
      if (device.keyboard) device.keyboard = null;
      if (keyboardTarget) jobs.push(connectKeyboard(keyboardTarget.key, true));
    }
    if (jobs.length) {
      device.busy = 'connecting';
      await Promise.allSettled(jobs);
    }
    syncActive();
  } catch (e) {
    setError(e, '扫描设备失败');
  } finally {
    scanning = false;
    device.busy = null;
  }
}

function syncActive() {
  if (device.active === 'mouse' && device.mouse) return;
  if (device.active === 'keyboard' && device.keyboard) return;
  device.active = device.mouse ? 'mouse' : device.keyboard ? 'keyboard' : null;
}

async function connectMouse(key: string, quiet = false) {
  try {
    const snap = await api.connect(key);
    device.mouse = snap;
    device.current = snap;
    device.active = 'mouse';
    if (!quiet) showToast('已连接鼠标');
    return snap;
  } catch (e) {
    setError(e, '连接鼠标失败');
    throw e;
  }
}

async function connectKeyboard(key: string, quiet = false) {
  try {
    const snap = await api.connectKeyboard(key);
    device.keyboard = snap;
    if (!device.mouse) device.active = 'keyboard';
    if (!quiet) showToast('已连接键盘');
    return snap;
  } catch (e) {
    setError(e, '连接键盘失败');
    throw e;
  }
}

export async function connect(key: string, quiet = false) {
  const summary = device.devices.find((d) => d.key === key);
  if (!summary) return;
  device.busy = 'connecting';
  clearError();
  try {
    if (summary.kind === 'keyboard') await connectKeyboard(key, quiet);
    else await connectMouse(key, quiet);
  } finally {
    device.busy = null;
  }
}

export async function refresh() {
  if (device.busy === 'connecting' || device.busy === 'scanning') return;
  device.busy = 'reading';
  clearError();
  try {
    if (device.active === 'keyboard' && device.keyboard) {
      device.keyboard = await api.refreshKeyboard();
      showToast('已从键盘重新读取');
    } else if (device.mouse) {
      const snap = await api.refresh();
      device.mouse = snap;
      device.current = snap;
      showToast('已从鼠标重新读取');
    }
  } catch (e) {
    setError(e, '重新读取失败');
    showToast('重新读取失败');
  } finally {
    device.busy = null;
  }
}

export async function pollLive() {
  if (device.busy) return;
  try {
    if (device.active === 'keyboard' && device.keyboard) {
      const live = await api.keyboardLive();
      device.keyboard.online = live.online;
      device.keyboard.battery = live.battery;
    } else if (device.mouse) {
      const live = await api.liveStatus();
      device.mouse.online = live.online;
      device.mouse.battery = live.battery;
    }
  } catch {
    /* 轮询失败保持安静 */
  }
}

export async function disconnect(kind?: DeviceKind) {
  try {
    await api.disconnect(kind);
  } catch {
    /* 关连接失败也复位界面 */
  }
  const target = kind ?? device.active;
  if (target === 'keyboard') {
    device.keyboard = null;
  } else {
    device.mouse = null;
    device.current = null;
  }
  syncActive();
  await scan(false);
  showToast(target === 'keyboard' ? '已断开键盘' : '已断开鼠标');
}

export async function setShowUnknown(show: boolean) {
  prefs.showUnknown = show;
  persistPrefs();
  await scan(false);
}

// ---------------------------------------------------------------------------
// 鼠标：改动即时写入
// ---------------------------------------------------------------------------

function mouse(): Snapshot | null {
  return device.mouse;
}

export function canEdit(): boolean {
  return !!device.mouse?.online && !!device.mouse?.config;
}

function scheduleMouseSave() {
  if (!device.mouse) return;
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => void flushMouse(), 320);
}

async function flushMouse() {
  saveTimer = null;
  const snap = device.mouse;
  if (!snap?.config) return;
  const config: MouseConfig = cloneConfig(snap.config);
  const buttons: ButtonAction[] = snap.buttons ? [...snap.buttons] : [];
  beginSave();
  try {
    const next = await api.applyConfig(config, buttons);
    if (device.mouse) {
      device.mouse.info = next.info;
      device.mouse.online = next.online;
      device.mouse.battery = next.battery;
      device.mouse.readAt = next.readAt;
      // 写入期间用户可能又改了别的值，只在没有新改动时采用回读值。
      if (device.mouse.config && sameConfig(device.mouse.config, config)) {
        device.mouse.config = next.config;
        device.mouse.buttons = next.buttons;
      }
    }
    clearError();
    endSave();
  } catch (e) {
    setError(e, '写入鼠标失败');
    device.saveState = 'error';
    savesInFlight = 0;
    showToast('写入鼠标失败');
  }
}

function sameConfig(a: MouseConfig, b: MouseConfig): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

export function setStageCount(n: number) {
  const m = mouse();
  const c = m?.info.capabilities;
  if (!m?.config || !c?.dpi) return;
  m.config.stageCount = clamp(n, 1, c.maxStages);
  if (m.config.activeStage > m.config.stageCount) m.config.activeStage = m.config.stageCount;
  scheduleMouseSave();
}

export function setActiveStage(n: number) {
  const m = mouse();
  if (!m?.config) return;
  m.config.activeStage = clamp(n, 1, m.config.stageCount);
  scheduleMouseSave();
}

export function setDpi(index: number, value: number) {
  const m = mouse();
  const c = m?.info.capabilities;
  if (!m?.config || !c?.dpi) return;
  const { min, max, step } = c.dpiRange;
  const snapped = Math.round(value / step) * step;
  m.config.stages[index] = clamp(snapped, min, max);
  scheduleMouseSave();
}

export function nudgeDpi(index: number, delta: number) {
  const m = mouse();
  if (!m?.config) return;
  setDpi(index, m.config.stages[index] + delta);
}

export function setPolling(hz: number) {
  const m = mouse();
  if (!m?.config || !m.info.capabilities.polling) return;
  m.config.pollingHz = hz;
  scheduleMouseSave();
}

export function setDebounce(ms: number) {
  const m = mouse();
  if (!m?.config || !m.info.capabilities.debounce) return;
  m.config.debounceMs = clamp(Math.round(ms), 0, 20);
  scheduleMouseSave();
}

export function setSleep(minutes: number) {
  const m = mouse();
  if (!m?.config || !m.info.capabilities.sleep) return;
  m.config.sleepMinutes = clamp(Math.round(minutes), 0, 100);
  scheduleMouseSave();
}

export function setLod(lod: 'low' | 'high') {
  const m = mouse();
  if (!m?.config || !m.info.capabilities.lod) return;
  m.config.lod = lod;
  scheduleMouseSave();
}

export function setScrollReversed(reversed: boolean) {
  const m = mouse();
  if (!m?.config || !m.info.capabilities.scrollDirection) return;
  m.config.scrollReversed = reversed;
  scheduleMouseSave();
}

export function setButton(index: number, action: ButtonAction) {
  const m = mouse();
  if (!m?.buttons || index === 0 || !m.info.capabilities.buttons) return;
  m.buttons[index] = action;
  scheduleMouseSave();
}

export function restoreFactory() {
  const m = mouse();
  if (!m?.config) return;
  m.config = cloneConfig(m.factoryConfig);
  if (m.factoryButtons.length === 5) m.buttons = [...m.factoryButtons];
  scheduleMouseSave();
  showToast('已恢复默认值，正在写入设备');
}

// ---------------------------------------------------------------------------
// 键盘：改动即时写入
// ---------------------------------------------------------------------------

export function canEditKeyboard(): boolean {
  return !!device.keyboard?.online;
}

function scheduleKeyboardSave() {
  if (!device.keyboard) return;
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => void flushKeyboard(), 450);
}

function clonePerformance(p: KeyboardPerformance): KeyboardPerformance {
  return { ...p, color: [...p.color] as [number, number, number] };
}

function cloneLayers(
  layers: [number, number, number, number][][],
): [number, number, number, number][][] {
  return layers.map((layer) =>
    layer.map((c) => [...c] as [number, number, number, number]),
  );
}

async function flushKeyboard() {
  saveTimer = null;
  const kb = device.keyboard;
  if (!kb) return;
  const input: KeyboardInput = {
    performance: clonePerformance(kb.performance),
    layers: cloneLayers(kb.layers),
  };
  beginSave();
  try {
    const next = await api.applyKeyboard(input);
    if (device.keyboard) {
      device.keyboard.info = next.info;
      device.keyboard.online = next.online;
      device.keyboard.battery = next.battery;
      device.keyboard.readAt = next.readAt;
      // 回读的层/性能以设备为准；如果用户刚好又改了，下一轮再写入。
      if (JSON.stringify(device.keyboard.performance) === JSON.stringify(input.performance)) {
        device.keyboard.performance = next.performance;
      }
      if (JSON.stringify(device.keyboard.layers) === JSON.stringify(input.layers)) {
        device.keyboard.layers = next.layers;
      }
    }
    clearError();
    endSave();
  } catch (e) {
    setError(e, '写入键盘失败');
    device.saveState = 'error';
    savesInFlight = 0;
    showToast('写入键盘失败');
  }
}

export function setKeyboardPerformance(patch: Partial<KeyboardPerformance>) {
  const kb = device.keyboard;
  if (!kb) return;
  Object.assign(kb.performance, patch);
  scheduleKeyboardSave();
}

export function setLightMode(mode: number) {
  const kb = device.keyboard;
  if (!kb) return;
  kb.performance.lightMode = mode;
  scheduleKeyboardSave();
}

export function setLightColor(rgb: [number, number, number]) {
  const kb = device.keyboard;
  if (!kb) return;
  kb.performance.color = rgb;
  scheduleKeyboardSave();
}

export function setKey(layer: number, index: number, code: [number, number, number, number]) {
  const kb = device.keyboard;
  if (!kb?.layers[layer] || index < 0 || index >= kb.layers[layer].length) return;
  kb.layers[layer][index] = [...code] as [number, number, number, number];
  scheduleKeyboardSave();
}

export function restoreKeyboardLayer(layer: number, defaults: [number, number, number, number][]) {
  const kb = device.keyboard;
  if (!kb?.layers[layer]) return;
  kb.layers[layer] = defaults.map((c) => [...c] as [number, number, number, number]);
  scheduleKeyboardSave();
  showToast('已恢复本层默认按键，正在写入键盘');
}

// ---------------------------------------------------------------------------
// 诊断
// ---------------------------------------------------------------------------

export function diagnosticsText(): string {
  const lines = [
    'MHUB Lite 诊断信息',
    `版本: ${device.version}`,
    `平台: ${navigator.userAgent}`,
    `活动设备: ${device.active ?? '无'}`,
  ];
  const snap = device.mouse;
  if (snap) {
    const c = snap.config;
    lines.push(
      `鼠标: ${snap.info.productName} (${snap.info.model ?? '未知型号'})`,
      `VID:PID: ${snap.info.vendorId.toString(16).toUpperCase()}:${snap.info.productId.toString(16).toUpperCase()}`,
      `连接: ${connectionLabel(snap.info.connection)}`,
      `固件: ${snap.info.firmware ?? '未知'}`,
      `在线: ${snap.online ? '是' : '否'}`,
      `电量: ${snap.battery ? `${snap.battery.percent}%${snap.battery.charging ? '（充电中）' : ''}` : '未知'}`,
    );
    if (c) {
      lines.push(
        `回报率: ${formatHz(c.pollingHz)}`,
        `DPI 档位: ${c.stageCount} / 当前第 ${c.activeStage} 档`,
        `各档 DPI: ${c.stages.slice(0, c.stageCount).join(' / ')}`,
        `防抖: ${c.debounceMs} ms`,
        `休眠: ${formatMinutes(c.sleepMinutes)}`,
      );
    }
  }
  const kb = device.keyboard;
  if (kb) {
    lines.push(
      `键盘: ${kb.info.productName} (${kb.info.model ?? '未知型号'})`,
      `键盘电量: ${kb.battery ? `${kb.battery.percent}%${kb.battery.charging ? '（充电中）' : ''}` : '未知'}`,
      `回报率: ${formatHz(kb.performance.rateHz)}`,
      `灯效: ${kb.performance.lightMode} / 亮度 ${kb.performance.brightness}`,
      `休眠: ${formatMinutes(kb.performance.sleepMinutes)}`,
    );
  }
  return lines.join('\n');
}
