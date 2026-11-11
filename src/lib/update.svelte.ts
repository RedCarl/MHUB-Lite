import { check, type Update } from '@tauri-apps/plugin-updater';

/** 后台静默检查结果：只记版本号，设置页再拿真实句柄。 */
export const updateState = $state<{ version: string }>({ version: '' });

let handle: Update | null = null;
let inflight = false;
const isTauri = '__TAURI_INTERNALS__' in window;

export function pendingUpdate(): Update | null {
  return handle;
}

export function setPendingUpdate(u: Update | null) {
  handle = u;
  updateState.version = u?.version ?? '';
}

/** 静默检查；离线、浏览器预览、GitHub 不可达都当作没有更新。 */
export async function checkUpdatesSilently() {
  if (!isTauri || inflight) return;
  inflight = true;
  try {
    setPendingUpdate(await check());
  } catch {
    /* 下一次轮询再试 */
  } finally {
    inflight = false;
  }
}
