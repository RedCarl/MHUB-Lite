/** 极简 toast：写入成功、设备唤醒、复制成功这类轻反馈。 */

export const toast = $state({
  msg: '',
  visible: false,
});

let timer: ReturnType<typeof setTimeout> | null = null;

export function showToast(msg: string, ms = 2000) {
  toast.msg = msg;
  toast.visible = true;
  if (timer) clearTimeout(timer);
  timer = setTimeout(() => {
    toast.visible = false;
    timer = null;
  }, ms);
}

export async function copyText(text: string, label = '已复制到剪贴板') {
  const ok = await writeClipboard(text);
  showToast(ok ? label : '复制失败');
  return ok;
}

async function writeClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    try {
      const ta = document.createElement('textarea');
      ta.value = text;
      ta.style.position = 'fixed';
      ta.style.opacity = '0';
      ta.setAttribute('readonly', '');
      document.body.appendChild(ta);
      ta.select();
      const ok = document.execCommand('copy');
      ta.remove();
      return ok;
    } catch {
      return false;
    }
  }
}
