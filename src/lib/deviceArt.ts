import a7V3UltraPlusWhite from '../assets/devices/a7-v3-ultra-plus-white.webp';
import g3V2White from '../assets/devices/g3-v2-white.webp';
import k99V2Mount from '../assets/devices/k99-v2-mount.webp';

/**
 * 官方产品渲染图。
 *
 * 来自 MCHOSE 官方网页驱动（configCenter）的产品资源，仅用于在本地识别设备；
 * 商标与图片版权归 MCHOSE 所有。没有对应型号时返回 null，页面回退到内置 SVG。
 */
export function mouseArt(model?: string | null, productName?: string | null): string | null {
  const text = `${model ?? ''} ${productName ?? ''}`.toLowerCase();
  if (text.includes('a7 v3') || text.includes('a7 v3 ultra')) return a7V3UltraPlusWhite;
  if (text.includes('g3 v2')) return g3V2White;
  return null;
}

export function keyboardArt(model?: string | null, productName?: string | null): string | null {
  const text = `${model ?? ''} ${productName ?? ''}`.toLowerCase();
  if (text.includes('k99 v2') || text.includes('k99v2')) return k99V2Mount;
  return null;
}
