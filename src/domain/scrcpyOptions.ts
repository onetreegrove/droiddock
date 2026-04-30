import type { PresetId, ScrcpyOptions } from '../types/app';

export const defaultScrcpyOptions: ScrcpyOptions = {
  maxSize: 1920,
  maxFps: 60,
  videoCodec: 'default',
  noAudio: true,
  stayAwake: true,
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

export function buildScrcpyArgs(serial: string, options: ScrcpyOptions): string[] {
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

  return args;
}

export function buildScrcpyCommand(serial: string, options: ScrcpyOptions): string {
  return ['scrcpy', ...buildScrcpyArgs(serial, options)].join(' ');
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

  return tags;
}
