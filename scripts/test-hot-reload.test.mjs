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

for (const scenario of ['patch', 'reset', 'timeout', 'exit']) {
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
console.log('GPUI_RSX_LIVE pid=' + process.pid + ' seconds=7 clicks=2 revision=revision-a');
const scenario = ${JSON.stringify(scenario)};
setInterval(() => {
  if (scenario === 'exit') process.exit(3);
  if (scenario === 'timeout') return;
  if (!source().includes('REVISION B / 补丁已应用') || !source().includes('bg-teal-600')) return;
  console.log('GPUI_RSX_LIVE pid=' + process.pid + ' seconds=' + (scenario === 'reset' ? 0 : 8) + ' clicks=2 revision=revision-b');
}, 20);
`);
    await chmod(dx, 0o755);
    const options = { dx, target, temporaryRoot: directory, initialTimeout: 3000,
      patchTimeout: scenario === 'timeout' ? 200 : 3000, pollInterval: 10, output() {} };
    if (scenario === 'patch') {
      const verification = await probe(options);
      assert.equal(verification.after.seconds, 8);
      assert.equal(verification.after.clicks, 2);
      assert.deepEqual(JSON.parse(await readFile(path.join(target, 'verification.json'), 'utf8')), verification);
    } else {
      const expected = { reset: /Timer state was reset/, timeout: /No matching rendered frame/, exit: /dx exited before verification \(3\)/ };
      await assert.rejects(probe(options), expected[scenario]);
      await assert.rejects(readFile(path.join(target, 'verification.json')), { code: 'ENOENT' });
    }
    const pid = Number(await readFile(path.join(target, 'pid'), 'utf8'));
    assert.throws(() => process.kill(pid, 0), { code: 'ESRCH' });
    assert.match(await readFile(path.join(target, 'dx.log'), 'utf8'), /revision-a/);
    const retained = (await readdir(directory)).filter(name => name.startsWith('gpui-rsx-kit-patch-'));
    assert.equal(retained.length, scenario === 'patch' ? 0 : 1);
    if (scenario !== 'patch') {
      assert.match(await readFile(path.join(directory, retained[0], 'src/live.rs'), 'utf8'), /REVISION B \/ 补丁已应用/);
    }
  });
}
