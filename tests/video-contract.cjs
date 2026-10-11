const assert = require('node:assert/strict');
const video = require('../frontend/video-settings.js');

const defaults = video.normalizeOptions();
assert.equal(defaults.encoder, 'auto');
assert.equal(defaults.container, 'mp4');
assert.equal(defaults.resolution, 0);
assert.equal(defaults.frameRate, null, 'source frame timing stays unchanged by default');
assert.equal(defaults.preset, 'medium', 'encoder speed preset is separate from the named quality profile');
assert.equal(defaults.audioPolicy, 'copy');
assert.equal(defaults.allowLargerOutput, false);
assert.equal(Object.hasOwn(defaults, 'quality'), false, 'video settings do not inherit image quality');

const capabilities = {
  backend: 'ffmpeg',
  available: true,
  encoders: [
    {id: 'libx264', label: 'H.264 software', modes: ['crf', 'abr', 'twoPass']},
    {id: 'hevc_videotoolbox', label: 'HEVC hardware', modes: ['abr']},
  ],
};
const sdr = {
  durationUs: 120000000,
  sizeBytes: 50 * 1024 * 1024,
  streams: [{codecType: 'video', codecName: 'h264', width: 1920, height: 1080, avgFrameRate: '30000/1001'}],
};

assert.equal(video.validateOptions(defaults, sdr, capabilities).valid, true,
  'auto mode may use a runtime software encoder that exposes CRF');
const validX264 = video.validateOptions({...defaults, encoder: 'libx264', mode: 'crf'}, sdr, capabilities);
assert.equal(validX264.valid, true);
assert.equal(video.validateOptions({...defaults, encoder: 'hevc_videotoolbox', mode: 'crf'}, sdr, capabilities).valid, false,
  'hardware VideoToolbox does not inherit software CRF modes');
assert.equal(video.validateOptions(defaults, sdr, {
  available: true, encoders: [{id: 'hevc_videotoolbox', modes: ['abr']}],
}).valid, false, 'auto mode cannot expose CRF if every runtime encoder lacks it');
assert.match(video.validateOptions({...defaults, encoder: 'libx264', container: 'webm'}, sdr, capabilities).errors.join(' '), /WebM/);
assert.match(video.validateOptions({...defaults, encoder: 'libx264', audioPolicy: 'opus'}, sdr, capabilities).errors.join(' '), /Opus/);

const hdr = {...sdr, hdrKind: 'dolbyVision'};
assert.match(video.validateOptions({...defaults, encoder: 'libx264'}, hdr, capabilities).errors.join(' '), /HDR/,
  'unknown HDR preservation must stop the job before encoding');
assert.match(video.validateOptions(defaults, sdr, {backend: 'unavailable', available: false, reason: 'App Store 尚未支持视频编码'}).errors.join(' '), /尚未支持/);

assert.equal(video.videoSummary(sdr).dimensions, '1920 × 1080');
assert.equal(video.videoSummary(sdr).frameRate, '29.97 fps');
assert.equal(video.formatDurationUs(3661000000), '1:01:01');
assert.equal(video.estimateOutputBytes(sdr, {...defaults, mode: 'crf'}), null,
  'CRF output estimates stay unknown instead of claiming a compression ratio');
assert.equal(video.estimateOutputBytes(sdr, {...defaults, mode: 'targetSize', targetMiB: 24}).bytes, 24 * 1024 * 1024);

const larger = video.normalizeResult({success: false, sourcePath: '/movie.mov', outputPath: null, originalBytes: 100, outputBytes: 110, reason: 'skippedLarger'}, '/movie.mov', sdr, defaults);
assert.equal(larger.skippedLarger, true);
assert.equal(larger.success, false);
assert.equal(larger.savings, -10);
console.log('PASS: video options honor runtime capabilities, source timing, HDR safeguards, and honest size results');
