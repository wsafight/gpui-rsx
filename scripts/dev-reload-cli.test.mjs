import assert from 'node:assert/strict';
import { execFile, spawn } from 'node:child_process';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import * as os from 'node:os';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';
import { setTimeout as delay } from 'node:timers/promises';
import { test } from 'node:test';

const script = fileURLToPath(new URL('./dev-reload.mjs', import.meta.url));
const execute = promisify(execFile);

async function waitFor(read, message) {
  const deadline = Date.now() + 10_000;
  while (Date.now() < deadline) {
    const value = await read();
    if (value) return value;
    await delay(20);
  }
  assert.fail(message);
}

async function records(file) {
  let source;
  try { source = await readFile(file, 'utf8'); } catch (error) {
    if (error.code === 'ENOENT') return [];
    throw error;
  }
  return source.trim().split('\n').filter(Boolean).map((line) => JSON.parse(line));
}

async function fixture(context, source, { customWatch = true } = {}) {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'rsx reload cli '));
  const bin = path.join(directory, 'fake bin');
  const watched = path.join(directory, customWatch ? 'watched' : 'demo/src');
  await mkdir(bin);
  await mkdir(watched, { recursive: true });
  const manifest = path.join(directory, customWatch ? 'Cargo.toml' : 'demo/Cargo.toml');
  const sourceFile = path.join(watched, 'view.rs');
  const events = path.join(directory, 'events.jsonl');
  const report = path.join(directory, 'reload report.json');
  const app = path.join(bin, 'fixture-app');
  await writeFile(manifest, '[package]\nname = "fixture-app"\n');
  await writeFile(sourceFile, source);
  if (!customWatch) {
    await mkdir(path.join(directory, 'crates/gpui-rsx-core/src'), { recursive: true });
    await writeFile(path.join(directory, 'Cargo.toml'), '[workspace]\n');
    await writeFile(path.join(directory, 'crates/gpui-rsx-core/Cargo.toml'), '[package]\nname = "core"\n');
  }
  for (const [name, output] of [['reload-cargo.cjs', path.join(bin, 'cargo')], ['reload-app.cjs', app]]) {
    const code = await readFile(new URL(`./fixtures/${name}`, import.meta.url), 'utf8');
    await writeFile(output, '#!/usr/bin/env node\n' + code, { mode: 0o755 });
  }
  const child = spawn(process.execPath, [script, '--manifest-path', manifest, '--bin', 'fixture-app',
    '--toolchain', 'fixture-toolchain', ...(customWatch ? ['--watch', watched] : []), '--report', report, '--offline'], {
    cwd: directory,
    env: { ...process.env, PATH: bin + path.delimiter + process.env.PATH,
      GPUI_RSX_TEST_EVENTS: events, GPUI_RSX_TEST_SOURCE: sourceFile, GPUI_RSX_TEST_APP: app },
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  let stdout = '';
  let stderr = '';
  child.stdout.on('data', (data) => { stdout += data; });
  child.stderr.on('data', (data) => { stderr += data; });
  const finished = new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('close', (code, signal) => resolve({ code, signal, stdout, stderr }));
  });
  const metrics = () => stderr.split('\n').flatMap((line) => {
    try { return [JSON.parse(line)]; } catch { return []; }
  });
  context.after(async () => {
    if (child.exitCode === null && child.signalCode === null) child.kill('SIGTERM');
    try { await finished; } finally {
      const final = await records(events);
      const stopped = new Set(final.filter((entry) => ['build-finished', 'app-stopped'].includes(entry.type))
        .map((entry) => entry.pid));
      for (const entry of final) {
        if (!['build', 'app-started'].includes(entry.type)) continue;
        if (stopped.has(entry.pid)) continue;
        try { process.kill(entry.pid, 'SIGKILL'); } catch (error) {
          if (error.code !== 'ESRCH') throw error;
        }
      }
      await rm(directory, { recursive: true, force: true });
    }
  });
  return { child, finished, metrics, directory, manifest, sourceFile, events, report };
}

test('CLI documents options and rejects invalid arguments before building', async () => {
  const help = await execute(process.execPath, [script, '--help']);
  assert.match(help.stdout, /--watch PATH/);
  assert.match(help.stdout, /--report FILE/);
  for (const [args, message] of [
    [['--bin'], 'Missing value for --bin'],
    [['--unknown', 'value'], 'Unknown option --unknown'],
  ]) {
    await assert.rejects(execute(process.execPath, [script, ...args]), (error) => {
      assert.equal(error.code, 1);
      assert.ok(error.stderr.includes(message), error.stderr);
      return true;
    });
  }
});

