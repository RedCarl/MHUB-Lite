/** 写在 localStorage 里的界面偏好；不涉及设备状态。 */

const KEY = 'mhub.prefs.v1';

export type ThemeMode = 'auto' | 'light' | 'dark';

type Stored = {
  showUnknown: boolean;
  pollSeconds: number;
  theme: ThemeMode;
};

const defaults: Stored = {
  showUnknown: true,
  pollSeconds: 5,
  theme: 'auto',
};

function load(): Stored {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return { ...defaults };
    const parsed = JSON.parse(raw) as Partial<Stored>;
    return {
      showUnknown: parsed.showUnknown !== false,
      pollSeconds: [0, 2, 5, 10, 30].includes(Number(parsed.pollSeconds))
        ? Number(parsed.pollSeconds)
        : defaults.pollSeconds,
      theme: parsed.theme === 'light' || parsed.theme === 'dark' ? parsed.theme : 'auto',
    };
  } catch {
    return { ...defaults };
  }
}

const initial = load();

export const prefs = $state<Stored>(initial);

export function persistPrefs() {
  try {
    localStorage.setItem(
      KEY,
      JSON.stringify({
        showUnknown: prefs.showUnknown,
        pollSeconds: prefs.pollSeconds,
        theme: prefs.theme,
      }),
    );
  } catch {
    /* 无痕模式 / 存储被禁：偏好只在本次运行里有效 */
  }
}
