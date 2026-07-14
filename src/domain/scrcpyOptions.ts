import type { PresetId, ScrcpyCapabilities, ScrcpyOptions } from '../types/app';

export const defaultScrcpyCapabilities: ScrcpyCapabilities = {
  supportsKeepActive: false,
  supportsBackgroundColor: false,
  supportsWindowAspectRatioLock: false,
};

export const defaultScrcpyOptions: ScrcpyOptions = {
  maxSize: 1920,
  maxFps: 60,
  videoCodec: 'default',
  noAudio: true,
  keepActive: true,
  stayAwake: false,
  windowAspectRatioLock: true,
};

export const presetOptions: Record<PresetId, ScrcpyOptions> = {
  daily: { maxSize: 1920, maxFps: 60, noAudio: true, stayAwake: true },
  lowBandwidth: { maxSize: 1024, videoBitRate: '2M', maxFps: 30, noAudio: true },
  demo: { maxSize: 1920, maxFps: 60, showTouches: true, alwaysOnTop: true },
  batterySaver: { maxSize: 1920, maxFps: 60, noAudio: true, stayAwake: true, turnScreenOff: true },
  viewOnly: { maxSize: 1920, maxFps: 60, noControl: true },
};

export const presetLabels: Record<PresetId, string> = {
  daily: '日常使用',
  lowBandwidth: '低带宽无线',
  demo: '演示模式',
  batterySaver: '息屏省电',
  viewOnly: '只看不控',
};

export function clearUndefinedOptions(options: ScrcpyOptions): ScrcpyOptions {
  return Object.fromEntries(Object.entries(options).filter(([, value]) => value !== undefined)) as ScrcpyOptions;
}

export function mergeScrcpyOptions(
  globalOptions: ScrcpyOptions,
  deviceOptions?: ScrcpyOptions,
  sessionOptions?: ScrcpyOptions,
): ScrcpyOptions {
  return clearUndefinedOptions({
    ...globalOptions,
    ...(deviceOptions ?? {}),
    ...(sessionOptions ?? {}),
  });
}

export function normalizeBackgroundColor(value: string): string | null {
  const color = value.trim();
  const match = color.match(/^#?([0-9a-fA-F]{3}|[0-9a-fA-F]{6})$/);
  if (!match) return null;

  const hex = match[1].toLowerCase();
  if (hex.length === 3) {
    return `#${hex[0]}${hex[0]}${hex[1]}${hex[1]}${hex[2]}${hex[2]}`;
  }
  return `#${hex}`;
}

export function backgroundColorErrorMessage(value: string): string | null {
  if (!value.trim()) return null;
  return normalizeBackgroundColor(value) ? null : '背景色格式不正确，请使用 #RGB 或 #RRGGBB。';
}

export function nextWindowAspectRatioLockValue(value: boolean | undefined): boolean {
  return !(value ?? true);
}

export function buildScrcpyArgs(
  serial: string,
  options: ScrcpyOptions,
  capabilities: ScrcpyCapabilities = defaultScrcpyCapabilities,
): string[] {
  const args = ['-s', serial];

  if (options.maxSize !== undefined) args.push(`--max-size=${options.maxSize}`);
  if (options.maxFps !== undefined) args.push(`--max-fps=${options.maxFps}`);
  if (options.videoBitRate?.trim()) args.push(`--video-bit-rate=${options.videoBitRate.trim()}`);
  if (options.videoCodec && options.videoCodec !== 'default') args.push(`--video-codec=${options.videoCodec}`);
  if (options.noAudio) args.push('--no-audio');
  if (options.noControl) args.push('--no-control');
  if (options.stayAwake) args.push('--stay-awake');
  if (options.turnScreenOff) args.push('--turn-screen-off');
  if (options.showTouches) args.push('--show-touches');
  if (options.alwaysOnTop) args.push('--always-on-top');
  if (options.fullscreen) args.push('--fullscreen');
  if (options.keepActive && capabilities.supportsKeepActive) args.push('--keep-active');
  if (options.backgroundColor?.trim() && capabilities.supportsBackgroundColor) {
    const color = normalizeBackgroundColor(options.backgroundColor);
    if (color) args.push(`--background-color=${color}`);
  }
  if (options.windowAspectRatioLock === false && capabilities.supportsWindowAspectRatioLock) {
    args.push('--no-window-aspect-ratio-lock');
  }

  return args;
}

export function buildScrcpyCommand(
  serial: string,
  options: ScrcpyOptions,
  capabilities: ScrcpyCapabilities = defaultScrcpyCapabilities,
): string {
  return ['scrcpy', ...buildScrcpyArgs(serial, options, capabilities)].join(' ');
}

export function optionSummaryTags(options: ScrcpyOptions): string[] {
  const tags: string[] = [];

  if (options.maxSize !== undefined) tags.push(`${options.maxSize}p`);
  if (options.maxFps !== undefined) tags.push(`${options.maxFps}fps`);
  if (options.videoBitRate?.trim()) tags.push(options.videoBitRate.trim());
  if (options.videoCodec && options.videoCodec !== 'default') tags.push(options.videoCodec);
  if (options.noAudio) tags.push('no-audio');
  if (options.noControl) tags.push('no-control');
  if (options.stayAwake) tags.push('stay-awake');
  if (options.turnScreenOff) tags.push('screen-off');
  if (options.showTouches) tags.push('touches');
  if (options.alwaysOnTop) tags.push('top');
  if (options.fullscreen) tags.push('fullscreen');
  if (options.keepActive) tags.push('保持活跃');
  if (options.backgroundColor) {
    const color = normalizeBackgroundColor(options.backgroundColor);
    if (color) tags.push(`背景 ${color}`);
  }
  if (options.windowAspectRatioLock === false) tags.push('自由缩放窗口');

  return tags;
}

export function optionSummaryTagsFromArgs(args: string[]): string[] {
  const options: ScrcpyOptions = {};

  for (const arg of args) {
    if (arg.startsWith('--max-size=')) options.maxSize = Number(arg.slice('--max-size='.length));
    if (arg.startsWith('--max-fps=')) options.maxFps = Number(arg.slice('--max-fps='.length));
    if (arg.startsWith('--video-bit-rate=')) options.videoBitRate = arg.slice('--video-bit-rate='.length);
    if (arg.startsWith('--video-codec=')) options.videoCodec = arg.slice('--video-codec='.length) as ScrcpyOptions['videoCodec'];
    if (arg === '--no-audio') options.noAudio = true;
    if (arg === '--no-control') options.noControl = true;
    if (arg === '--stay-awake') options.stayAwake = true;
    if (arg === '--turn-screen-off') options.turnScreenOff = true;
    if (arg === '--show-touches') options.showTouches = true;
    if (arg === '--always-on-top') options.alwaysOnTop = true;
    if (arg === '--fullscreen') options.fullscreen = true;
    if (arg === '--keep-active') options.keepActive = true;
    if (arg.startsWith('--background-color=')) options.backgroundColor = arg.slice('--background-color='.length);
    if (arg === '--no-window-aspect-ratio-lock') options.windowAspectRatioLock = false;
  }

  return optionSummaryTags(options);
}
