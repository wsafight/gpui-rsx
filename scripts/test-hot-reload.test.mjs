import assert from 'node:assert/strict';
import test from 'node:test';
import { chmod, mkdir, mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { frames, preserved, probe } from './test-hot-reload.mjs';

test('render observations survive chunked/noisy output and retain Unicode labels', () => {
  const log = 'compiling…\nGPUI_RSX_LIVE pid=123 seconds=4 clicks=2 revision=revision-a\n'
    + 'patch applied\nGPUI_RSX_LIVE pid=123 seconds=9 clicks=2 revision=revision-b\n';
  const [before, after] = frames(log);
  assert.equal(before.seconds, 4);
  assert.equal(after.revision, 'revision-b');
  preserved(before, after);
});

test('patch verification rejects restarts, reset state and unchanged revisions', () => {
  const before = { pid: 1, seconds: 10, clicks: 3, revision: 'revision-a' };
  for (const change of [{ pid: 2 }, { seconds: 0 }, { clicks: 0 }, { revision: 'revision-a' }]) {
    assert.throws(() => preserved(before, { ...before, revision: 'revision-b', ...change }));
  }
});

test('incomplete render lines cannot be mistaken for an applied revision', () => {
  const line = 'GPUI_RSX_LIVE pid=123 seconds=4 clicks=2 revision=revision-b-stale';
  for (let end = 0; end <= line.length; end++) assert.deepEqual(frames(line.slice(0, end)), []);
  assert.equal(frames(`${line}\r\n`)[0].revision, 'revision-b-stale');
  assert.equal(frames(`${line}\n`)[0].revision, 'revision-b-stale');
  assert.deepEqual(frames('GPUI_RSX_LIVE pid=bad seconds=4 clicks=2 revision=revision-b\n'), []);
});

for (const scenario of ['patch', 'reset', 'click-reset', 'restart', 'timeout', 'exit', 'build-failure',
  'initial-build-failure', 'initial-timeout', 'unicode']) {
  test(`probe handles ${scenario} and cleans up its process`, { skip: process.platform === 'win32' }, async t => {
    const directory = await mkdtemp(path.join(tmpdir(), 'rsx-probe-test-'));
    t.after(() => rm(directory, { recursive: true, force: true }));
    const target = path.join(directory, 'target');
    const dx = path.join(directory, 'dx');
    await mkdir(target);
    await writeFile(path.join(target, 'verification.json'), '{"stale":true}');
    await writeFile(dx, `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
if (process.argv.includes('--version')) { console.log('dioxus 0.7.10'); process.exit(0); }
fs.writeFileSync(path.join(process.env.CARGO_TARGET_DIR, 'pid'), String(process.pid));
const source = () => fs.readFileSync('src/live.rs', 'utf8');
if (!source().includes('REVISION A / 绿色')) process.exit(5);
const scenario = ${JSON.stringify(scenario)};
if (!scenario.startsWith('initial-')) console.log('GPUI_RSX_LIVE pid=' + process.pid + ' seconds=7 clicks=2 revision=revision-a');
else console.log('Waiting for the initial build');
if (scenario === 'unicode') {
  const label = Buffer.from('热更新😀日志\\n');
  process.stdout.write(label.subarray(0, 1));
  process.stderr.write(label.subarray(0, 2));
  setTimeout(() => { process.stdout.write(label.subarray(1)); process.stderr.write(label.subarray(2)); }, 50);
}
setInterval(() => {
  if (scenario === 'exit') process.exit(3);
  if (scenario.endsWith('build-failure')) { console.error('ERROR \\x1b[31mBuild failed\\x1b[0m: fixture failure'); return; }
  if (scenario.endsWith('timeout')) return;
  if (!source().includes('REVISION B / 补丁已应用') || !source().includes('bg-teal-600')) return;
  if (scenario === 'unicode' && !globalThis.ready) return;
  console.log('GPUI_RSX_LIVE pid=' + (process.pid + (scenario === 'restart' ? 1 : 0)) + ' seconds=' + (scenario === 'reset' ? 0 : 8) + ' clicks=' + (scenario === 'click-reset' ? 0 : 2) + ' revision=revision-b');
}, 20);
setTimeout(() => { globalThis.ready = true; }, 80);
`);
    await chmod(dx, 0o755);
    const options = { dx, target, temporaryRoot: directory, initialTimeout: scenario === 'initial-timeout' ? 1000 : 5000,
      patchTimeout: scenario === 'timeout' ? 200 : 3000, pollInterval: 10, output() {} };
    if (scenario === 'patch' || scenario === 'unicode') {
      const verification = await probe(options);
      assert.equal(verification.after.seconds, 8);
      assert.equal(verification.after.clicks, 2);
      assert.ok(verification.saveToRenderObservationMs >= 0);
      assert.equal(verification.observationPollMs, options.pollInterval);
      assert.deepEqual(JSON.parse(await readFile(path.join(target, 'verification.json'), 'utf8')), verification);
    } else {
      const expected = { reset: /Timer state was reset/, timeout: /No matching rendered frame/, exit: /dx exited before verification \(3\)/,
        'click-reset': /Click state was reset/, restart: /application restarted/,
        'build-failure': /dx build failed before verification/, 'initial-build-failure': /dx build failed before verification/,
        'initial-timeout': /No matching rendered frame/ };
      await assert.rejects(probe(options), expected[scenario]);
      await assert.rejects(readFile(path.join(target, 'verification.json')), { code: 'ENOENT' });
    }
    const pid = Number(await readFile(path.join(target, 'pid'), 'utf8'));
    assert.throws(() => process.kill(pid, 0), { code: 'ESRCH' });
    const log = await readFile(path.join(target, 'dx.log'), 'utf8');
    assert.match(log, scenario.startsWith('initial-') ? /Waiting for the initial build/ : /revision-a/);
    if (scenario === 'unicode') {
      assert.equal(log.split('热更新😀日志').length - 1, 2, log);
      assert.ok(!log.includes('\uFFFD'), log);
    }
    const retained = (await readdir(directory)).filter(name => name.startsWith('gpui-rsx-kit-patch-'));
    const succeeded = scenario === 'patch' || scenario === 'unicode';
    assert.equal(retained.length, succeeded ? 0 : 1);
    if (!succeeded) {
      const retainedSource = await readFile(path.join(directory, retained[0], 'src/live.rs'), 'utf8');
      assert.match(retainedSource, scenario.startsWith('initial-') ? /REVISION A \/ 绿色/ : /REVISION B \/ 补丁已应用/);
    }
  });
}
