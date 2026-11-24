/** macOS「输入监控」权限：检测到 HID 被拒时，给用户一个直达系统设置的入口。 */

import { api, isMacOS, isTauri } from './api';
import { showToast } from './toast.svelte';

export const canOpenInputMonitoring = isTauri && isMacOS;

export async function openInputMonitoringSettings() {
  try {
    await api.openInputMonitoringSettings();
    showToast('已打开「输入监控」设置；勾选 MHUB Lite 后点「重启应用」', 6000);
  } catch (e) {
    showToast(e instanceof Error ? e.message : String(e), 6000);
  }
}
