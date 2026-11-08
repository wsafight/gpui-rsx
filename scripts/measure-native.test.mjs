import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { setTimeout as delay } from 'node:timers/promises';
import path from 'node:path';
import { probe, validate } from './measure-native.mjs';

const metric = { count: 60, p50: 1, p95: 2 };
const report = { schema: 1, scenario: 'scroll-and-select', records: 1000,
  sample_frames: 60, max_rendered_rows: 12,
  draw_ms: metric, present_interval_ms: metric, dirty_to_present_ms: metric, frame_callback_interval_ms: metric };

test('native reports require real samples and bounded row construction', () => {
  assert.equal(validate(report, 1000), report);
  for (const invalid of [
    { ...report, records: 10_000 }, { ...report, max_rendered_rows: 1000 },
    { ...report, sample_frames: 0 }, { ...report, draw_ms: { ...metric, count: 0 } },
    { ...report, draw_ms: { ...metric, p95: Number.NaN } },
  ]) assert.throws(() => validate(invalid, 1000));
});

test('native probes collect reports and retain failure and timeout logs', async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'rsx-native-probe-'));
  const output = path.join(directory, 'report.json');
  try {
    const script = `require('node:fs').writeFileSync(process.env.RSX_NATIVE_REPORT, ${JSON.stringify(JSON.stringify(report))}); console.log('native done');`;
    const result = await probe({ command: process.execPath, args: ['-e', script], rows: 1000, output });
    assert.equal(result.records, 1000);
    assert.match(await readFile(`${output}.log`, 'utf8'), /native done/);
    await assert.rejects(probe({ command: process.execPath, args: ['-e', 'process.exit(2)'], rows: 1000, output }), /exited 2/);
    await assert.rejects(readFile(output), /ENOENT/);
    await assert.rejects(probe({ command: process.execPath, args: ['-e', 'setInterval(() => {}, 1000)'], rows: 1000, output, timeout: 100 }), /timed out/);
    await assert.rejects(probe({ command: path.join(directory, 'missing'), rows: 1000, output }), /ENOENT/);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test('every metric rejects missing, non-finite, negative and inconsistent samples', () => {
  for (const name of ['draw_ms', 'present_interval_ms', 'dirty_to_present_ms', 'frame_callback_interval_ms']) {
    for (const invalid of [
      undefined, null, {}, { ...metric, count: 0 }, { ...metric, count: -1 },
      { ...metric, count: 1.5 }, { ...metric, count: '60' },
      { ...metric, p50: -1 }, { ...metric, p50: Number.NaN },
      { ...metric, p50: Number.POSITIVE_INFINITY }, { ...metric, p50: '1' },
      { ...metric, p95: 0 }, { ...metric, p95: Number.NaN },
      { ...metric, p95: Number.POSITIVE_INFINITY }, { ...metric, p95: '2' },
    ]) assert.throws(() => validate({ ...report, [name]: invalid }, 1000), `${name}: ${JSON.stringify(invalid)}`);
  }
});

test('report identity and virtualization budgets use strict numeric boundaries', () => {
  for (const invalid of [null, {}, { ...report, schema: 2 }, { ...report, scenario: 'mock' },
    { ...report, records: '1000' }, { ...report, sample_frames: 59 }]) {
    assert.throws(() => validate(invalid, 1000));
  }
  for (const rows of [0, -1, 0.5, 100, Number.NaN, Number.POSITIVE_INFINITY, '12']) {
    assert.throws(() => validate({ ...report, max_rendered_rows: rows }, 1000));
  }
  for (const rows of [1, 99]) assert.equal(validate({ ...report, max_rendered_rows: rows }, 1000).max_rendered_rows, rows);
  assert.equal(validate({ ...report, draw_ms: { count: 1, p50: 0, p95: 0 } }, 1000).draw_ms.count, 1);
});

test('missing or malformed reports never reuse stale successful output', async t => {
  const directory = await mkdtemp(path.join(tmpdir(), 'rsx-native-invalid-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const output = path.join(directory, 'report.json');
  for (const contents of [undefined, '', '{broken', '{}', JSON.stringify({ ...report, records: 10_000 })]) {
    await writeFile(output, JSON.stringify(report));
    const script = contents === undefined ? 'console.error("missing report");' :
      `require('node:fs').writeFileSync(process.env.RSX_NATIVE_REPORT, ${JSON.stringify(contents)}); console.error('invalid report');`;
    await assert.rejects(probe({ command: process.execPath, args: ['-e', script], rows: 1000, output }));
    assert.match(await readFile(`${output}.log`, 'utf8'), /missing report|invalid report/);
    if (contents === undefined) await assert.rejects(readFile(output), { code: 'ENOENT' });
    else assert.equal(await readFile(output, 'utf8'), contents);
  }
});

test('successful exit cannot hide a failure status or override protected environment values', async t => {
  const directory = await mkdtemp(path.join(tmpdir(), 'rsx-native-env-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const output = path.join(directory, 'report.json');
  const script = `const assert = require('node:assert/strict');
assert.equal(process.env.RSX_DEMO_ROWS, '1000');
assert.equal(process.env.RSX_NATIVE_REPORT, ${JSON.stringify(output)});
assert.equal(process.env.RSX_TEST_LABEL, '中文😀');
require('node:fs').writeFileSync(process.env.RSX_NATIVE_REPORT, ${JSON.stringify(JSON.stringify(report))});
process.exit(Number(process.env.RSX_TEST_EXIT));`;
  for (const code of [0, 7]) {
    const running = probe({ command: process.execPath, args: ['-e', script], rows: 1000, output,
      env: { RSX_DEMO_ROWS: '10', RSX_NATIVE_REPORT: '/not-the-report', RSX_TEST_LABEL: '中文😀', RSX_TEST_EXIT: String(code) } });
    if (code === 0) assert.equal((await running).records, 1000);
    else await assert.rejects(running, /exited 7/);
  }
});

test('UTF-8 log characters survive stdout and stderr chunk boundaries', async t => {
  const directory = await mkdtemp(path.join(tmpdir(), 'rsx-native-unicode-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const output = path.join(directory, 'report.json');
  const script = `const label = Buffer.from('原生😀采样');
process.stdout.write(label.subarray(0, 1));
process.stderr.write(label.subarray(0, 2));
setTimeout(() => {
  process.stdout.write(label.subarray(1));
  process.stderr.write(label.subarray(2));
  require('node:fs').writeFileSync(process.env.RSX_NATIVE_REPORT, ${JSON.stringify(JSON.stringify(report))});
}, 50);`;
  await probe({ command: process.execPath, args: ['-e', script], rows: 1000, output });
  const log = await readFile(`${output}.log`, 'utf8');
  assert.equal(log.split('原生😀采样').length - 1, 2, log);
  assert.ok(!log.includes('\uFFFD'), log);
});

test('timeout force-kills a resistant child and restores signal listeners', { skip: process.platform === 'win32' }, async t => {
  const directory = await mkdtemp(path.join(tmpdir(), 'rsx-native-kill-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const output = path.join(directory, 'report.json');
  const pidFile = path.join(directory, 'pid');
  const counts = ['SIGINT', 'SIGTERM'].map(signal => process.listenerCount(signal));
  const script = `require('node:fs').writeFileSync(${JSON.stringify(pidFile)}, String(process.pid));
process.on('SIGTERM', () => console.log('ignored termination'));
setInterval(() => {}, 1000);`;
  await assert.rejects(probe({ command: process.execPath, args: ['-e', script], rows: 1000, output, timeout: 1500 }), /timed out/);
  const pid = Number(await readFile(pidFile, 'utf8'));
  assert.throws(() => process.kill(pid, 0), { code: 'ESRCH' });
  assert.match(await readFile(`${output}.log`, 'utf8'), /ignored termination/);
  assert.deepEqual(['SIGINT', 'SIGTERM'].map(signal => process.listenerCount(signal)), counts);
});

for (const signal of ['SIGINT', 'SIGTERM']) {
  test(`native probe cleans up on ${signal}`, { skip: process.platform === 'win32' }, async t => {
    const { spawn } = await import('node:child_process');
    const directory = await mkdtemp(path.join(tmpdir(), 'rsx-native-signal-'));
    t.after(() => rm(directory, { recursive: true, force: true }));
    const output = path.join(directory, 'report.json');
    const pidFile = path.join(directory, 'pid');
    const fixture = `require('node:fs').writeFileSync(${JSON.stringify(pidFile)}, String(process.pid)); setInterval(() => {}, 1000);`;
    const wrapper = `import { probe } from ${JSON.stringify(new URL('./measure-native.mjs', import.meta.url).href)};
try { await probe({ command: process.execPath, args: ['-e', ${JSON.stringify(fixture)}], rows: 1000, output: ${JSON.stringify(output)} }); }
catch (error) { console.error(error.message); process.exitCode = 12; }`;
    const child = spawn(process.execPath, ['--input-type=module', '-e', wrapper], { stdio: ['ignore', 'pipe', 'pipe'] });
    t.after(() => { if (child.exitCode === null) child.kill('SIGKILL'); });
    let log = '';
    child.stderr.setEncoding('utf8').on('data', chunk => { log += chunk; });
    const exited = new Promise((resolve, reject) => { child.once('error', reject); child.once('close', resolve); });
    const deadline = Date.now() + 5000;
    let pid;
    while (Date.now() < deadline) {
      try { pid = Number(await readFile(pidFile, 'utf8')); if (pid) break; } catch (error) { if (error.code !== 'ENOENT') throw error; }
      await delay(20);
    }
    assert.ok(pid, `Native child did not start: ${log}`);
    child.kill(signal);
    assert.equal(await exited, 12, log);
    assert.match(log, /Native probe interrupted/);
    assert.throws(() => process.kill(pid, 0), { code: 'ESRCH' });
  });
}
