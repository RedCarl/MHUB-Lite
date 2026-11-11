import type { ButtonAction, ConnectionKind, MouseConfig } from './api';

export const ACTION_LABELS: Record<string, string> = {
  left: '左键单击',
  right: '右键单击',
  middle: '中键单击',
  forward: '前进（Mouse 5）',
  backward: '后退（Mouse 4）',
  disabled: '禁用',
};

export function actionLabel(action: ButtonAction): string {
  if (action.startsWith('custom:')) return '自定义动作';
  return ACTION_LABELS[action] ?? action;
}

export function actionShortLabel(action: ButtonAction): string {
  if (action.startsWith('custom:')) return '自定义';
  return ACTION_LABELS[action]?.replace('单击', '') ?? action;
}

export const BUTTON_LABELS = ['左键', '右键', '中键', '前进键', '后退键'];

export function connectionLabel(c: ConnectionKind): string {
  if (c === 'wireless') return '2.4G 无线';
  if (c === 'wired') return 'USB 有线';
  return '连接方式未知';
}

export function hexId(v: number): string {
  return `0x${v.toString(16).toUpperCase().padStart(4, '0')}`;
}

export function formatMinutes(m: number): string {
  if (m === 0) return '永不休眠';
  return `${m} 分钟`;
}

export function timeAgo(unixSeconds: number): string {
  const diff = Math.max(0, Math.floor(Date.now() / 1000) - unixSeconds);
  if (diff < 5) return '刚刚';
  if (diff < 60) return `${diff} 秒前`;
  if (diff < 3600) return `${Math.floor(diff / 60)} 分钟前`;
  return `${Math.floor(diff / 3600)} 小时前`;
}

export function batteryTone(percent: number, charging: boolean): 'ok' | 'warn' | 'low' | 'charge' {
  if (charging) return 'charge';
  if (percent <= 10) return 'low';
  if (percent <= 30) return 'warn';
  return 'ok';
}

/** 当前档位的 DPI（用于概览卡片） */
export function activeDpi(cfg: MouseConfig | null): number | null {
  if (!cfg) return null;
  return cfg.stages[cfg.activeStage - 1] ?? null;
}

export function formatHz(hz: number): string {
  return hz >= 1000 ? `${hz / 1000} kHz` : `${hz} Hz`;
}

export function clamp(v: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, v));
}
