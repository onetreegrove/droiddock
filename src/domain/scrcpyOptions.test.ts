import { describe, expect, it } from 'vitest';
import {
  buildScrcpyArgs,
  buildScrcpyCommand,
  defaultScrcpyCapabilities,
  backgroundColorErrorMessage,
  optionSummaryTagsFromArgs,
  clearUndefinedOptions,
  defaultScrcpyOptions,
  mergeScrcpyOptions,
  normalizeBackgroundColor,
  nextWindowAspectRatioLockValue,
  optionSummaryTags,
  presetOptions,
} from './scrcpyOptions';

describe('scrcpy option domain', () => {
  const consistencyFixture = {
    serial: 'R9YT301WXXX',
    options: {
      maxSize: 1920,
      maxFps: 60,
      videoBitRate: '4M',
      videoCodec: 'h265' as const,
      noAudio: true,
      stayAwake: true,
      alwaysOnTop: true,
    },
    args: [
      '-s',
      'R9YT301WXXX',
      '--max-size=1920',
      '--max-fps=60',
      '--video-bit-rate=4M',
      '--video-codec=h265',
      '--no-audio',
      '--stay-awake',
      '--always-on-top',
    ],
  };

  it('merges global, device, and session options while preserving explicit false', () => {
    const result = mergeScrcpyOptions(
      { ...defaultScrcpyOptions, noAudio: true, stayAwake: true },
      { maxFps: 30, noAudio: false },
      { videoBitRate: '2M' },
    );

    expect(result).toEqual({
      ...defaultScrcpyOptions,
      noAudio: false,
      stayAwake: true,
      maxFps: 30,
      videoBitRate: '2M',
    });
  });

  it('removes undefined values without dropping false values', () => {
    expect(clearUndefinedOptions({ noAudio: false, maxFps: undefined, videoBitRate: '' })).toEqual({
      noAudio: false,
      videoBitRate: '',
    });
  });

  it('builds scrcpy args in stable UI order', () => {
    expect(buildScrcpyArgs(consistencyFixture.serial, consistencyFixture.options)).toEqual(consistencyFixture.args);
  });

  it('normalizes supported scrcpy 4 color formats', () => {
    expect(normalizeBackgroundColor('#abc')).toBe('#aabbcc');
    expect(normalizeBackgroundColor('567')).toBe('#556677');
    expect(normalizeBackgroundColor('#AABBCC')).toBe('#aabbcc');
    expect(normalizeBackgroundColor('234567')).toBe('#234567');
  });

  it('rejects unsupported background color formats', () => {
    expect(normalizeBackgroundColor('red')).toBeNull();
    expect(normalizeBackgroundColor('#12')).toBeNull();
    expect(normalizeBackgroundColor('#abcd')).toBeNull();
    expect(normalizeBackgroundColor('#12345g')).toBeNull();
  });

  it('describes invalid background color input before launch', () => {
    expect(backgroundColorErrorMessage('red')).toBe('背景色格式不正确，请使用 #RGB 或 #RRGGBB。');
    expect(backgroundColorErrorMessage('#234567')).toBeNull();
    expect(backgroundColorErrorMessage('')).toBeNull();
  });

  it('toggles window aspect ratio lock from the displayed value', () => {
    expect(nextWindowAspectRatioLockValue(undefined)).toBe(false);
    expect(nextWindowAspectRatioLockValue(true)).toBe(false);
    expect(nextWindowAspectRatioLockValue(false)).toBe(true);
  });

  it('builds scrcpy 4 args only when capabilities allow them', () => {
    expect(
      buildScrcpyArgs(
        'SERIAL',
        {
          keepActive: true,
          backgroundColor: '567',
          windowAspectRatioLock: false,
        },
        {
          supportsKeepActive: true,
          supportsBackgroundColor: true,
          supportsWindowAspectRatioLock: true,
        },
      ),
    ).toEqual(['-s', 'SERIAL', '--keep-active', '--background-color=#556677', '--no-window-aspect-ratio-lock']);

    expect(
      buildScrcpyArgs(
        'SERIAL',
        {
          keepActive: true,
          backgroundColor: '567',
          windowAspectRatioLock: false,
        },
        defaultScrcpyCapabilities,
      ),
    ).toEqual(['-s', 'SERIAL']);
  });

  it('skips empty bit rate and default codec in command preview', () => {
    expect(buildScrcpyCommand('SERIAL', { maxSize: 1024, videoBitRate: '', videoCodec: 'default' })).toBe(
      'scrcpy -s SERIAL --max-size=1024',
    );
  });

  it('exposes PRD presets and summary tags', () => {
    expect(presetOptions.lowBandwidth).toEqual({
      maxSize: 1024,
      videoBitRate: '2M',
      maxFps: 30,
      noAudio: true,
    });
    expect(optionSummaryTags({ maxSize: 1024, maxFps: 30, videoBitRate: '2M', noAudio: true })).toEqual([
      '1024p',
      '30fps',
      '2M',
      'no-audio',
    ]);
  });

  it('summarizes scrcpy 4 options', () => {
    expect(optionSummaryTags({ keepActive: true, backgroundColor: '#234567', windowAspectRatioLock: false })).toEqual([
      '保持活跃',
      '背景 #234567',
      '自由缩放窗口',
    ]);
  });

  it('summarizes the actual session args instead of recomputing current defaults', () => {
    expect(
      optionSummaryTagsFromArgs([
        '-s',
        'R9YT301WXXX',
        '--max-size=1024',
        '--max-fps=30',
        '--video-bit-rate=2M',
        '--no-control',
      ]),
    ).toEqual(['1024p', '30fps', '2M', 'no-control']);
  });

  it('summarizes scrcpy 4 args from running sessions', () => {
    expect(
      optionSummaryTagsFromArgs([
        '-s',
        'R9YT301WXXX',
        '--keep-active',
        '--background-color=#234567',
        '--no-window-aspect-ratio-lock',
      ]),
    ).toEqual(['保持活跃', '背景 #234567', '自由缩放窗口']);
  });
});
