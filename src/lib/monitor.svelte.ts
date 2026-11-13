/**
 * 后台常驻与电量提醒的前端状态。
 *
 * 真正的读取和系统通知都在 Rust 侧；这里只做三件事：
 * 显示设置、把修改写回去、在窗口打开时同步电量与提醒事件。
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { api, isTauri, DEFAULT_MONITOR_SETTINGS } from './api';
import type { BatteryAlert, MonitorConfig, MonitorReading, MonitorSettings, MonitorStatus } from './api';
import { showToast } from './toast.svelte';

const emptyStatus: MonitorStatus = { mouse: null, keyboard: null, readAt: 0 };

export const monitor = $state<{
  settings: MonitorSettings;
  autostart: boolean;
  status: MonitorStatus;
  ready: boolean;
  saving: boolean;
}>({
  settings: { ...DEFAULT_MONITOR_SETTINGS },
  autostart: false,
  status: emptyStatus,
  ready: false,
  saving: false,
});

let started = false;
let unlisteners: UnlistenFn[] = [];

function applyConfig(config: MonitorConfig) {
  monitor.settings = config.settings;
  monitor.autostart = config.autostart;
}

function errorText(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

export async function initMonitor() {
  if (started) return;
  started = true;

  if (isTauri) {
    try {
      unlisteners.push(
        await listen<MonitorStatus>('monitor-reading', (event) => {
          monitor.status = event.payload;
        }),
      );
      unlisteners.push(
        await listen<MonitorConfig>('monitor-settings', (event) => {
          applyConfig(event.payload);
        }),
      );
      unlisteners.push(
        await listen<BatteryAlert>('monitor-alert', (event) => {
          const alert = event.payload;
          if (alert.notified) {
            showToast(
              `${alert.name} 电量 ${alert.percent}%，已低于 ${alert.threshold}%。请及时充电。`,
              6000,
            );
          } else {
            showToast(
              `${alert.name} 电量 ${alert.percent}%，但系统通知没有发出来：${
                alert.message ?? '请检查系统的通知权限'
              }`,
              6000,
            );
          }
        }),
      );
    } catch (e) {
      // 事件订阅失败不影响设置修改，只静默记录。
      console.warn('订阅电量事件失败', e);
    }
  }

  try {
    applyConfig(await api.monitorConfig());
    monitor.status = await api.monitorStatus();
  } catch {
    /* 后台能力不可用时保持默认设置，界面照常可用 */
  } finally {
    monitor.ready = true;
  }
}

/** 修改任意一项设置并写回 Rust；失败时重新拉取，避免界面和实际状态不一致。 */
export async function updateMonitorSettings(patch: Partial<MonitorSettings>) {
  if (monitor.saving) return;
  const next: MonitorSettings = { ...monitor.settings, ...patch };
  monitor.saving = true;
  try {
    applyConfig(await api.setMonitorSettings(next));
  } catch (e) {
    showToast(`后台设置没有生效：${errorText(e)}`, 4000);
    try {
      applyConfig(await api.monitorConfig());
    } catch {
      /* 保持当前界面状态 */
    }
  } finally {
    monitor.saving = false;
  }
}

export async function testMonitorNotification() {
  try {
    await api.testBatteryNotification();
    showToast('测试提醒已发送；如果没看到，请在系统设置里允许 MHUB Lite 发通知', 5000);
  } catch (e) {
    showToast(`测试提醒发送失败：${errorText(e)}`, 5000);
  }
}

export async function quitApplication() {
  try {
    await api.quitApp();
  } catch (e) {
    showToast(`退出失败：${errorText(e)}`, 4000);
  }
}

/** 停止事件订阅（正常运行时不需要调用，留给以后窗口重建用）。 */
export function disposeMonitor() {
  for (const unlisten of unlisteners) unlisten();
  unlisteners = [];
  started = false;
}

export function readingText(reading: MonitorReading | null): string {
  if (!reading) return '未连接';
  if (!reading.online) return '已休眠';
  if (!reading.battery) return '电量未知';
  return reading.battery.charging
    ? `${reading.battery.percent}%（充电中）`
    : `${reading.battery.percent}%`;
}

/** 设置页「当前状态」那一行用。 */
export function monitorStatusText(): string {
  if (!monitor.settings.enabled) return '监测已关闭';
  const parts: string[] = [];
  if (monitor.status.mouse) parts.push(`鼠标 ${readingText(monitor.status.mouse)}`);
  if (monitor.status.keyboard) parts.push(`键盘 ${readingText(monitor.status.keyboard)}`);
  if (!parts.length) return monitor.ready ? '没有检测到设备' : '正在读取…';
  return parts.join(' · ');
}