test('CLI watches custom roots, survives failure and writes metrics while cleaning up', {
  skip: process.platform === 'win32', timeout: 15_000,
}, async (context) => {
  const run = await fixture(context, 'success');
  await waitFor(() => run.metrics().filter((entry) => entry.ready).length === 1, 'first application never became ready');
  const initial = await records(run.events);
  const first = initial.find((entry) => entry.type === 'app-started');
  assert.equal(first.readyFlag, '1');
  await writeFile(path.join(run.directory, 'outside.rs'), 'outside the watch roots');
  await delay(450);
  assert.equal((await records(run.events)).filter((entry) => entry.type === 'build').length, 1);

  await writeFile(run.sourceFile, 'fail');
  await waitFor(() => run.metrics().some((entry) => !entry.ok), 'compilation failure was not reported');
  const failed = await records(run.events);
  assert.equal(failed.filter((entry) => entry.type === 'app-started').length, 1);
  assert.equal(failed.filter((entry) => entry.type === 'app-stopped').length, 0);
  assert.doesNotThrow(() => process.kill(first.pid, 0));

  await writeFile(run.sourceFile, 'success after recovery');
  await waitFor(() => run.metrics().filter((entry) => entry.ready).length === 2, 'application did not recover');
  run.child.kill('SIGINT');
  const result = await run.finished;
  assert.equal(result.code, 0, result.stderr);
  assert.match(result.stdout, /application output/);
  const report = JSON.parse(await readFile(run.report, 'utf8'));
  assert.equal(report.filter((entry) => entry.ready).length, 2);
  assert.equal(report.filter((entry) => !entry.ok).length, 1);
  for (const entry of report.filter((entry) => entry.ready)) {
    assert.ok(entry.editToReadyMs >= entry.buildMs);
    assert.ok(entry.restartMs >= 0);
  }
  const final = await records(run.events);
  const builds = final.filter((entry) => entry.type === 'build');
  assert.equal(builds.length, 3);
  for (const build of builds) {
    assert.deepEqual(build.args, ['+fixture-toolchain', 'build', '--manifest-path', run.manifest,
      '--bin', 'fixture-app', '--locked', '--message-format=json-render-diagnostics', '--offline']);
  }
  const started = final.filter((entry) => entry.type === 'app-started').map((entry) => entry.pid);
  const stopped = final.filter((entry) => entry.type === 'app-stopped').map((entry) => entry.pid);
  assert.deepEqual(stopped, started);
  for (const pid of started) assert.throws(() => process.kill(pid, 0), { code: 'ESRCH' });
});

test('CLI shutdown cancels a running Cargo process and still writes its report', {
  skip: process.platform === 'win32', timeout: 15_000,
}, async (context) => {
  const run = await fixture(context, 'block');
  await waitFor(async () => (await records(run.events)).some((entry) => entry.type === 'build'), 'Cargo never started');
  run.child.kill('SIGTERM');
  const result = await run.finished;
  assert.equal(result.code, 0, result.stderr);
  await waitFor(async () => (await records(run.events)).some((entry) => entry.type === 'build-stopped'), 'Cargo was not cancelled');
  const report = JSON.parse(await readFile(run.report, 'utf8'));
  assert.equal(report.filter((entry) => entry.ready).length, 0);
  assert.ok(report.some((entry) => !entry.ok && /abort/i.test(entry.error)));
});

test('CLI default roots rebuild after macro and core manifest changes', {
  skip: process.platform === 'win32', timeout: 15_000,
}, async (context) => {
  const run = await fixture(context, 'success', { customWatch: false });
  await waitFor(() => run.metrics().filter((entry) => entry.ready).length === 1, 'first application never became ready');
  await mkdir(path.join(run.directory, 'crates/gpui-rsx-core/tests'));
  await writeFile(path.join(run.directory, 'crates/gpui-rsx-core/tests/unrelated.rs'), 'not an application dependency');
  await delay(450);
  assert.equal((await records(run.events)).filter((entry) => entry.type === 'build').length, 1);
  let ready = 1;
  for (const relative of ['Cargo.toml', 'crates/gpui-rsx-core/Cargo.toml']) {
    await writeFile(path.join(run.directory, relative), '[dependencies]\nchanged = "1"\n');
    ready += 1;
    await waitFor(() => run.metrics().filter((entry) => entry.ready).length === ready, `${relative} change did not rebuild`);
  }
  run.child.kill('SIGTERM');
  const result = await run.finished;
  assert.equal(result.code, 0, result.stderr);
  assert.equal((await records(run.events)).filter((entry) => entry.type === 'build').length, 3);
});
