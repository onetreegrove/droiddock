import { describe, expect, it } from 'vitest';
import {
  buildScrcpyArgs,
  buildScrcpyCommand,
  optionSummaryTagsFromArgs,
  clearUndefinedOptions,
  defaultScrcpyOptions,
  mergeScrcpyOptions,
  optionSummaryTags,
  presetOptions,
} from './scrcpyOptions';

describe('scrcpy option domain', () => {
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
    const args = buildScrcpyArgs('R9YT301WXXX', {
      maxSize: 1920,
      maxFps: 60,
      videoBitRate: '4M',
      videoCodec: 'h265',
      noAudio: true,
      stayAwake: true,
      alwaysOnTop: true,
    });

    expect(args).toEqual([
      '-s',
      'R9YT301WXXX',
      '--max-size=1920',
      '--max-fps=60',
      '--video-bit-rate=4M',
      '--video-codec=h265',
      '--no-audio',
      '--stay-awake',
      '--always-on-top',
    ]);
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
});
