import { mount } from 'svelte';
import { getCurrentWindow } from '@tauri-apps/api/window';
import App from './App.svelte';
import StatusPanel from './StatusPanel.svelte';
import './app.css';
import { initTheme } from './lib/theme.svelte';

/**
 * 窗口外观按系统分开处理（和 TinyMusic 同一套判定）：
 * - macOS：Tauri 用 Overlay 标题栏，红绿灯浮在侧边栏顶部，需要留 inset
 * - Windows：关掉原生边框，顶部换成应用自绘的标题栏
 * - 浏览器：按 web 处理，既不画标题栏，也不留 macOS 的空白
 */
function detectShell(): { os: string; shell: 'tauri' | 'web' } {
  const tauri = '__TAURI_INTERNALS__' in window;
  const ua = navigator.userAgent;
  const os = /Macintosh|Mac OS X/i.test(ua)
    ? 'macos'
    : /Windows/i.test(ua)
      ? 'windows'
      : /Linux|X11/i.test(ua)
        ? 'linux'
        : 'other';
  return { os, shell: tauri ? 'tauri' : 'web' };
}

const { os, shell } = detectShell();
document.documentElement.dataset.os = os;
document.documentElement.dataset.shell = shell;
document.documentElement.dataset.chrome =
  shell === 'tauri' && os === 'windows' ? 'custom' : 'native';

initTheme();

const target = document.getElementById('app')!;

/** 菜单栏电量面板是第二个 WebView，用窗口 label 区分。 */
const isStatusPanel = shell === 'tauri' && getCurrentWindow().label === 'status';

if (isStatusPanel) {
  // 面板窗口本身透明，圆角交给 StatusPanel 自己画。
  for (const node of [document.documentElement, document.body, target]) {
    node.style.background = 'transparent';
  }
  document.body.style.overflow = 'hidden';
}

const app = isStatusPanel
  ? mount(StatusPanel, { target })
  : mount(App, { target });

export default app;
