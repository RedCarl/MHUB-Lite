import { invoke } from '@tauri-apps/api/core';

export type ConnectionKind = 'wired' | 'wireless' | 'unknown';
export type Lod = 'low' | 'high';
export type ButtonAction =
  | 'left'
  | 'right'
  | 'middle'
  | 'forward'
  | 'backward'
  | 'disabled'
  | `custom:${string}`;

export type ErrorCode =
  | 'notFound'
  | 'permission'
  | 'offline'
  | 'timeout'
  | 'protocol'
  | 'unsupported'
  | 'hid'
  | 'busy'
  | 'invalid';

export type DpiRange = { min: number; max: number; step: number };

export type Capabilities = {
  dpi: boolean;
  polling: boolean;
  debounce: boolean;
  sleep: boolean;
  lod: boolean;
  buttons: boolean;
  scrollDirection: boolean;
  battery: boolean;
  firmware: boolean;
  dpiRange: DpiRange;
  pollingRates: number[];
  maxStages: number;
};

export type MouseConfig = {
  pollingHz: number;
  stageCount: number;
  activeStage: number;
  stages: number[];
  lod: Lod;
  debounceMs: number;
  sleepMinutes: number;
  scrollReversed: boolean;
  sensorFlags: number;
  highspeedMode: number;
  sensorAngle: number;
};

export type Battery = { percent: number; charging: boolean };

export type DeviceInfo = {
  key: string;
  model: string | null;
  productName: string;
  vendorId: number;
  productId: number;
  serial: string | null;
  connection: ConnectionKind;
  supported: boolean;
  capabilities: Capabilities;
  firmware: string | null;
  buildDate: string | null;
  isDemo: boolean;
  note: string | null;
};

export type DeviceSummary = {
  key: string;
  name: string;
  model: string | null;
  serial: string | null;
  vendorId: number;
  productId: number;
  connection: ConnectionKind;
  supported: boolean;
  statusNote: string | null;
  isDemo: boolean;
};

export type LiveStatus = { online: boolean; battery: Battery | null };

export type Snapshot = {
  info: DeviceInfo;
  online: boolean;
  battery: Battery | null;
  config: MouseConfig | null;
  buttons: ButtonAction[] | null;
  factoryConfig: MouseConfig;
  factoryButtons: ButtonAction[];
  readAt: number;
};

export class ApiFailure extends Error {
  code: ErrorCode;
  constructor(code: ErrorCode, message: string) {
    super(message);
    this.name = 'ApiFailure';
    this.code = code;
  }
}

export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export const FACTORY_CONFIG: MouseConfig = {
  pollingHz: 250,
  stageCount: 6,
  activeStage: 3,
  stages: [400, 800, 1600, 3200, 6400, 12000],
  lod: 'high',
  debounceMs: 8,
  sleepMinutes: 3,
  scrollReversed: false,
  sensorFlags: 0,
  highspeedMode: 0,
  sensorAngle: 0,
};

export const FACTORY_BUTTONS: ButtonAction[] = ['left', 'right', 'middle', 'forward', 'backward'];

export const DEMO_CAPABILITIES: Capabilities = {
  dpi: true,
  polling: true,
  debounce: true,
  sleep: true,
  lod: true,
  buttons: true,
  scrollDirection: true,
  battery: true,
  firmware: true,
  dpiRange: { min: 100, max: 12000, step: 100 },
  pollingRates: [125, 250, 500, 1000],
  maxStages: 6,
};

export function cloneConfig(c: MouseConfig): MouseConfig {
  return { ...c, stages: [...c.stages] };
}

function normalizeError(e: unknown): ApiFailure {
  if (e instanceof ApiFailure) return e;
  if (typeof e === 'object' && e !== null && 'message' in e) {
    const obj = e as { code?: ErrorCode; message?: unknown };
    return new ApiFailure(obj.code ?? 'hid', String(obj.message ?? e));
  }
  if (typeof e === 'string') {
    // 某些 Tauri 版本会把 Err 序列化成 JSON 字符串再抛回来
    try {
      const parsed = JSON.parse(e) as { code?: ErrorCode; message?: unknown };
      if (parsed && typeof parsed === 'object' && parsed.message) {
        return new ApiFailure(parsed.code ?? 'hid', String(parsed.message));
      }
    } catch {
      /* 不是 JSON：按普通字符串处理 */
    }
    return new ApiFailure('hid', e);
  }
  return new ApiFailure('hid', String(e));
}

// ---------------------------------------------------------------------------
// 浏览器演示实现
//
// `pnpm dev` 在浏览器里打开时没有 Tauri，也没有 HID。保持一套同形状的
// 内存设备，界面、校验和写入流程都能照常走通（方便调样式和做截图）。
// ---------------------------------------------------------------------------

let mockConfig: MouseConfig = cloneConfig(FACTORY_CONFIG);
let mockButtons: ButtonAction[] = [...FACTORY_BUTTONS];
let mockBattery: Battery = { percent: 82, charging: false };
let mockOnline = true;
let mockConnected = false;

