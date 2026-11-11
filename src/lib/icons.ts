/**
 * 统一图标出口。
 *
 * 规则和 TinyMusic 一致：
 * - 全项目只用一套样式：免费 Stroke Rounded（@hugeicons/core-free-icons）
 * - 同义同图标：扫设备永远用 refresh，写入永远用 bolt，恢复默认永远用 refresh-dot
 * - 尺寸只用三档：16（行内）/ 18（导航与按钮）/ 40（空状态）
 */
import {
  Alert01Icon,
  ArrowLeft01Icon,
  ArrowRight01Icon,
  BatteryCharging01Icon,
  BatteryFullIcon,
  BatteryLowIcon,
  BoltIcon,
  Cancel01Icon,
  CheckIcon,
  ChipIcon,
  Clock01Icon,
  Copy01Icon,
  Cursor01Icon,
  GaugeIcon,
  InformationCircleIcon,
  KeyboardIcon,
  Loading03Icon,
  Mouse01Icon,
  MousePointerClickIcon,
  MousePointerIcon,
  MouseRightClick02Icon,
  MouseScroll01Icon,
  MouseScroll02Icon,
  PlugIcon,
  Refresh01Icon,
  RefreshDotIcon,
  Settings01Icon,
  SlidersHorizontalIcon,
  Target01Icon,
  UsbIcon,
  WirelessIcon,
  ZzzIcon,
  Moon02Icon,
  Sun01Icon,
} from '@hugeicons/core-free-icons';

export const icons = {
  // 导航
  overview: Mouse01Icon,
  dpi: Target01Icon,
  buttons: MousePointerClickIcon,
  performance: GaugeIcon,
  settings: Settings01Icon,
  // 设备
  mouse: Mouse01Icon,
  keyboard: KeyboardIcon,
  wired: UsbIcon,
  wireless: WirelessIcon,
  battery: BatteryFullIcon,
  batteryLow: BatteryLowIcon,
  batteryCharging: BatteryCharging01Icon,
  firmware: ChipIcon,
  sleep: ZzzIcon,
  debounce: Clock01Icon,
  scroll: MouseScroll02Icon,
  wheel: MouseScroll01Icon,
  plug: PlugIcon,
  // 按键
  buttonLeft: Cursor01Icon,
  buttonRight: MouseRightClick02Icon,
  buttonMiddle: MouseScroll01Icon,
  buttonForward: ArrowRight01Icon,
  buttonBackward: ArrowLeft01Icon,
  // 动作
  forward: ArrowRight01Icon,
  back: ArrowLeft01Icon,
  refresh: Refresh01Icon,
  reset: RefreshDotIcon,
  apply: BoltIcon,
  check: CheckIcon,
  alert: Alert01Icon,
  info: InformationCircleIcon,
  copy: Copy01Icon,
  close: Cancel01Icon,
  loading: Loading03Icon,
  sliders: SlidersHorizontalIcon,
  themeAuto: Sun01Icon,
  themeLight: Sun01Icon,
  themeDark: Moon02Icon,
} as const;

/** 尺寸三档，避免各处随意取值。 */
export const ICON_SIZE = {
  inline: 16,
  control: 18,
  empty: 40,
} as const;
