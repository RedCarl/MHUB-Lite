import { prefs, persistPrefs, type ThemeMode } from './prefs.svelte';

export type { ThemeMode };
export type ResolvedTheme = 'light' | 'dark';

const media = window.matchMedia('(prefers-color-scheme: dark)');

export const theme = $state<{ mode: ThemeMode; resolved: ResolvedTheme }>({
  mode: prefs.theme,
  resolved: media.matches ? 'dark' : 'light',
});

function resolve(): ResolvedTheme {
  if (theme.mode === 'auto') return media.matches ? 'dark' : 'light';
  return theme.mode;
}

function apply() {
  theme.resolved = resolve();
  document.documentElement.dataset.theme = theme.resolved;
  document.documentElement.style.colorScheme = theme.resolved;
}

export function setTheme(mode: ThemeMode) {
  theme.mode = mode;
  prefs.theme = mode;
  persistPrefs();
  apply();
}

/** 在挂载前调用，避免首帧闪一下深色。 */
export function initTheme() {
  media.addEventListener('change', () => {
    if (theme.mode === 'auto') apply();
  });
  apply();
}