function mockInfo(): DeviceInfo {
  return {
    key: 'demo',
    model: 'mchose-g3-v2',
    productName: 'MCHOSE G3 V2',
    vendorId: 0x3837,
    productId: 0x4245,
    serial: 'DEMO-0001',
    connection: 'wireless',
    supported: true,
    capabilities: DEMO_CAPABILITIES,
    firmware: 'MHUB-DEMO 0.1.0',
    buildDate: '202611',
    isDemo: true,
    note: '浏览器演示数据：写入只作用于这个页面，不会碰真实硬件。',
  };
}

function mockSnapshot(): Snapshot {
  return {
    info: mockInfo(),
    online: mockOnline,
    battery: mockOnline ? { ...mockBattery } : null,
    config: mockOnline ? cloneConfig(mockConfig) : null,
    buttons: mockOnline ? [...mockButtons] : null,
    factoryConfig: cloneConfig(FACTORY_CONFIG),
    factoryButtons: [...FACTORY_BUTTONS],
    readAt: Math.floor(Date.now() / 1000),
  };
}

function mockValidate(config: MouseConfig) {
  if (config.stageCount < 1 || config.stageCount > 6) {
    throw new ApiFailure('invalid', '启用的 DPI 档位数量必须在 1–6 之间');
  }
  if (config.activeStage < 1 || config.activeStage > config.stageCount) {
    throw new ApiFailure('invalid', '当前档位必须落在已启用的档位范围内');
  }
  for (let i = 0; i < config.stageCount; i++) {
    const v = config.stages[i];
    if (v < 100 || v > 12000 || v % 100 !== 0) {
      throw new ApiFailure('invalid', `第 ${i + 1} 档 DPI 必须是 100–12000 之间、以 100 为步进的值`);
    }
  }
  if (![125, 250, 500, 1000].includes(config.pollingHz)) {
    throw new ApiFailure('invalid', '不支持的回报率');
  }
  if (config.debounceMs < 0 || config.debounceMs > 20) {
    throw new ApiFailure('invalid', '防抖时间必须在 0–20 ms 之间');
  }
  if (config.sleepMinutes < 0 || config.sleepMinutes > 100) {
    throw new ApiFailure('invalid', '休眠时间必须在 0–100 分钟之间');
  }
}

async function mockCall<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  await new Promise((r) => setTimeout(r, cmd === 'connect' || cmd === 'refresh' ? 260 : 60));
  switch (cmd) {
    case 'scan_devices':
      return [
        {
          key: 'demo',
          name: 'MCHOSE G3 V2',
          model: 'mchose-g3-v2',
          serial: 'DEMO-0001',
          vendorId: 0x3837,
          productId: 0x4245,
          connection: 'wireless',
          supported: true,
          statusNote: '浏览器演示数据',
          isDemo: true,
        },
      ] as T;
    case 'connect':
      mockConnected = true;
      mockOnline = true;
      return mockSnapshot() as T;
    case 'disconnect':
      mockConnected = false;
      return undefined as T;
    case 'refresh':
      if (!mockConnected) throw new ApiFailure('offline', '还没有连接设备');
      return mockSnapshot() as T;
    case 'live_status':
      if (!mockConnected) throw new ApiFailure('offline', '还没有连接设备');
      return { online: mockOnline, battery: mockOnline ? { ...mockBattery } : null } as T;
    case 'apply_config': {
      if (!mockConnected) throw new ApiFailure('offline', '还没有连接设备');
      const config = args?.config as MouseConfig;
      const buttons = args?.buttons as ButtonAction[];
      mockValidate(config);
      if (buttons.length !== 5) throw new ApiFailure('invalid', '按键映射必须正好 5 项');
      mockConfig = cloneConfig(config);
      mockButtons = [...buttons];
      return mockSnapshot() as T;
    }
    case 'set_demo_mode':
      return undefined as T;
    case 'app_version':
      return '0.1.0' as T;
    default:
      throw new ApiFailure('unsupported', `演示模式没有实现命令 ${cmd}`);
  }
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) return mockCall<T>(cmd, args);
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    throw normalizeError(e);
  }
}

export const api = {
  scanDevices(includeUnknown = true) {
    return call<DeviceSummary[]>('scan_devices', { includeUnknown });
  },
  connect(key: string) {
    return call<Snapshot>('connect', { key });
  },
  disconnect() {
    return call<void>('disconnect');
  },
  refresh() {
    return call<Snapshot>('refresh');
  },
  liveStatus() {
    return call<LiveStatus>('live_status');
  },
  applyConfig(config: MouseConfig, buttons: ButtonAction[]) {
    return call<Snapshot>('apply_config', { config, buttons });
  },
  setDemoMode(enabled: boolean) {
    return call<void>('set_demo_mode', { enabled });
  },
  appVersion() {
    return call<string>('app_version');
  },
};
